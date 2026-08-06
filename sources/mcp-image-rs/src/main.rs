// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ----------------------------------------------------------------------------
// Variables d'environnement (fichier .env ou shell) :
//   COMFYUI_URL          — URL du serveur ComfyUI (défaut : http://127.0.0.1:8188)
//   DEFAULT_MODEL        — Modèle .safetensors par défaut
//   DEFAULT_NEGATIVE     — Prompt négatif par défaut
//   DEFAULT_WIDTH        — Largeur par défaut en pixels (défaut : 512)
//   DEFAULT_HEIGHT       — Hauteur par défaut en pixels (défaut : 512)
//   DEFAULT_STEPS        — Nombre d'étapes par défaut (défaut : 20)
//   DEFAULT_CFG          — CFG scale par défaut (défaut : 7.0)
//   DEFAULT_SAMPLER      — Sampler par défaut (défaut : euler)
//   DEFAULT_SCHEDULER    — Scheduler par défaut (défaut : normal)
//   HTTP_TIMEOUT_SECS    — Timeout HTTP vers ComfyUI en secondes (défaut : 120)
//   MCP_HOST             — Adresse d'écoute (défaut : 0.0.0.0)
//   MCP_PORT             — Port d'écoute (défaut : 6510)
//   MCP_PATH             — Chemin de l'endpoint MCP (défaut : /mcp)
// ============================================================================

// ============================================================================
// main.rs — Serveur MCP HTTP Streamable — Génération d'images par IA
// ============================================================================


mod comfyui;
mod mcp;
mod tools;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::{Parser, ValueEnum};
use mcp::{
    Capabilities, InitializeResult, JsonRpcRequest, JsonRpcResponse,
    ServerInfo, ToolsCapability, ToolsListResult,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use tools::{all_tools, dispatch, Defaults};

// ---------------------------------------------------------------------------
// CLI — toutes les valeurs sont également lisibles depuis le .env
// ---------------------------------------------------------------------------

/// Transport utilisé pour exposer le serveur MCP.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Transport {
    /// HTTP Streamable (spec 2025-03-26) — pour usage réseau / partagé.
    Http,
    /// stdio — un message JSON-RPC par ligne sur stdin/stdout.
    /// C'est le mode utilisé quand le serveur est lancé comme sous-processus
    /// par un client MCP (ex : Demeter).
    Stdio,
}

/// Serveur MCP — Génération d'images par IA via ComfyUI
/// Supporte deux transports : HTTP Streamable et stdio.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Transport à utiliser
    #[arg(long, value_enum, default_value_t = Transport::Http, env = "MCP_TRANSPORT")]
    transport: Transport,

    /// Adresse d'écoute du serveur MCP (mode http uniquement)
    #[arg(long, default_value = "0.0.0.0", env = "MCP_HOST")]
    host: String,

    /// Port d'écoute du serveur MCP (mode http uniquement)
    #[arg(long, default_value_t = 6510, env = "MCP_PORT")]
    port: u16,

    /// Chemin de l'endpoint MCP (mode http uniquement)
    #[arg(long, default_value = "/mcp", env = "MCP_PATH")]
    path: String,

    /// URL de l'API ComfyUI
    #[arg(long, default_value = "http://127.0.0.1:8188", env = "COMFYUI_URL")]
    comfyui_url: String,

    /// Modèle .safetensors par défaut
    #[arg(long, default_value = "v1-5-pruned-emaonly.safetensors", env = "DEFAULT_MODEL")]
    default_model: String,

    /// Prompt négatif par défaut
    #[arg(long, default_value = "ugly, blurry, low quality, watermark", env = "DEFAULT_NEGATIVE")]
    default_negative: String,

    /// Largeur d'image par défaut (pixels, multiple de 64)
    #[arg(long, default_value_t = 512, env = "DEFAULT_WIDTH")]
    default_width: u32,

    /// Hauteur d'image par défaut (pixels, multiple de 64)
    #[arg(long, default_value_t = 512, env = "DEFAULT_HEIGHT")]
    default_height: u32,

    /// Nombre d'étapes de débruitage par défaut
    #[arg(long, default_value_t = 20, env = "DEFAULT_STEPS")]
    default_steps: u32,

    /// CFG scale par défaut
    #[arg(long, default_value_t = 7.0, env = "DEFAULT_CFG")]
    default_cfg: f32,

    /// Sampler par défaut
    #[arg(long, default_value = "euler", env = "DEFAULT_SAMPLER")]
    default_sampler: String,

    /// Scheduler par défaut
    #[arg(long, default_value = "normal", env = "DEFAULT_SCHEDULER")]
    default_scheduler: String,

    /// Timeout HTTP vers ComfyUI (secondes)
    #[arg(long, default_value_t = 120, env = "HTTP_TIMEOUT_SECS")]
    http_timeout_secs: u64,
}

