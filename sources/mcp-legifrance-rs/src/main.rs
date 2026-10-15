// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com (portage Rust)
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ----------------------------------------------------------------------------
// Démarrage :
//   LEGIFRANCE_CLIENT_ID=... LEGIFRANCE_CLIENT_SECRET=... ./mcp-legifrance
//   ./mcp-legifrance --host 127.0.0.1 --port 6503
//
// Endpoint MCP :
//   http://localhost:6503/mcp
// ============================================================================

mod client;
mod fmt;
mod mcp;
mod tools;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::{Parser, ValueEnum};
use mcp::{
    Capabilities, InitializeResult, JsonRpcRequest, JsonRpcResponse,
    ServerInfo, ToolsCapability,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use client::LegifranceClient;
use tools::{all_tools, dispatch};

// ---------------------------------------------------------------------------
// CLI
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

/// Serveur MCP HTTP Streamable — API Légifrance (version Rust)
/// Supporte deux transports : HTTP Streamable et stdio.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Transport à utiliser
    #[arg(long, value_enum, default_value_t = Transport::Http, env = "MCP_TRANSPORT")]
    transport: Transport,
    /// Adresse d'écoute (mode http uniquement). 127.0.0.1 par défaut : le serveur
    /// n'a pas d'authentification et consomme VOTRE quota PISTE ; ne l'exposez
    /// sur 0.0.0.0 (MCP_HOST) que derrière un proxy authentifié.
    #[arg(long, default_value = "127.0.0.1", env = "MCP_HOST")]
    host: String,
    /// Port d'écoute (mode http uniquement)
    #[arg(long, default_value_t = 6503, env = "MCP_PORT")]
    port: u16,
    /// Chemin de l'endpoint MCP (mode http uniquement)
    #[arg(long, default_value = "/mcp", env = "MCP_PATH")]
    path: String,
}

// ---------------------------------------------------------------------------
// État partagé
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct AppState {
    client: Arc<LegifranceClient>,
    tools: Arc<Vec<mcp::ToolDef>>,
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------
#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    // En mode stdio, stdout est le canal JSON-RPC : les logs DOIVENT partir sur
    // stderr, sous peine de corrompre le flux et de casser le client MCP.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_legifrance=info,tower_http=warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let client_id     = std::env::var("LEGIFRANCE_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("LEGIFRANCE_CLIENT_SECRET").unwrap_or_default();

    if client_id.is_empty() || client_secret.is_empty() {
        warn!("⚠️  LEGIFRANCE_CLIENT_ID / LEGIFRANCE_CLIENT_SECRET non définis.");
        warn!("   Obtenez vos credentials sur https://piste.gouv.fr");
    } else {
        info!("✅ Credentials Légifrance détectés.");
    }
    let env_label = if std::env::var("LEGIFRANCE_ENV").as_deref() == Ok("sandbox") {
        "SANDBOX"
    } else {
        "PRODUCTION"
    };
    info!(
        "🌐 Environnement PISTE : {} ({}) — pour basculer, définissez LEGIFRANCE_ENV=sandbox",
        env_label,
        client::base_url()
    );

    let state = AppState {
        client: Arc::new(LegifranceClient::new(client_id, client_secret)),
        tools:  Arc::new(all_tools()),
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

    let mcp_path = args.path.clone();
    let app = Router::new()
        .route(&mcp_path, post(handle_mcp))
        .route(&mcp_path, get(handle_mcp_get))
        .route("/health", get(handle_health))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Impossible d'écouter sur {addr}: {e}"));

    info!("🚀 Serveur MCP Légifrance démarré sur http://{addr}{}", args.path);
    info!("   {} outils disponibles — spec MCP 2025-03-26 HTTP Streamable", all_tools().len());

    axum::serve(listener, app)
        .await
        .expect("Erreur fatale du serveur HTTP");
}

// ---------------------------------------------------------------------------
// Transport stdio — un message JSON-RPC par ligne sur stdin/stdout
// ---------------------------------------------------------------------------
//
// Convention MCP stdio : le client écrit une requête JSON-RPC (terminée par
// \n) sur le stdin du process, le serveur répond par une ligne JSON-RPC sur
// stdout. Les notifications (pas de "id") n'appellent aucune réponse.
// Tout ce qui n'est pas du JSON-RPC (logs, warnings...) doit partir sur
// stderr — jamais sur stdout.

async fn run_stdio(state: AppState) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    info!(
        "🚀 Serveur MCP Légifrance démarré en mode stdio — {} outils disponibles",
        all_tools().len()
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
// Handlers
// ---------------------------------------------------------------------------
async fn handle_mcp_get() -> impl IntoResponse {
    // Streamable HTTP : ce serveur n'ouvre pas de flux SSE ; la spec impose 405.
    (StatusCode::METHOD_NOT_ALLOWED, [(header::ALLOW, "POST")], "Méthode non supportée : utilisez POST")
}

/// Protection DNS-rebinding (spec MCP, « Security Warning ») : une requête
/// portant un en-tête Origin doit venir de localhost ou d'une origine listée
/// dans MCP_ALLOWED_ORIGINS (séparées par des virgules).
fn origin_allowed(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) else { return true };
    let host = origin.split("://").nth(1).unwrap_or(origin);
    let host = host.split('/').next().unwrap_or("");
    let bare = host.rsplit_once(':').map(|(h, p)| if p.chars().all(|c| c.is_ascii_digit()) { h } else { host }).unwrap_or(host);
    if matches!(bare, "localhost" | "127.0.0.1" | "[::1]") { return true; }
    std::env::var("MCP_ALLOWED_ORIGINS").map(|l| l.split(',').any(|o| o.trim() == origin)).unwrap_or(false)
}

async fn handle_health() -> impl IntoResponse { (StatusCode::OK, "OK") }

async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    if !origin_allowed(&headers) {
        return (StatusCode::FORBIDDEN, "Origin non autorisée").into_response();
    }
    let text = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return json_response(JsonRpcResponse::err(None, -32700, "Corps non-UTF-8")),
    };
    let parsed: Result<Value, _> = serde_json::from_str(text);
    let val = match parsed {
        Ok(v) => v,
        Err(e) => return json_response(JsonRpcResponse::err(None, -32700, format!("JSON invalide : {e}"))),
    };
    if val.is_array() {
        let reqs = val.as_array().cloned().unwrap_or_default();
        let mut responses = Vec::new();
        for req_val in reqs {
            let is_notification = req_val.get("id").is_none();
            let resp = process_request(req_val, &state).await;
            if !is_notification {
                responses.push(serde_json::to_value(resp).unwrap_or(Value::Null));
            }
        }
        if responses.is_empty() {
            return StatusCode::ACCEPTED.into_response();
        }
        return (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")],
                serde_json::to_string(&responses).unwrap_or_default())
            .into_response();
    }
    let is_notification = val.get("id").is_none();
    let resp = process_request(val, &state).await;
    if is_notification {
        // Notification (ex. notifications/initialized) : 202 Accepted, sans corps.
        return StatusCode::ACCEPTED.into_response();
    }
    json_response(resp)
}

