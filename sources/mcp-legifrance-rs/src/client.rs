// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const BASE_URL: &str = "https://api.piste.gouv.fr/dila/legifrance/lf-engine-app";
const OAUTH_URL: &str = "https://oauth.piste.gouv.fr/api/oauth/token";
const BASE_URL_SANDBOX: &str = "https://sandbox-api.piste.gouv.fr/dila/legifrance/lf-engine-app";
const OAUTH_URL_SANDBOX: &str = "https://sandbox-oauth.piste.gouv.fr/api/oauth/token";

/// URL de base effective : production par défaut, ou sandbox si
/// LEGIFRANCE_ENV=sandbox est positionné. Utile pour vérifier rapidement,
/// sans recompiler, si une erreur 403 vient d'un identifiant/jeton
/// provisionné pour le mauvais environnement PISTE (cause très fréquente
/// d'après la FAQ officielle Légifrance : "j'essaie de consommer l'API de
/// production avec un token obtenu en sandbox").
///
/// Surcharge avancée (proxy, tests) : LEGIFRANCE_API_URL et
/// LEGIFRANCE_OAUTH_URL remplacent entièrement les URL ci-dessus.
pub fn base_url() -> String {
    if let Ok(u) = std::env::var("LEGIFRANCE_API_URL") {
        if !u.trim().is_empty() {
            return u.trim().trim_end_matches('/').to_string();
        }
    }
    if std::env::var("LEGIFRANCE_ENV").as_deref() == Ok("sandbox") {
        BASE_URL_SANDBOX.to_string()
    } else {
        BASE_URL.to_string()
    }
}

pub fn oauth_url() -> String {
    if let Ok(u) = std::env::var("LEGIFRANCE_OAUTH_URL") {
        if !u.trim().is_empty() {
            return u.trim().to_string();
        }
    }
    if std::env::var("LEGIFRANCE_ENV").as_deref() == Ok("sandbox") {
        OAUTH_URL_SANDBOX.to_string()
    } else {
        OAUTH_URL.to_string()
    }
}

// Mapping noms courts → (ID LEGITEXT, intitulé officiel).
// L'intitulé est la valeur attendue par les facettes NOM_CODE / TEXT_NOM_CODE
// de POST /search (cf. exemples-d-utilisation-de-l-api.docx §7).
// Pour tout autre code, resolve_code_dyn (tools.rs) interroge /list/code.
pub const CODES: &[(&str, &str, &str)] = &[
    ("civil",                    "LEGITEXT000006070721", "Code civil"),
    ("penal",                    "LEGITEXT000006070719", "Code pénal"),
    ("travail",                  "LEGITEXT000006072050", "Code du travail"),
    ("commerce",                 "LEGITEXT000005634379", "Code de commerce"),
    ("consommation",             "LEGITEXT000006069565", "Code de la consommation"),
    ("procedure_civile",         "LEGITEXT000006070716", "Code de procédure civile"),
    ("procedure_penale",         "LEGITEXT000006071154", "Code de procédure pénale"),
    ("sante_publique",           "LEGITEXT000006072665", "Code de la santé publique"),
    ("education",                "LEGITEXT000006071191", "Code de l'éducation"),
    ("environnement",            "LEGITEXT000006074220", "Code de l'environnement"),
    ("securite_sociale",         "LEGITEXT000006073189", "Code de la sécurité sociale"),
    ("general_collectivites",    "LEGITEXT000006070633", "Code général des collectivités territoriales"),
    ("urbanisme",                "LEGITEXT000006074075", "Code de l'urbanisme"),
    ("propriete_intellectuelle", "LEGITEXT000006069414", "Code de la propriété intellectuelle"),
    ("assurances",               "LEGITEXT000006073984", "Code des assurances"),
    ("monetaire_financier",      "LEGITEXT000006072026", "Code monétaire et financier"),
];

#[allow(dead_code)]
pub fn code_ids() -> HashMap<&'static str, &'static str> {
    CODES.iter().map(|(k, id, _)| (*k, *id)).collect()
}

fn norm_key(code: &str) -> String {
    code.trim().to_lowercase().replace([' ', '-'], "_")
}

/// Intitulé officiel d'un code à partir de son nom court ("civil"), de son ID
/// LEGITEXT ou de son intitulé déjà complet ("Code civil").
pub fn code_title(code: &str) -> Option<&'static str> {
    let key = norm_key(code);
    CODES
        .iter()
        .find(|(k, id, title)| *k == key || *id == code.trim() || title.to_lowercase() == code.trim().to_lowercase())
        .map(|(_, _, title)| *title)
}

/// Résolution statique (sans réseau) : nom court ou ID LEGITEXT → ID LEGITEXT.
pub fn resolve_code(code: &str) -> Result<String> {
    let code = code.trim();
    if code.starts_with("LEGITEXT") {
        return Ok(code.to_string());
    }
    let key = norm_key(code);
    if let Some((_, id, _)) = CODES
        .iter()
        .find(|(k, _, title)| *k == key || title.to_lowercase() == code.to_lowercase())
    {
        return Ok((*id).to_string());
    }
    let known: Vec<&str> = CODES.iter().map(|(k, _, _)| *k).collect();
    anyhow::bail!(
        "Code non reconnu : '{}'. Noms courts valides : {}. Sinon utilisez un ID LEGITEXT \
         (cherchez-le avec legifrance_lister_codes).",
        code,
        known.join(", ")
    )
}

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
// LegifranceClient
// ---------------------------------------------------------------------------
#[derive(Clone)]
pub struct LegifranceClient {
    cache: Arc<Mutex<TokenCache>>,
    client_id: String,
    client_secret: String,
    agent: Arc<ureq::Agent>,
}

