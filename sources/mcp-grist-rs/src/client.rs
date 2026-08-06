// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// GristClient
// ---------------------------------------------------------------------------
#[derive(Clone)]
pub struct GristClient {
    pub api_key: String,
    pub base_url: String, // ex: "https://docs.getgrist.com/api"
    agent: Arc<ureq::Agent>,
}

impl GristClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        // base_url peut arriver avec ou sans "/api" — on normalise
        let base_url = base_url.trim_end_matches('/').to_string();
        let base_url = if base_url.ends_with("/api") {
            base_url
        } else {
            format!("{base_url}/api")
        };

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();

        Self { api_key, base_url, agent: Arc::new(agent) }
    }

    // -------------------------------------------------------------------------
    // Helpers HTTP synchrones (run via spawn_blocking depuis l'async)
    // -------------------------------------------------------------------------

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    pub fn get_sync(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);

        // Construire la query string manuellement
        let full_url = if params.is_empty() {
            url.clone()
        } else {
            let qs: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", urlenc(k), urlenc(v)))
                .collect::<Vec<_>>()
                .join("&");
            format!("{url}?{qs}")
        };

        let resp = self
            .agent
            .get(&full_url)
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/json")
            .call()
            .with_context(|| format!("GET {path}"))?;

        resp.into_json::<Value>().context("Réponse non-JSON")
    }

    pub fn post_sync(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .agent
            .post(&url)
            .set("Authorization", &self.auth_header())
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_json(body)
            .with_context(|| format!("POST {path}"))?;

        if resp.status() == 204 {
            return Ok(serde_json::json!({"status": "ok"}));
        }
        resp.into_json::<Value>().context("Réponse non-JSON")
    }

    pub fn patch_sync(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .agent
            .patch(&url)
            .set("Authorization", &self.auth_header())
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_json(body)
            .with_context(|| format!("PATCH {path}"))?;

        if resp.status() == 204 {
            return Ok(serde_json::json!({"status": "ok"}));
        }
        resp.into_json::<Value>().context("Réponse non-JSON")
    }

    pub fn delete_sync(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .agent
            .delete(&url)
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/json")
            .call()
            .with_context(|| format!("DELETE {path}"))?;

        if resp.status() == 204 {
            return Ok(serde_json::json!({"status": "ok"}));
        }
        resp.into_json::<Value>().context("Réponse non-JSON")
    }

    // -------------------------------------------------------------------------
    // Versions async — délèguent au thread pool
    // -------------------------------------------------------------------------

    pub async fn get(&self, path: &str, params: Vec<(String, String)>) -> Result<Value> {
        let client = self.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let p: Vec<(&str, &str)> = params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            client.get_sync(&path, &p)
        })
        .await
        .context("spawn_blocking paniqué")?
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let client = self.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || client.post_sync(&path, &body))
            .await
            .context("spawn_blocking paniqué")?
    }

    pub async fn patch(&self, path: &str, body: Value) -> Result<Value> {
        let client = self.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || client.patch_sync(&path, &body))
            .await
            .context("spawn_blocking paniqué")?
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        let client = self.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || client.delete_sync(&path))
            .await
            .context("spawn_blocking paniqué")?
    }
}

// ---------------------------------------------------------------------------
// Encodage URL minimal
// ---------------------------------------------------------------------------
fn urlenc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