// ---------------------------------------------------------------------------
// État partagé
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    tools:       Arc<Vec<mcp::ToolDef>>,
    http_client: Client,
    comfyui_url: Arc<String>,
    defaults:    Arc<Defaults>,
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    // Charger le fichier .env s'il existe (silencieux si absent)
    dotenvy::dotenv().ok();

    let args = Args::parse();

    // En mode stdio, stdout est le canal JSON-RPC : les logs DOIVENT partir
    // sur stderr, sous peine de corrompre le flux et de casser le client MCP.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_image=info,tower_http=warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let defaults = Arc::new(Defaults {
        model:     args.default_model.clone(),
        negative:  args.default_negative.clone(),
        width:     args.default_width,
        height:    args.default_height,
        steps:     args.default_steps,
        cfg_scale: args.default_cfg,
        sampler:   args.default_sampler.clone(),
        scheduler: args.default_scheduler.clone(),
    });

    // Client HTTP réutilisable (pool de connexions vers ComfyUI)
    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(args.http_timeout_secs))
        .build()
        .expect("Impossible de créer le client HTTP");

    let tools = Arc::new(all_tools(&defaults));

    let state = AppState {
        tools,
        http_client,
        comfyui_url: Arc::new(args.comfyui_url.clone()),
        defaults,
    };

    match args.transport {
        Transport::Stdio => run_stdio(state).await,
        Transport::Http => run_http(state, &args).await,
    }
}

// ---------------------------------------------------------------------------
// Transport HTTP Streamable
// ---------------------------------------------------------------------------

async fn run_http(state: AppState, args: &Args) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let num_tools = state.tools.len();
    let mcp_path = args.path.clone();
    let app: Router = Router::new()
        .route(&mcp_path, post(handle_mcp))
        .route(&mcp_path, get(handle_mcp_get))
        .route("/health", get(handle_health))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Impossible d'écouter sur {addr}: {e}"));

    info!("🎨 Serveur MCP ImageGen démarré sur http://{addr}{}", args.path);
    info!("   Backend ComfyUI    : {}", args.comfyui_url);
    info!("   Modèle par défaut  : {}", args.default_model);
    info!("   Timeout HTTP       : {}s", args.http_timeout_secs);
    info!("   {} outils disponibles — spec MCP 2025-03-26 HTTP Streamable", num_tools);

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Erreur serveur : {e}"));
}

// ---------------------------------------------------------------------------
// Transport stdio — un message JSON-RPC par ligne sur stdin/stdout
// ---------------------------------------------------------------------------
//
// Convention MCP stdio : le client écrit une requête JSON-RPC (terminée par
// \n) sur le stdin du process, le serveur répond par une ligne JSON-RPC sur
// stdout. Les notifications (pas de "id") n'appellent aucune réponse.
// Tout ce qui n'est pas du JSON-RPC (logs, warnings...) doit partir sur
// stderr — jamais sur stdout. Les images générées transitent en base64 dans
// le champ "data" d'un ContentBlock JSON classique : aucune adaptation
// nécessaire par rapport au transport HTTP pour ce flux binaire.

async fn run_stdio(state: AppState) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    info!(
        "🎨 Serveur MCP ImageGen démarré en mode stdio — {} outils disponibles — backend ComfyUI : {}",
        state.tools.len(),
        state.comfyui_url,
    );

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut lines = BufReader::new(stdin).lines();

    loop {
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => {
                info!("stdin fermé — arrêt du serveur.");
                break;
            }
            Err(e) => {
                error!("Erreur de lecture stdin : {e}");
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let val: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let resp = JsonRpcResponse::err(None, -32700, format!("JSON invalide : {e}"));
                write_stdio_line(&mut stdout, &resp).await;
                continue;
            }
        };

        // Un batch de requêtes est possible en JSON-RPC (tableau).
        if let Some(arr) = val.as_array() {
            for req_val in arr.clone() {
                let is_notification = req_val.get("id").is_none();
                let resp = process_request(req_val, &state).await;
                if !is_notification {
                    write_stdio_line(&mut stdout, &resp).await;
                }
            }
            continue;
        }

        let is_notification = val.get("id").is_none();
        let resp = process_request(val, &state).await;
        // Les notifications (ex : notifications/initialized) n'attendent pas
        // de réponse JSON-RPC.
        if !is_notification {
            write_stdio_line(&mut stdout, &resp).await;
        }
    }
}

