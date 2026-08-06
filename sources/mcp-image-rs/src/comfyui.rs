// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
// ComfyUI expose une API REST sur http://127.0.0.1:8188 (par défaut).
//
// Flux d'appel :
//   1. POST /prompt          → soumet un workflow JSON, reçoit un prompt_id
//   2. GET  /history/{id}    → poll jusqu'à ce que le job soit terminé
//   3. GET  /view?filename=… → télécharge les bytes PNG de l'image
//
// ============================================================================

// ============================================================================
// comfyui.rs — Client HTTP pour l'API ComfyUI
// ============================================================================


use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Paramètres de génération exposés par l'outil MCP.
/// Les valeurs par défaut viennent du .env via `tools::Defaults` — aucune
/// valeur en dur ici pour éviter la duplication.
pub struct GenerateParams<'a> {
    pub prompt:          &'a str,
    pub negative_prompt: &'a str,
    pub width:           u32,
    pub height:          u32,
    pub steps:           u32,
    pub cfg_scale:       f32,
    pub seed:            i64,   // -1 = aléatoire
    pub model:           &'a str,
    pub sampler:         &'a str,
    pub scheduler:       &'a str,
}

/// Génère une image via ComfyUI et retourne les bytes PNG bruts.
pub async fn generate_image(
    client:   &Client,
    base_url: &str,
    params:   &GenerateParams<'_>,
) -> Result<Vec<u8>> {
    // 1. Construire le workflow ComfyUI (format "API JSON")
    let seed = if params.seed < 0 {
        rand_seed()
    } else {
        params.seed as u64
    };

    let workflow = build_workflow(params, seed);
    let client_id = uuid::Uuid::new_v4().to_string();

    // 2. Soumettre le workflow
    let submit_url = format!("{}/prompt", base_url);
    let body = json!({
        "prompt":    workflow,
        "client_id": client_id,
    });

    debug!("ComfyUI → POST {submit_url}");
    let resp = client
        .post(&submit_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let submit_resp: Value = resp.json().await?;
    let prompt_id = submit_resp["prompt_id"]
        .as_str()
        .ok_or_else(|| anyhow!("ComfyUI n'a pas retourné de prompt_id"))?
        .to_string();

    info!("ComfyUI : prompt_id = {prompt_id}");

    // 3. Poll /history/{prompt_id} jusqu'à completion
    let image_filename = poll_until_done(client, base_url, &prompt_id).await?;

    // 4. Télécharger l'image générée
    let image_bytes = download_image(client, base_url, &image_filename).await?;

    Ok(image_bytes)
}

// ---------------------------------------------------------------------------
// Construction du workflow ComfyUI (API JSON)
// ---------------------------------------------------------------------------
// Nœuds utilisés :
//   4  CheckpointLoaderSimple  — charge le modèle
//   5  EmptyLatentImage        — dimensions du latent
//   6  CLIPTextEncode          — encode le prompt positif
//   7  CLIPTextEncode          — encode le prompt négatif
//   3  KSampler                — sampling (débruitage)
//   8  VAEDecode               — décode le latent en image
//   9  SaveImage               — sauvegarde l'image (nécessaire pour /view)
// ---------------------------------------------------------------------------

fn build_workflow(p: &GenerateParams<'_>, seed: u64) -> Value {
    json!({
        "4": {
            "class_type": "CheckpointLoaderSimple",
            "inputs": {
                "ckpt_name": p.model
            }
        },
        "5": {
            "class_type": "EmptyLatentImage",
            "inputs": {
                "width":       p.width,
                "height":      p.height,
                "batch_size":  1
            }
        },
        "6": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": p.prompt,
                "clip": ["4", 1]
            }
        },
        "7": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": p.negative_prompt,
                "clip": ["4", 1]
            }
        },
        "3": {
            "class_type": "KSampler",
            "inputs": {
                "seed":            seed,
                "steps":           p.steps,
                "cfg":             p.cfg_scale,
                "sampler_name":    p.sampler,
                "scheduler":       p.scheduler,
                "denoise":         1.0,
                "model":           ["4", 0],
                "positive":        ["6", 0],
                "negative":        ["7", 0],
                "latent_image":    ["5", 0]
            }
        },
        "8": {
            "class_type": "VAEDecode",
            "inputs": {
                "samples": ["3", 0],
                "vae":     ["4", 2]
            }
        },
        "9": {
            "class_type": "SaveImage",
            "inputs": {
                "filename_prefix": "mcp_image",
                "images":          ["8", 0]
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Poll /history jusqu'à ce que le job soit terminé
// Retourne le nom du fichier image produit.
// ---------------------------------------------------------------------------

async fn poll_until_done(
    client:    &Client,
    base_url:  &str,
    prompt_id: &str,
) -> Result<String> {
    let history_url = format!("{}/history/{}", base_url, prompt_id);
    let max_attempts = 120; // 120 × 500 ms = 60 secondes max
    let poll_interval = Duration::from_millis(500);

    for attempt in 0..max_attempts {
        sleep(poll_interval).await;

        let resp = client
            .get(&history_url)
            .send()
            .await?;

        if resp.status() == 404 {
            debug!("ComfyUI poll #{attempt}: pas encore dans l'historique");
            continue;
        }

        let history: Value = resp.error_for_status()?.json().await?;

        let job = &history[prompt_id];
        if job.is_null() {
            debug!("ComfyUI poll #{attempt}: job absent de l'historique");
            continue;
        }

        let outputs = &job["outputs"];
        if let Some(node_out) = outputs.get("9") {
            if let Some(images) = node_out["images"].as_array() {
                if let Some(first) = images.first() {
                    let filename = first["filename"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Nom de fichier manquant dans l'historique"))?
                        .to_string();
                    info!("ComfyUI : image générée → {filename}");
                    return Ok(filename);
                }
            }
        }

        if job.get("status").and_then(|s| s["completed"].as_bool()) == Some(true) {
            warn!("ComfyUI : job terminé mais aucune image trouvée. Historique : {job}");
            return Err(anyhow!("ComfyUI a terminé le job sans produire d'image"));
        }
    }

    Err(anyhow!("Timeout : ComfyUI n'a pas terminé dans le délai imparti (60 s)"))
}

// ---------------------------------------------------------------------------
// Téléchargement de l'image via GET /view?filename=…
// ---------------------------------------------------------------------------

async fn download_image(
    client:    &Client,
    base_url:  &str,
    filename:  &str,
) -> Result<Vec<u8>> {
    let url = format!("{}/view?filename={}&type=output", base_url, filename);
    debug!("ComfyUI → GET {url}");

    let bytes = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    Ok(bytes.to_vec())
}

// ---------------------------------------------------------------------------
// Seed aléatoire simple
// ---------------------------------------------------------------------------

fn rand_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}