impl LegifranceClient {
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
                "Identifiants Légifrance manquants. \
                Définissez LEGIFRANCE_CLIENT_ID et LEGIFRANCE_CLIENT_SECRET \
                (obtenus sur https://piste.gouv.fr)."
            );
        }
        let resp = match self.agent
            .post(&oauth_url())
            .send_form(&[
                ("grant_type",    "client_credentials"),
                ("client_id",     &self.client_id),
                ("client_secret", &self.client_secret),
                ("scope",         "openid"),
            ]) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let detail: String = r.into_string().unwrap_or_default().chars().take(300).collect();
                anyhow::bail!(
                    "Authentification PISTE refusée (HTTP {code}) sur {} : {detail}. \
                     Vérifiez LEGIFRANCE_CLIENT_ID / LEGIFRANCE_CLIENT_SECRET et que \
                     l'environnement (LEGIFRANCE_ENV) correspond à celui de votre application PISTE.",
                    oauth_url()
                );
            }
            Err(e) => return Err(anyhow::anyhow!("Erreur réseau lors de l'obtention du token OAuth2 : {e}")),
        };
        let body: Value = resp.into_json().context("Réponse OAuth2 non-JSON")?;
        let token = body["access_token"]
            .as_str()
            .context("Champ access_token absent dans la réponse OAuth2")?
            .to_owned();
        let expires_in = body["expires_in"].as_u64().unwrap_or(3600);
        self.cache.lock().unwrap().set(token.clone(), expires_in);
        Ok(token)
    }

    /// Appel POST JSON vers l'API Légifrance (méthode principale).
    fn request_sync(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{path}", base_url());
        for attempt in 0u32..3 {
            let token = self.get_token_sync()?;
            let resp_result = self.agent
                .post(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Accept", "application/json")
                .set("Content-Type", "application/json")
                .send_json(body.clone());
            match resp_result {
                Ok(resp) => {
                    return resp.into_json::<Value>().context("Réponse API non-JSON");
                }
                Err(ureq::Error::Status(401, _)) if attempt == 0 => {
                    self.cache.lock().unwrap().invalidate();
                    continue;
                }
                Err(ureq::Error::Status(429, _)) => {
                    anyhow::bail!("Limite de débit API Légifrance atteinte (429)");
                }
                Err(ureq::Error::Status(code, resp)) => {
                    if attempt == 2 || code < 500 {
                        // On récupère le corps de la réponse d'erreur : l'API PISTE
                        // y met souvent le détail exact (champ invalide, requis
                        // manquant, etc.) — sans ça, on ne renvoie qu'un code HTTP
                        // muet, inexploitable aussi bien pour l'utilisateur que
                        // pour le modèle qui essaie de corriger son appel suivant.
                        let detail = resp
                            .into_string()
                            .unwrap_or_default()
                            .chars()
                            .take(500)
                            .collect::<String>();
                        if detail.trim().is_empty() {
                            anyhow::bail!("Erreur HTTP {code} sur {path}");
                        }
                        anyhow::bail!("Erreur HTTP {code} sur {path} : {detail}");
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

    /// Appel GET (quelques endpoints misc utilisent GET).
    fn get_sync(&self, path: &str) -> Result<Value> {
        let url = format!("{}{path}", base_url());
        for attempt in 0u32..3 {
            let token = self.get_token_sync()?;
            let resp_result = self.agent
                .get(&url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Accept", "application/json")
                .call();
            match resp_result {
                Ok(resp) => {
                    let ct = resp.content_type().to_string();
                    if ct.contains("application/json") {
                        return resp.into_json::<Value>().context("Réponse API non-JSON");
                    }
                    let text = resp.into_string().unwrap_or_default();
                    return Ok(Value::String(text));
                }
                Err(ureq::Error::Status(401, _)) if attempt == 0 => {
                    self.cache.lock().unwrap().invalidate();
                    continue;
                }
                Err(ureq::Error::Status(code, resp)) => {
                    if attempt == 2 || code < 500 {
                        let detail = resp
                            .into_string()
                            .unwrap_or_default()
                            .chars()
                            .take(500)
                            .collect::<String>();
                        if detail.trim().is_empty() {
                            anyhow::bail!("Erreur HTTP {code} sur {path}");
                        }
                        anyhow::bail!("Erreur HTTP {code} sur {path} : {detail}");
                    }
                    std::thread::sleep(Duration::from_secs(2u64.pow(attempt)));
                }
                Err(e) => {
                    if attempt == 2 { anyhow::bail!("Erreur réseau : {e}"); }
                    std::thread::sleep(Duration::from_secs(2u64.pow(attempt)));
                }
            }
        }
        anyhow::bail!("Échec après 3 tentatives sur {path}");
    }

    /// Version async POST — délègue au thread pool
    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let client = self.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || client.request_sync(&path, &body))
            .await
            .context("spawn_blocking paniqué")?
    }

    /// Version async GET
    pub async fn get(&self, path: &str) -> Result<Value> {
        let client = self.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || client.get_sync(&path))
            .await
            .context("spawn_blocking paniqué")?
    }
}