async fn write_stdio_line(
    stdout: &mut (impl tokio::io::AsyncWrite + Unpin),
    resp: &JsonRpcResponse,
) {
    use tokio::io::AsyncWriteExt;
    let mut line = serde_json::to_string(resp).unwrap_or_default();
    line.push('\n');
    if let Err(e) = stdout.write_all(line.as_bytes()).await {
        error!("Erreur d'écriture stdout : {e}");
        return;
    }
    let _ = stdout.flush().await;
}

// ---------------------------------------------------------------------------
// Handler GET (discovery / ping)
// ---------------------------------------------------------------------------

async fn handle_mcp_get() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"server":"mcp-image","status":"ok","protocol":"MCP 2025-03-26"}"#,
    )
}

async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// ---------------------------------------------------------------------------
// Handler POST principal — JSON-RPC 2.0
// ---------------------------------------------------------------------------

async fn handle_mcp(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Response {
    let text = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return json_response(JsonRpcResponse::err(None, -32700, "Corps non-UTF-8")),
    };

    let parsed: Result<Value, _> = serde_json::from_str(text);
    let val = match parsed {
        Ok(v) => v,
        Err(e) => {
            return json_response(JsonRpcResponse::err(
                None,
                -32700,
                format!("JSON invalide : {e}"),
            ));
        }
    };

    // Batch JSON-RPC
    if val.is_array() {
        let reqs: Vec<Value> = val.as_array().cloned().unwrap_or_default();
        let mut responses = Vec::new();
        for req_val in reqs {
            let resp = process_request(req_val, &state).await;
            responses.push(serde_json::to_value(resp).unwrap_or(Value::Null));
        }
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&responses).unwrap_or_default(),
        )
            .into_response();
    }

    let resp = process_request(val, &state).await;
    json_response(resp)
}

// ---------------------------------------------------------------------------
// Traitement d'une requête JSON-RPC individuelle
// ---------------------------------------------------------------------------

async fn process_request(val: Value, state: &AppState) -> JsonRpcResponse {
    let req: JsonRpcRequest = match serde_json::from_value(val) {
        Ok(r) => r,
        Err(e) => {
            return JsonRpcResponse::err(None, -32600, format!("Requête invalide : {e}"));
        }
    };

    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2025-03-26",
                capabilities: Capabilities {
                    tools: ToolsCapability { list_changed: false },
                },
                server_info: ServerInfo {
                    name:    "image",
                    version: env!("CARGO_PKG_VERSION"),
                },
                instructions: "Serveur MCP de génération d'images par IA. \
                    Utilise ComfyUI comme moteur de diffusion local. \
                    Outil principal : generate_image(prompt) — retourne une image PNG base64. \
                    Outil utilitaire : list_models — liste les modèles disponibles dans ComfyUI. \
                    Les images générées sont retournées directement dans la réponse MCP \
                    sous forme de ContentBlock de type 'image' (base64 PNG).",
            };
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }

        "notifications/initialized" => JsonRpcResponse::ok(id, json!({})),

        "ping" => JsonRpcResponse::ok(id, json!({})),

        "tools/list" => {
            let result = ToolsListResult { tools: state.tools.as_ref().clone() };
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }

        "tools/call" => {
            let params    = req.params.unwrap_or(Value::Null);
            let tool_name = params["name"].as_str().unwrap_or("").to_string();
            let args      = params.get("arguments").cloned().unwrap_or(json!({}));

            if tool_name.is_empty() {
                return JsonRpcResponse::err(
                    id,
                    -32602,
                    "Paramètre 'name' manquant dans tools/call",
                );
            }

            info!("→ tools/call : {tool_name}");
            let result = dispatch(
                &tool_name,
                &args,
                &state.http_client,
                &state.comfyui_url,
                &state.defaults,
            )
            .await;
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }

        other => {
            error!("Méthode MCP inconnue : {other}");
            JsonRpcResponse::err(id, -32601, format!("Méthode inconnue : {other}"))
        }
    }
}

// ---------------------------------------------------------------------------
// Helper : construire une Response JSON
// ---------------------------------------------------------------------------

fn json_response(resp: JsonRpcResponse) -> Response {
    let body = serde_json::to_string(&resp).unwrap_or_default();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response()
}