async fn process_request(val: Value, state: &AppState) -> JsonRpcResponse {
    let req: JsonRpcRequest = match serde_json::from_value(val) {
        Ok(r) => r,
        Err(e) => return JsonRpcResponse::err(None, -32600, format!("Requête invalide : {e}")),
    };
    let id = req.id.clone();
    match req.method.as_str() {
        "initialize" => {
            // Négociation : on renvoie la version du client si on la supporte,
            // sinon la nôtre la plus récente.
            const SUPPORTED: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
            let wanted = req.params.as_ref().and_then(|p| p["protocolVersion"].as_str()).unwrap_or("");
            let version: &'static str = SUPPORTED.iter().copied().find(|v| *v == wanted).unwrap_or(SUPPORTED[0]);
            let n_tools = state.tools.len();
            let result = InitializeResult {
                protocol_version: version,
                capabilities: Capabilities { tools: ToolsCapability { list_changed: false } },
                server_info: ServerInfo { name: "legifrance", version: env!("CARGO_PKG_VERSION") },
                instructions: format!("Serveur MCP pour l'API Légifrance (PISTE) — {n_tools} outils : recherche plein texte multi-fonds \
                    (codes, LODA, JORF, KALI, jurisprudence JURI/CETAT/CONSTIT/JUFI, CNIL, circulaires, accords d'entreprise), \
                    consultation des codes (table des matières, sections, articles par ID, numéro, ELI, version à une date), \
                    lois/décrets, Journal Officiel, conventions collectives, dossiers législatifs, débats et questions \
                    parlementaires, BOCC, BODMR, documents administratifs. \
                    Prérequis : LEGIFRANCE_CLIENT_ID et LEGIFRANCE_CLIENT_SECRET (application PISTE)."),
            };
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }
        m if m.starts_with("notifications/") => JsonRpcResponse::ok(id, json!({})),
        "ping" => JsonRpcResponse::ok(id, json!({})),
        "tools/list" => {
            let tools: Vec<Value> = state.tools.iter().map(|tool| {
                let mut v = serde_json::to_value(tool).unwrap_or(Value::Null);
                v["annotations"] = json!({
                    "readOnlyHint": true, "destructiveHint": false,
                    "idempotentHint": true, "openWorldHint": true
                });
                v
            }).collect();
            JsonRpcResponse::ok(id, json!({ "tools": tools }))
        }
        "tools/call" => {
            let params    = req.params.unwrap_or(Value::Null);
            let tool_name = params["name"].as_str().unwrap_or("").to_string();
            let args      = params.get("arguments").cloned().unwrap_or(json!({}));
            if tool_name.is_empty() {
                return JsonRpcResponse::err(id, -32602, "Paramètre 'name' manquant");
            }
            info!("→ tools/call : {tool_name}");
            let (c, n, a) = (state.client.clone(), tool_name.clone(), args.clone());
            let result = match tokio::spawn(async move { dispatch(&n, &a, &c).await }).await {
                Ok(r) => r,
                Err(e) => mcp::ToolCallResult::error(format!("❌ Erreur interne de l'outil : {e}")),
            };
            // On journalise le résultat (succès/erreur + aperçu du texte) car
            // jusqu'ici seul l'appel était loggé, jamais son issue — rendant
            // tout diagnostic à distance impossible sans reproduire le bug.
            let preview: String = result
                .content
                .first()
                .map(|c| c.text.chars().take(300).collect())
                .unwrap_or_default();
            if result.is_error {
                error!("← tools/call {tool_name} ÉCHEC : {preview}");
            } else {
                info!("← tools/call {tool_name} OK : {preview}");
            }
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }
        other => {
            error!("Méthode MCP inconnue : {other}");
            JsonRpcResponse::err(id, -32601, format!("Méthode inconnue : {other}"))
        }
    }
}

fn json_response(resp: JsonRpcResponse) -> Response {
    let body = serde_json::to_string(&resp).unwrap_or_default();
    (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}
