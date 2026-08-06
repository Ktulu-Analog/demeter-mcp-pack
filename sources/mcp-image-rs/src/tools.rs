// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================

// ============================================================================
// tools.rs — Outils MCP exposés par le serveur mcp-image
// ============================================================================

use crate::comfyui::{generate_image, GenerateParams};
use crate::mcp::{ToolCallResult, ToolDef};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::info;

// ---------------------------------------------------------------------------
// Valeurs par défaut centralisées (issues du .env / CLI)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Defaults {
    pub model:     String,
    pub negative:  String,
    pub width:     u32,
    pub height:    u32,
    pub steps:     u32,
    pub cfg_scale: f32,
    pub sampler:   String,
    pub scheduler: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            model:     "v1-5-pruned-emaonly.safetensors".into(),
            negative:  "ugly, blurry, low quality, watermark".into(),
            width:     512,
            height:    512,
            steps:     20,
            cfg_scale: 7.0,
            sampler:   "euler".into(),
            scheduler: "normal".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Registre des outils
// ---------------------------------------------------------------------------

/// Retourne la liste complète des outils déclarés (appelé par tools/list).
/// Les valeurs par défaut du .env sont injectées dans les descriptions JSON Schema.
pub fn all_tools(defaults: &Defaults) -> Vec<ToolDef> {
    vec![tool_generate_image(defaults), tool_list_models()]
}

// ---------------------------------------------------------------------------
// Dispatch (appelé par tools/call)
// ---------------------------------------------------------------------------

pub async fn dispatch(
    name:        &str,
    args:        &Value,
    http:        &Client,
    comfyui_url: &str,
    defaults:    &Defaults,
) -> ToolCallResult {
    match name {
        "generate_image" => handle_generate_image(args, http, comfyui_url, defaults).await,
        "list_models"    => handle_list_models(http, comfyui_url).await,
        other => ToolCallResult::error(format!("Outil inconnu : {other}")),
    }
}

// ---------------------------------------------------------------------------
// Outil 1 : generate_image
// ---------------------------------------------------------------------------

fn tool_generate_image(d: &Defaults) -> ToolDef {
    ToolDef {
        name:        "generate_image".into(),
        description: "Génère une image à partir d'un prompt texte en utilisant \
                       un modèle de diffusion local via ComfyUI. \
                       Retourne l'image encodée en base64 (PNG).".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type":        "string",
                    "description": "Description de l'image à générer (prompt positif)"
                },
                "negative_prompt": {
                    "type":        "string",
                    "description": "Ce que l'image ne doit PAS contenir (prompt négatif)",
                    "default":     d.negative
                },
                "width": {
                    "type":        "integer",
                    "description": "Largeur de l'image en pixels (multiple de 64)",
                    "default":     d.width,
                    "minimum":     64,
                    "maximum":     2048
                },
                "height": {
                    "type":        "integer",
                    "description": "Hauteur de l'image en pixels (multiple de 64)",
                    "default":     d.height,
                    "minimum":     64,
                    "maximum":     2048
                },
                "steps": {
                    "type":        "integer",
                    "description": "Nombre d'étapes de débruitage (plus = meilleure qualité, plus lent)",
                    "default":     d.steps,
                    "minimum":     1,
                    "maximum":     150
                },
                "cfg_scale": {
                    "type":        "number",
                    "description": "Guidance scale — contrôle l'adhérence au prompt (7 = équilibré)",
                    "default":     d.cfg_scale,
                    "minimum":     1.0,
                    "maximum":     30.0
                },
                "seed": {
                    "type":        "integer",
                    "description": "Graine aléatoire pour la reproductibilité (-1 = aléatoire)",
                    "default":     -1
                },
                "model": {
                    "type":        "string",
                    "description": "Nom du fichier modèle (.safetensors) présent dans ComfyUI/models/checkpoints/",
                    "default":     d.model
                }
            },
            "required": ["prompt"]
        }),
    }
}

async fn handle_generate_image(
    args:        &Value,
    http:        &Client,
    comfyui_url: &str,
    defaults:    &Defaults,
) -> ToolCallResult {
    // --- Lecture des paramètres (fallback sur les defaults du .env) ---
    let prompt = match args["prompt"].as_str() {
        Some(p) if !p.trim().is_empty() => p,
        _ => return ToolCallResult::error("Paramètre 'prompt' manquant ou vide"),
    };

    let negative_prompt = args["negative_prompt"]
        .as_str()
        .unwrap_or(&defaults.negative);

    let width  = args["width"].as_u64().unwrap_or(defaults.width as u64) as u32;
    let height = args["height"].as_u64().unwrap_or(defaults.height as u64) as u32;
    let steps  = args["steps"].as_u64().unwrap_or(defaults.steps as u64) as u32;
    let cfg    = args["cfg_scale"].as_f64().unwrap_or(defaults.cfg_scale as f64) as f32;
    let seed   = args["seed"].as_i64().unwrap_or(-1);
    let model  = args["model"].as_str().unwrap_or(&defaults.model);

    // Validation dimensions (multiples de 64)
    if width % 64 != 0 || height % 64 != 0 {
        return ToolCallResult::error(
            format!("width ({width}) et height ({height}) doivent être des multiples de 64"),
        );
    }

    info!("generate_image : prompt={prompt:?} {width}×{height} steps={steps} model={model}");

    let params = GenerateParams {
        prompt,
        negative_prompt,
        width,
        height,
        steps,
        cfg_scale: cfg,
        seed,
        model,
        sampler:   &defaults.sampler,
        scheduler: &defaults.scheduler,
    };

    // --- Appel ComfyUI ---
    match generate_image(http, comfyui_url, &params).await {
        Ok(png_bytes) => {
            let b64 = B64.encode(&png_bytes);
            let caption = format!(
                "Image générée — prompt: «{prompt}» — {width}×{height} — {steps} steps — modèle: {model}"
            );
            info!("generate_image : succès ({} octets PNG)", png_bytes.len());
            ToolCallResult::image_with_caption(b64, "image/png", caption)
        }
        Err(e) => {
            ToolCallResult::error(format!("Erreur ComfyUI : {e}"))
        }
    }
}

// ---------------------------------------------------------------------------
// Outil 2 : list_models
// ---------------------------------------------------------------------------

fn tool_list_models() -> ToolDef {
    ToolDef {
        name:        "list_models".into(),
        description: "Liste les modèles de checkpoint disponibles dans ComfyUI \
                       (fichiers .safetensors / .ckpt présents dans models/checkpoints/).".into(),
        input_schema: json!({
            "type":       "object",
            "properties": {},
            "required":   []
        }),
    }
}

async fn handle_list_models(
    http:        &Client,
    comfyui_url: &str,
) -> ToolCallResult {
    let url = format!("{}/object_info/CheckpointLoaderSimple", comfyui_url);

    match http.get(&url).send().await {
        Err(e) => ToolCallResult::error(format!("Impossible de joindre ComfyUI : {e}")),
        Ok(resp) => {
            match resp.error_for_status() {
                Err(e) => ToolCallResult::error(format!("ComfyUI a retourné une erreur : {e}")),
                Ok(r) => {
                    match r.json::<Value>().await {
                        Err(e) => ToolCallResult::error(format!("Réponse JSON invalide : {e}")),
                        Ok(info) => {
                            let models = info
                                .pointer("/CheckpointLoaderSimple/input/required/ckpt_name/0")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|m| m.as_str())
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                })
                                .unwrap_or_else(|| "Aucun modèle trouvé".into());

                            ToolCallResult::text(format!("Modèles disponibles :\n{models}"))
                        }
                    }
                }
            }
        }
    }
}
