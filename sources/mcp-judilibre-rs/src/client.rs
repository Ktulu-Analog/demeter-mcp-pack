// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

pub const BASE_URL: &str = "https://api.piste.gouv.fr/cassation/judilibre/v1.0";
const OAUTH_URL: &str = "https://oauth.piste.gouv.fr/api/oauth/token";

// ---------------------------------------------------------------------------
// Token cache
// ---------------------------------------------------------------------------
struct TokenCache {
    token: Option<String>,
    expires_at: Option<Instant>,
}

impl TokenCache {
    fn new() -> Self { Self { token: None, expires_at: None } }

    fn get_valid(&self) -> Option<String> {
        if let (Some(tok), Some(exp)) = (&self.token, self.expires_at) {
            if Instant::now() + Duration::from_secs(60) < exp {
                return Some(tok.clone());
            }
        }
        None
    }

    fn set(&mut self, token: String, expires_in_secs: u64) {
        self.expires_at = Some(Instant::now() + Duration::from_secs(expires_in_secs));
        self.token = Some(token);
    }

    fn invalidate(&mut self) { self.token = None; self.expires_at = None; }
}

// ---------------------------------------------------------------------------
// JudiLibreClient
// ---------------------------------------------------------------------------
#[derive(Clone)]
pub struct JudiLibreClient {
    cache: Arc<Mutex<TokenCache>>,
    client_id: String,
    client_secret: String,
    agent: Arc<ureq::Agent>,
}

impl JudiLibreClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();
        Self {
            cache: Arc::new(Mutex::new(TokenCache::new())),
            client_id,
            client_secret,
            agent: Arc::new(agent),
        }
    }

    fn get_token_sync(&self) -> Result<String> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(tok) = cache.get_valid() {
                return Ok(tok);
            }
        }

        if self.client_id.is_empty() || self.client_secret.is_empty() {
            anyhow::bail!(
                "Identifiants JUDILIBRE manquants. \
                Définissez JUDILIBRE_CLIENT_ID et JUDILIBRE_CLIENT_SECRET \
                (obtenus sur https://piste.gouv.fr)."
            );
        }

        let resp = self.agent
            .post(OAUTH_URL)
            .send_form(&[
                ("grant_type",    "client_credentials"),
                ("client_id",     &self.client_id),
                ("client_secret", &self.client_secret),
                ("scope",         "openid"),
            ])
            .context("Erreur réseau lors de l'obtention du token OAuth2")?;

        let body: Value = resp.into_json()
            .context("Réponse OAuth2 non-JSON")?;
        let token = body["access_token"]
            .as_str()
            .context("Champ access_token absent dans la réponse OAuth2")?
            .to_owned();
        let expires_in = body["expires_in"].as_u64().unwrap_or(3600);

        self.cache.lock().unwrap().set(token.clone(), expires_in);
        Ok(token)
    }

    fn request_sync(
        &self,
        path: &str,
        params: &HashMap<String, Vec<String>>,
    ) -> Result<Value> {
        let url = format!("{BASE_URL}{path}");

        for attempt in 0u32..3 {
            let token = self.get_token_sync()?;

            // Ajouter les query params (plusieurs valeurs par clé pour les listes)
            let mut query_pairs: Vec<(String, String)> = Vec::new();
            for (k, vs) in params {
                for v in vs {
                    query_pairs.push((k.clone(), v.clone()));
                }
            }

            // ureq ne permet pas d'ajouter des query params dynamiquement après build
            // On construit l'URL manuellement
            let full_url = if query_pairs.is_empty() {
                url.clone()
            } else {
                let qs = query_pairs.iter()
                    .map(|(k, v)| format!("{}={}", urlenc(k), urlenc(v)))
                    .collect::<Vec<_>>()
                    .join("&");
                format!("{url}?{qs}")
            };

            let resp_result = self.agent.get(&full_url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Accept", "application/json")
                .call();

            match resp_result {
                Ok(resp) => {
                    return resp.into_json::<Value>()
                        .context("Réponse API non-JSON");
                }
                Err(ureq::Error::Status(401, _)) if attempt == 0 => {
                    self.cache.lock().unwrap().invalidate();
                    continue;
                }
                Err(ureq::Error::Status(429, _)) => {
                    anyhow::bail!("Limite de débit API JUDILIBRE atteinte (429)");
                }
                Err(ureq::Error::Status(code, _)) => {
                    if attempt == 2 || code < 500 {
                        anyhow::bail!("Erreur HTTP {code} sur {path}");
                    }
                    std::thread::sleep(Duration::from_secs(2u64.pow(attempt)));
                }
                Err(e) => {
                    if attempt == 2 {
                        anyhow::bail!("Erreur réseau : {e}");
                    }
                    std::thread::sleep(Duration::from_secs(2u64.pow(attempt)));
                }
            }
        }
        anyhow::bail!("Échec après 3 tentatives sur {path}");
    }

    /// Version async — délègue au thread pool pour ne pas bloquer le runtime
    pub async fn request(
        &self,
        path: &str,
        params: &HashMap<String, Vec<String>>,
    ) -> Result<Value> {
        let client = self.clone();
        let path = path.to_string();
        let params = params.clone();
        tokio::task::spawn_blocking(move || client.request_sync(&path, &params))
            .await
            .context("spawn_blocking paniqué")?
    }
}

// ---------------------------------------------------------------------------
// Encodage URL minimal (percent-encoding des caractères spéciaux)
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
