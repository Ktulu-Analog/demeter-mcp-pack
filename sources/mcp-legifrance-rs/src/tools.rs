// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================

use crate::client::{LegifranceClient, resolve_code};
use crate::fmt::{fmt_article, fmt_search, fmt_toc, fmt_date, strip_html, today};
use crate::mcp::{ToolCallResult, ToolDef};
use anyhow::Result;
use serde_json::{json, Value};

// ============================================================================
// Registre public
// ============================================================================

pub fn all_tools() -> Vec<ToolDef> {
    let mut v = Vec::new();
    // ── Recherche générale ──
    v.push(tool_rechercher());
    v.push(tool_suggerer());
    v.push(tool_suggerer_acco());
    v.push(tool_suggerer_pdc());
    // ── Codes ──
    v.push(tool_lister_codes());
    v.push(tool_consulter_code());
    v.push(tool_code_complet());
    v.push(tool_code_par_ancien_id());
    v.push(tool_section_par_cid());
    v.push(tool_obtenir_article());
    v.push(tool_article_par_numero());
    v.push(tool_article_par_eli());
    v.push(tool_articles_meme_numero());
    v.push(tool_versions_article());
    v.push(tool_versions_element());
    v.push(tool_version_canonique_article());
    v.push(tool_version_canonique());
    v.push(tool_version_proche());
    v.push(tool_a_des_versions());
    v.push(tool_liens_concordance());
    v.push(tool_liens_relatifs());
    v.push(tool_liens_service_public());
    v.push(tool_a_liens_service_public());
    // ── Lois & règlements (LODA/LEGI) ──
    v.push(tool_loi_decret());
    v.push(tool_legi_part());
    v.push(tool_lister_loda());
    v.push(tool_historique_texte());
    v.push(tool_eli_alias_texte());
    // ── Journal Officiel (JORF) ──
    v.push(tool_jorf());
    v.push(tool_jorf_part());
    v.push(tool_jo_par_nor());
    v.push(tool_derniers_jo());
    v.push(tool_sommaire_jorf());
    v.push(tool_tables_annuelles());
    v.push(tool_dates_sans_jo());
    v.push(tool_annees_sans_table());
    // ── Conventions collectives (KALI) ──
    v.push(tool_conventions());
    v.push(tool_convention_par_idcc());
    v.push(tool_convention_cont());
    v.push(tool_convention_texte());
    v.push(tool_convention_article());
    v.push(tool_convention_section());
    // ── Jurisprudence ──
    v.push(tool_jurisprudence());
    v.push(tool_jurisprudence_plan_classement());
    v.push(tool_jurisprudence_ancien_id());
    // ── CNIL ──
    v.push(tool_cnil());
    v.push(tool_cnil_ancien_id());
    // ── ACCO (accords d'entreprise) ──
    v.push(tool_acco());
    // ── Circulaires ──
    v.push(tool_circulaire());
    // ── Dossiers législatifs ──
    v.push(tool_dossier_legislatif());
    v.push(tool_lister_dossiers_legislatifs());
    v.push(tool_lister_legislatures());
    // ── Débats & questions parlementaires ──
    v.push(tool_debat());
    v.push(tool_lister_debats_parlementaires());
    v.push(tool_lister_questions_parlementaires());
    // ── BOCC / BODMR ──
    v.push(tool_lister_bocc());
    v.push(tool_lister_bocc_textes());
    v.push(tool_lister_boccs_et_textes());
    v.push(tool_bocc_pdf_metadata());
    v.push(tool_lister_bodmr());
    // ── Documents administratifs ──
    v.push(tool_lister_docs_admins());
    // ── Utilitaires ──
    v.push(tool_commit_id());
    v
}

// ============================================================================
// Dispatch
// ============================================================================

pub async fn dispatch(name: &str, args: &Value, client: &LegifranceClient) -> ToolCallResult {
    let r: Result<String> = match name {
        // Recherche
        "legifrance_rechercher"                    => do_search_all(args, client).await,
        "legifrance_suggerer" => do_search_suggest(args, client).await,
        "legifrance_suggerer_acco"                 => do_suggerer_acco(args, client).await,
        "legifrance_suggerer_pdc"                  => do_suggerer_pdc(args, client).await,
        // Codes & articles
        "legifrance_lister_codes" => do_code_list(args, client).await,
        "legifrance_consulter_code" => do_code_toc(args, client).await,
        "legifrance_code_complet"                  => do_code_complet(args, client).await,
        "legifrance_code_par_ancien_id"            => do_code_par_ancien_id(args, client).await,
        "legifrance_section_par_cid" => do_code_section(args, client).await,
        "legifrance_obtenir_article" => do_code_article(args, client).await,
        "legifrance_article_par_numero" => do_code_article_by_num(args, client).await,
        "legifrance_article_par_eli"               => do_article_par_eli(args, client).await,
        "legifrance_articles_meme_numero"          => do_articles_meme_numero(args, client).await,
        "legifrance_versions_article" => do_code_article_versions(args, client).await,
        "legifrance_versions_element"              => do_versions_element(args, client).await,
        "legifrance_version_canonique_article"     => do_version_canonique_article(args, client).await,
        "legifrance_version_canonique"             => do_version_canonique(args, client).await,
        "legifrance_version_proche"                => do_version_proche(args, client).await,
        "legifrance_a_des_versions"                => do_a_des_versions(args, client).await,
        "legifrance_liens_concordance" => do_code_article_liens(args, client).await,
        "legifrance_liens_relatifs"                => do_liens_relatifs(args, client).await,
        "legifrance_liens_service_public"          => do_liens_service_public(args, client).await,
        "legifrance_a_liens_service_public"        => do_a_liens_service_public(args, client).await,
        // LODA / LEGI
        "legifrance_loi_decret"                    => do_legi_text(args, client).await,
        "legifrance_legi_part" => do_legi_section(args, client).await,
        "legifrance_lister_loda"                   => do_lister_loda(args, client).await,
        "legifrance_historique_texte"              => do_historique_texte(args, client).await,
        "legifrance_eli_alias_texte"               => do_eli_alias_texte(args, client).await,
        // JORF
        "legifrance_jorf" => do_jorf_text(args, client).await,
        "legifrance_jorf_part" => do_jorf_section(args, client).await,
        "legifrance_jo_par_nor"                    => do_jorf_article(args, client).await,
        "legifrance_derniers_jo" => do_jorf_jo_list(args, client).await,
        "legifrance_sommaire_jorf" => do_consult_jorf_date(args, client).await,
        "legifrance_tables_annuelles"              => do_tables_annuelles(args, client).await,
        "legifrance_dates_sans_jo"                 => do_dates_sans_jo(args, client).await,
        "legifrance_annees_sans_table"             => do_annees_sans_table(client).await,
        // KALI
        "legifrance_conventions" => do_kali_list(args, client).await,
        "legifrance_convention_par_idcc"           => do_convention_par_idcc(args, client).await,
        "legifrance_convention_cont" => do_kali_text(args, client).await,
        "legifrance_convention_texte"              => do_convention_texte(args, client).await,
        "legifrance_convention_article" => do_kali_article(args, client).await,
        "legifrance_convention_section" => do_kali_section(args, client).await,
        // Jurisprudence
        "legifrance_jurisprudence" => do_jade_text(args, client).await,
        "legifrance_jurisprudence_plan_classement" => do_jade_search(args, client).await,
        "legifrance_jurisprudence_ancien_id"       => do_jurisprudence_ancien_id(args, client).await,
        // CNIL
        "legifrance_cnil" => do_cnil_text(args, client).await,
        "legifrance_cnil_ancien_id"                => do_cnil_ancien_id(args, client).await,
        // ACCO
        "legifrance_acco" => do_acco_text(args, client).await,
        // Circulaires
        "legifrance_circulaire"                    => do_circ_text(args, client).await,
        // Dossiers
        "legifrance_dossier_legislatif" => do_dossier_text(args, client).await,
        "legifrance_lister_dossiers_legislatifs" => do_lister_dossiers_legislatifs(args, client).await,
        "legifrance_lister_legislatures"           => do_lister_legislatures(client).await,
        // Débats / Questions
        "legifrance_debat"                         => do_debat(args, client).await,
        "legifrance_lister_debats_parlementaires"  => do_lister_debats_parlementaires(args, client).await,
        "legifrance_lister_questions_parlementaires" => do_lister_questions_parlementaires(args, client).await,
        // BOCC / BODMR
        "legifrance_lister_bocc"                   => do_lister_bocc(args, client).await,
        "legifrance_lister_bocc_textes"            => do_lister_bocc_textes(args, client).await,
        "legifrance_lister_boccs_et_textes"        => do_lister_boccs_et_textes(args, client).await,
        "legifrance_bocc_pdf_metadata"             => do_bocc_pdf_metadata(args, client).await,
        "legifrance_lister_bodmr"                  => do_lister_bodmr(args, client).await,
        // Docs admins
        "legifrance_lister_docs_admins"            => do_lister_docs_admins(args, client).await,
        // Utilitaires
        "legifrance_commit_id"                     => do_commit_id(client).await,
        other => Err(anyhow::anyhow!("Outil inconnu : {other}")),
    };
    match r {
        Ok(text) => ToolCallResult::text(text),
        Err(e)   => ToolCallResult::error(format!("❌ Erreur : {e}")),
    }
}

// ============================================================================
// Helpers génériques
// ============================================================================

fn str_arg<'a>(args: &'a Value, key: &str) -> &'a str {
    args[key].as_str().unwrap_or("")
}
fn int_arg(args: &Value, key: &str, default: i64) -> i64 {
    args[key].as_i64().unwrap_or(default)
}
fn date_arg(args: &Value, key: &str, default: &str) -> String {
    let s = args[key].as_str().unwrap_or(default);
    if s.is_empty() { default.to_string() } else { s.to_string() }
}
fn require_str<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    let s = args[key].as_str().unwrap_or("");
    if s.is_empty() { anyhow::bail!("Paramètre '{key}' obligatoire.") }
    Ok(s)
}

fn search_schema(extra: Option<Value>) -> Value {
    let mut props = json!({
        "query":      { "type": "string",  "description": "Termes de recherche" },
        "nb_resultats": { "type": "integer", "minimum": 1, "maximum": 20, "default": 5 },
        "fond":       { "type": "string",  "description": "ALL, CODE_DATE, LODA_DATE, JORF, KALI, CIRC, JADE, DOSSIER_LEGISLATIF (défaut: ALL)" },
        "date_debut": { "type": "string",  "description": "Date début YYYY-MM-DD" },
        "date_fin":   { "type": "string",  "description": "Date fin YYYY-MM-DD" },
        "tri":        { "type": "string",  "description": "PERTINENCE, DATE_ASC, DATE_DESC" }
    });
    if let Some(extra_props) = extra.and_then(|v| v.as_object().cloned()) {
        for (k, v) in extra_props { props[k] = v; }
    }
    json!({ "type": "object", "required": ["query"], "properties": props })
}

/// Facette de date correcte selon le fond (cf. doc officielle PISTE) —
/// envoyer la mauvaise facette de date pour un fond donné provoque une
/// erreur côté API plutôt qu'un filtre silencieusement ignoré.
fn date_facette(fond: &str) -> Option<&'static str> {
    match fond {
        "CODE_ETAT"                        => None,
        "CODE_DATE"                        => Some("DATE_VERSION"),
        "LODA_ETAT"                        => Some("TEXT_DATE_SIGNATURE"),
        "CONSTIT" | "CETAT" | "JURI" | "JUFI" => Some("DATE_DECISION"),
        "CNIL"                              => Some("DATE_DELIB"),
        "BOCC"                              => Some("INTERVAL_PUBLICATION"),
        "DEBAT_PARLEMENTAIRE"               => Some("DATE_PUBLICATION"),
        "QUESTION_ECRITE"                   => Some("DATE_PUBLICATION"),
        // JORF, LODA_DATE, CIRC, KALI, ACCO, et tout fond non listé.
        _ => Some("DATE_SIGNATURE"),
    }
}

/// Facette de nature correcte selon le fond — d'après la doc officielle,
/// seuls JORF, LODA_DATE et LODA_ETAT exposent une facette "nature" simple ;
/// CODE, CETAT, JURI, JUFI, KALI et ACCO n'en ont pas (l'envoyer y provoque
/// une erreur 500 côté API).
fn nature_facette(fond: &str) -> Option<&'static str> {
    match fond {
        "JORF"      => Some("NATURE"),
        "LODA_DATE" => Some("NATURE"),
        "LODA_ETAT" => Some("TEXT_NATURE"),
        _ => None,
    }
}

fn build_search_body(args: &Value, fond: &str) -> Value {
    let fond_val = {
        let f = str_arg(args, "fond");
        if f.is_empty() { fond } else { f }
    };
    let mut body = json!({
        "recherche": {
            "champs": [{
                "typeChamp": "ALL",
                "criteres": [{ "typeRecherche": "UN_DES_MOTS", "valeur": str_arg(args, "query"), "operateur": "ET" }],
                "operateur": "ET"
            }],
            "operateur": "ET",
            "pageNumber": int_arg(args, "page", 1).max(1),
            "pageSize":   int_arg(args, "nb_resultats", 5).clamp(1, 100),
            "sort":       str_arg(args, "tri"),
            "typePagination": "DEFAUT",
            "fromAdvancedRecherche": false,
            "filtres": []
        },
        "fond": fond_val
    });
    let dd = str_arg(args, "date_debut");
    let df = str_arg(args, "date_fin");
    // CORRECTION vs référence : la facette de date dépend du fond — utiliser
    // systématiquement "DATE_VERSION" fait échouer la recherche (ou pire,
    // 500) pour la plupart des fonds autres que CODE_DATE.
    if (!dd.is_empty() || !df.is_empty()) && fond_val != "ALL" {
        if let Some(facette) = date_facette(fond_val) {
            let filtres = body["recherche"]["filtres"].as_array_mut().unwrap();
            filtres.push(json!({
                "facette": facette,
                "dates": {
                    "start": if dd.is_empty() { Value::Null } else { json!(dd) },
                    "end":   if df.is_empty() { Value::Null } else { json!(df) }
                }
            }));
        }
    }
    if let Some(code) = args["code"].as_str() {
        if let Ok(code_id) = resolve_code(code) {
            body["recherche"]["filtres"].as_array_mut().unwrap()
                .push(json!({ "facette": "CODE_ID", "valeur": code_id }));
        }
    }
    if let Some(nature) = args["nature"].as_str() {
        // CORRECTION vs référence : n'envoyer la facette NATURE que pour les
        // fonds qui l'exposent réellement (JORF, LODA_DATE, LODA_ETAT) —
        // sinon 500 côté API pour CODE/CETAT/JURI/JUFI/KALI/ACCO.
        if !nature.is_empty() {
            if let Some(facette) = nature_facette(fond_val) {
                body["recherche"]["filtres"].as_array_mut().unwrap()
                    .push(json!({ "facette": facette, "valeur": nature }));
            }
        }
    }
    body
}

// ============================================================================
// ── RECHERCHE & SUGGESTIONS ──
// ============================================================================

fn tool_rechercher() -> ToolDef {
    ToolDef {
        name: "legifrance_rechercher".into(),
        description: "Recherche de textes juridiques français dans Légifrance. \
            Interroge codes, lois, décrets, arrêtés, circulaires, jurisprudence, \
            conventions collectives et Journal Officiel.".into(),
        input_schema: search_schema(None),
    }
}
fn tool_suggerer() -> ToolDef {
    ToolDef {
        name: "legifrance_suggerer".into(),
        description: "Suggestions et autocomplétion pour identifier un texte juridique. \
            Utile avant une recherche ou consultation quand on connaît le début du nom.".into(),
        input_schema: json!({ "type": "object", "required": ["query"], "properties": { "query": { "type": "string", "description": "Début du texte à compléter" }, "fond": { "type": "string", "description": "Fonds documentaire (optionnel)" } } }),
    }
}
fn tool_suggerer_acco() -> ToolDef {
    ToolDef {
        name: "legifrance_suggerer_acco".into(),
        description: "Suggestions de SIRET et raisons sociales pour les accords d'entreprise. \
            Utile pour trouver une entreprise avant de consulter ses accords ACCO.".into(),
        input_schema: json!({
            "type": "object", "required": ["texte"],
            "properties": {
                "texte": { "type": "string",
                    "description": "Début de la raison sociale ou numéro SIRET (ex: 'Renault', 'Airbus', '351')" }
            }
        }),
    }
}
async fn do_suggerer_acco(args: &Value, client: &LegifranceClient) -> Result<String> {
    let texte = require_str(args, "texte")?;
    let body  = json!({ "searchText": texte });
    let data  = client.post("/suggest/acco", body).await?;
    let results = data["results"].as_object()
        .map(|o| o.values().flat_map(|v| v.as_array().cloned().unwrap_or_default()).collect::<Vec<_>>())
        .or_else(|| data["results"].as_array().cloned())
        .unwrap_or_default();
    if results.is_empty() { return Ok(format!("Aucune suggestion entreprise pour : « {texte} »")); }
    let mut lines = vec![format!("**Entreprises suggérées pour « {texte} »**\n")];
    for r in results.iter().take(20) {
        let rs   = r["raisonSociale"].as_str().or_else(|| r["rs"].as_str())
                    .or_else(|| r["title"].as_str()).unwrap_or("");
        let siret = r["siret"].as_str().unwrap_or("");
        let id    = r["id"].as_str().unwrap_or("");
        let mut line = format!("- {rs}");
        if !siret.is_empty() { line.push_str(&format!(" — SIRET {siret}")); }
        if !id.is_empty()    { line.push_str(&format!(" — `{id}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn tool_suggerer_pdc() -> ToolDef {
    ToolDef {
        name: "legifrance_suggerer_pdc".into(),
        description: "Suggestions pour le plan de classement des débats parlementaires.".into(),
        input_schema: json!({
            "type": "object", "required": ["texte"],
            "properties": {
                "texte": { "type": "string", "description": "Début du terme à compléter" }
            }
        }),
    }
}
async fn do_suggerer_pdc(args: &Value, client: &LegifranceClient) -> Result<String> {
    let texte = require_str(args, "texte")?;
    let body  = json!({ "searchText": texte });
    let data  = client.post("/suggest/pdc", body).await?;
    let results = data["results"].as_object()
        .map(|o| o.values().flat_map(|v| v.as_array().cloned().unwrap_or_default()).collect::<Vec<_>>())
        .or_else(|| data["results"].as_array().cloned())
        .unwrap_or_default();
    if results.is_empty() { return Ok(format!("Aucune suggestion PDC pour : « {texte} »")); }
    let mut lines = vec![format!("**Suggestions PDC pour « {texte} »**\n")];
    for r in results.iter().take(20) {
        let t   = r["title"].as_str().or_else(|| r["titre"].as_str()).unwrap_or("");
        let id  = r["id"].as_str().unwrap_or("");
        lines.push(format!("- {t}{}", if id.is_empty() { String::new() } else { format!(" — `{id}`") }));
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── CODES & ARTICLES ──
// ============================================================================

fn tool_lister_codes() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_codes".into(),
        description: "Liste tous les codes juridiques disponibles dans Légifrance avec leurs identifiants.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "filtre":               { "type": "string",  "description": "Filtre par nom (ex: 'code de la route')" },
                "en_vigueur_seulement": { "type": "boolean", "description": "Ne retourner que les codes en vigueur (défaut: true)", "default": true }
            }
        }),
    }
}
fn tool_consulter_code() -> ToolDef {
    ToolDef {
        name: "legifrance_consulter_code".into(),
        description: "Consulte la table des matières structurée d'un code juridique français. \
            Retourne la hiérarchie des parties, livres, titres et articles avec leurs identifiants.".into(),
        input_schema: json!({ "type": "object", "required": ["code"], "properties": { "code": { "type": "string", "description": "Nom du code ('civil', 'travail', 'penal', etc.) ou ID LEGITEXT..." }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD (défaut: aujourd'hui)" } } }),
    }
}
fn tool_code_complet() -> ToolDef {
    ToolDef {
        name: "legifrance_code_complet".into(),
        description: "Récupère le contenu complet d'un code juridique avec tous ses articles. \
            Attention : réponse potentiellement volumineuse. Préférez legifrance_consulter_code \
            puis legifrance_obtenir_article pour consulter des articles spécifiques.".into(),
        input_schema: json!({
            "type": "object", "required": ["code"],
            "properties": {
                "code": { "type": "string", "description": "Nom ou ID LEGITEXT du code" },
                "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" }
            }
        }),
    }
}
async fn do_code_complet(args: &Value, client: &LegifranceClient) -> Result<String> {
    let code    = require_str(args, "code")?;
    let code_id = resolve_code(code)?;
    let date    = date_arg(args, "date", &today());
    let body    = json!({ "textId": code_id, "date": date });
    let data    = client.post("/consult/legi/tableMatieres", body).await?;
    Ok(fmt_toc(&data, code))
}

fn tool_code_par_ancien_id() -> ToolDef {
    ToolDef {
        name: "legifrance_code_par_ancien_id".into(),
        description: "Consulte un code par son ancien identifiant Légifrance. \
            Utile pour résoudre des références héritées vers des identifiants actuels.".into(),
        input_schema: json!({
            "type": "object", "required": ["ancien_id"],
            "properties": {
                "ancien_id": { "type": "string", "description": "Ancien identifiant du code" },
                "date":      { "type": "string", "description": "Date de vigueur YYYY-MM-DD" }
            }
        }),
    }
}
async fn do_code_par_ancien_id(args: &Value, client: &LegifranceClient) -> Result<String> {
    let ancien_id = require_str(args, "ancien_id")?;
    let date      = date_arg(args, "date", &today());
    let body      = json!({ "oldId": ancien_id, "date": date });
    let data      = client.post("/consult/legi/tableMatieresByOldId", body).await?;
    Ok(fmt_toc(&data, ancien_id))
}

fn tool_section_par_cid() -> ToolDef {
    ToolDef {
        name: "legifrance_section_par_cid".into(),
        description: "Récupère le contenu d'une section de code ou de texte par son CID canonique. \
            Utile pour naviguer dans la structure d'un code (format LEGISCTA...).".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "CID de la section (LEGISCTA...)" }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_obtenir_article() -> ToolDef {
    ToolDef {
        name: "legifrance_obtenir_article".into(),
        description: "Récupère le texte complet d'un article juridique par son identifiant Légifrance \
            (format LEGIARTI000...). L'ID s'obtient via legifrance_rechercher ou legifrance_consulter_code.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant de l'article (LEGIARTI...)" }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_article_par_numero() -> ToolDef {
    ToolDef {
        name: "legifrance_article_par_numero".into(),
        description: "Récupère un article d'un code par son numéro (ex: '1382', 'L1234-5', 'R123-1'). \
            Plus pratique que legifrance_obtenir_article quand on connaît le numéro mais pas l'ID.".into(),
        input_schema: json!({ "type": "object", "required": ["code", "numero"], "properties": { "code": { "type": "string", "description": "Nom ou ID LEGITEXT du code" }, "numero": { "type": "string", "description": "Numéro de l'article (ex: '1382', 'L1234-1')" }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_article_par_eli() -> ToolDef {
    ToolDef {
        name: "legifrance_article_par_eli".into(),
        description: "Récupère un article par son identifiant ELI (European Legislation Identifier). \
            L'ELI est le format d'URI standardisé européen pour identifier les textes législatifs \
            (ex: eli/loi/2016/10/7/2016-1321/jo/article_1).".into(),
        input_schema: json!({
            "type": "object", "required": ["eli"],
            "properties": {
                "eli": { "type": "string", "description": "Identifiant ELI de l'article" }
            }
        }),
    }
}
async fn do_article_par_eli(args: &Value, client: &LegifranceClient) -> Result<String> {
    let eli  = require_str(args, "eli")?;
    // CORRECTION vs référence : chemin /consult/getArticleWithIdEliOrAlias,
    // body {idEliOrAlias}, pas /consult/getArticleByEli {eli}.
    let body = json!({ "idEliOrAlias": eli });
    let data = client.post("/consult/getArticleWithIdEliOrAlias", body).await?;
    let article = data.get("article").unwrap_or(&data);
    Ok(fmt_article(article))
}

fn tool_articles_meme_numero() -> ToolDef {
    ToolDef {
        name: "legifrance_articles_meme_numero".into(),
        description: "Récupère tous les articles portant le même numéro dans un code. \
            Utile pour les articles qui ont été renumérotés ou qui existent en plusieurs versions.".into(),
        input_schema: json!({
            "type": "object", "required": ["code", "numero"],
            "properties": {
                "code":   { "type": "string", "description": "Nom ou ID LEGITEXT du code" },
                "numero": { "type": "string", "description": "Numéro de l'article" }
            }
        }),
    }
}
async fn do_articles_meme_numero(args: &Value, client: &LegifranceClient) -> Result<String> {
    let code    = require_str(args, "code")?;
    let numero  = require_str(args, "numero")?;
    let code_id = resolve_code(code)?;
    let body    = json!({ "id": code_id, "num": numero });
    let data    = client.post("/consult/getArticlesWithSameNum", body).await?;
    let articles = data["listArticle"].as_array()
        .or_else(|| data["articles"].as_array())
        .cloned()
        .unwrap_or_default();
    if articles.is_empty() { return Ok(format!("Aucun article numéroté '{numero}' dans ce code.")); }
    let mut lines = vec![format!("**{} article(s) portant le numéro '{numero}'**\n", articles.len())];
    for a in &articles {
        lines.push(fmt_article(a));
        lines.push(String::new());
    }
    Ok(lines.join("\n"))
}

fn tool_versions_article() -> ToolDef {
    ToolDef {
        name: "legifrance_versions_article".into(),
        description: "Récupère l'historique complet des versions d'un article (par son CID canonique). \
            Permet de suivre les modifications législatives d'un article au fil du temps.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "CID canonique de l'article (LEGIARTI...)" } } }),
    }
}
fn tool_version_canonique_article() -> ToolDef {
    ToolDef {
        name: "legifrance_version_canonique_article".into(),
        description: "Récupère les informations de version canonique d'un article. \
            Utile pour résoudre l'identifiant stable (CID) d'un article versionné.".into(),
        input_schema: json!({
            "type": "object", "required": ["article_id"],
            "properties": {
                "article_id": { "type": "string", "description": "Identifiant de l'article (ex: LEGIARTI...)" }
            }
        }),
    }
}
async fn do_version_canonique_article(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "article_id")?;
    let body = json!({ "id": id });
    let data = client.post("/search/canonicalArticleVersion", body).await?;
    let mut lines = vec![format!("**Version canonique de l'article `{id}`**\n")];
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            if !v.is_null() {
                let val = match v {
                    Value::String(s) => s.clone(),
                    Value::Bool(b)   => b.to_string(),
                    Value::Number(n) => n.to_string(),
                    other            => other.to_string(),
                };
                if !val.is_empty() { lines.push(format!("- **{k}** : {val}")); }
            }
        }
    }
    if lines.len() > 1 { Ok(lines.join("\n")) }
    else { Ok(format!("Aucune information pour `{id}`.")) }
}

fn tool_version_canonique() -> ToolDef {
    ToolDef {
        name: "legifrance_version_canonique".into(),
        description: "Récupère la version canonique d'un texte à une date donnée. \
            Permet de déterminer quelle version d'un texte était en vigueur à une date précise.".into(),
        input_schema: json!({
            "type": "object", "required": ["cid_text", "date"],
            "properties": {
                "cid_text":    { "type": "string", "description": "CID du texte (ex: LEGITEXT...)" },
                "date":        { "type": "string", "description": "Date de consultation YYYY-MM-DD" },
                "cid_section": { "type": "string", "description": "CID de la section (optionnel)" }
            }
        }),
    }
}
async fn do_version_canonique(args: &Value, client: &LegifranceClient) -> Result<String> {
    let cid_text = require_str(args, "cid_text")?;
    let date     = require_str(args, "date")?;
    let mut body = json!({ "cidText": cid_text, "date": date });
    let cid_section = str_arg(args, "cid_section");
    if !cid_section.is_empty() { body["cidSection"] = json!(cid_section); }
    let data  = client.post("/search/canonicalVersion", body).await?;
    let mut lines = vec![format!("**Version canonique de `{cid_text}` au {date}**\n")];
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            if !v.is_null() {
                let val = match v { Value::String(s) => s.clone(), other => other.to_string() };
                if !val.is_empty() { lines.push(format!("- **{k}** : {val}")); }
            }
        }
    }
    if lines.len() > 1 { Ok(lines.join("\n")) }
    else { Ok("Aucune information de version canonique trouvée.".into()) }
}

fn tool_version_proche() -> ToolDef {
    ToolDef {
        name: "legifrance_version_proche".into(),
        description: "Récupère la version d'un texte la plus proche d'une date donnée. \
            Utile quand la date exacte n'a pas de version et qu'on veut la version voisine.".into(),
        input_schema: json!({
            "type": "object", "required": ["cid_text", "date"],
            "properties": {
                "cid_text":    { "type": "string", "description": "CID du texte" },
                "date":        { "type": "string", "description": "Date souhaitée YYYY-MM-DD" },
                "cid_section": { "type": "string", "description": "CID de la section (optionnel)" }
            }
        }),
    }
}
async fn do_version_proche(args: &Value, client: &LegifranceClient) -> Result<String> {
    let cid_text = require_str(args, "cid_text")?;
    let date     = require_str(args, "date")?;
    let mut body = json!({ "cidText": cid_text, "date": date });
    let cid_section = str_arg(args, "cid_section");
    if !cid_section.is_empty() { body["cidSection"] = json!(cid_section); }
    let data  = client.post("/search/nearestVersion", body).await?;
    let mut lines = vec![format!("**Version la plus proche de `{cid_text}` autour du {date}**\n")];
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            if !v.is_null() {
                let val = match v { Value::String(s) => s.clone(), other => other.to_string() };
                if !val.is_empty() { lines.push(format!("- **{k}** : {val}")); }
            }
        }
    }
    if lines.len() > 1 { Ok(lines.join("\n")) }
    else { Ok("Aucune version proche trouvée.".into()) }
}

fn tool_a_des_versions() -> ToolDef {
    ToolDef {
        name: "legifrance_a_des_versions".into(),
        description: "Vérifie rapidement si un texte juridique possède des versions historiques. \
            Utile avant d'appeler legifrance_historique_texte.".into(),
        input_schema: json!({
            "type": "object", "required": ["text_cid"],
            "properties": {
                "text_cid": { "type": "string", "description": "CID du texte (ex: LEGITEXT...)" }
            }
        }),
    }
}
async fn do_a_des_versions(args: &Value, client: &LegifranceClient) -> Result<String> {
    let cid  = require_str(args, "text_cid")?;
    let path = format!("/chrono/textCid/{cid}");
    let data = client.get(&path).await?;
    let has  = data["hasChronolegi"].as_bool()
        .or_else(|| data["hasVersions"].as_bool())
        .unwrap_or(false);
    if has {
        Ok(format!("✅ Le texte `{cid}` **possède** un historique de versions. Utilisez `legifrance_historique_texte` pour les récupérer."))
    } else {
        Ok(format!("ℹ️ Le texte `{cid}` **ne possède pas** de versions historiques enregistrées."))
    }
}

fn tool_liens_concordance() -> ToolDef {
    ToolDef {
        name: "legifrance_liens_concordance".into(),
        description: "Récupère les liens de concordance d'un article (articles équivalents dans d'autres textes).".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant de l'article (LEGIARTI...)" }, "type": { "type": "string", "description": "Type de lien (optionnel)" } } }),
    }
}
fn tool_liens_relatifs() -> ToolDef {
    ToolDef {
        name: "legifrance_liens_relatifs".into(),
        description: "Récupère les liens relatifs (citants/cités) d'un article juridique.".into(),
        input_schema: json!({
            "type": "object", "required": ["article_id"],
            "properties": {
                "article_id": { "type": "string", "description": "Identifiant de l'article (LEGIARTI...)" }
            }
        }),
    }
}
async fn do_liens_relatifs(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "article_id")?;
    // CORRECTION vs référence : chemin /consult/relatedLinksArticle,
    // body {articleId}, pas /consult/relatifs {id}.
    let body = json!({ "articleId": id });
    let data = client.post("/consult/relatedLinksArticle", body).await?;
    let mut lines = vec![format!("**Liens relatifs de `{id}`**\n")];
    if let Some(obj) = data.as_object() {
        for (section_name, items) in obj {
            if let Some(arr) = items.as_array() {
                if !arr.is_empty() {
                    lines.push(format!("\n**{section_name}** ({})", arr.len()));
                    for lien in arr.iter().take(10) {
                        let t   = lien["titre"].as_str().or_else(|| lien["title"].as_str()).unwrap_or("");
                        let rid = lien["id"].as_str().unwrap_or("");
                        lines.push(format!("  - {t}{}", if rid.is_empty() { String::new() } else { format!(" — `{rid}`") }));
                    }
                }
            }
        }
    }
    if lines.len() > 1 { Ok(lines.join("\n")) }
    else { Ok(format!("Aucun lien relatif trouvé pour `{id}`.")) }
}

fn tool_liens_service_public() -> ToolDef {
    ToolDef {
        name: "legifrance_liens_service_public".into(),
        description: "Récupère les liens vers service-public.fr associés à un article.".into(),
        input_schema: json!({
            "type": "object", "required": ["article_id"],
            "properties": {
                "article_id": { "type": "string", "description": "Identifiant de l'article" }
            }
        }),
    }
}
async fn do_liens_service_public(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "article_id")?;
    // CORRECTION vs référence : chemin /consult/servicePublicLinksArticle,
    // body {articleCid}, pas /consult/getLinksServicePublic {id}.
    let body = json!({ "articleCid": id });
    let data = client.post("/consult/servicePublicLinksArticle", body).await?;
    let items = data["liens"].as_array()
        .or_else(|| data["links"].as_array())
        .cloned()
        .unwrap_or_default();
    if items.is_empty() { return Ok(format!("Aucun lien service-public.fr pour `{id}`.")); }
    let mut lines = vec![format!("**{} lien(s) service-public.fr**\n", items.len())];
    for lien in &items {
        let titre = lien["titre"].as_str().or_else(|| lien["title"].as_str()).unwrap_or("");
        let url   = lien["url"].as_str().unwrap_or("");
        if !url.is_empty() { lines.push(format!("- [{titre}]({url})")); }
        else { lines.push(format!("- {titre}")); }
    }
    Ok(lines.join("\n"))
}

fn tool_a_liens_service_public() -> ToolDef {
    ToolDef {
        name: "legifrance_a_liens_service_public".into(),
        description: "Vérifie si un article possède des liens vers service-public.fr.".into(),
        input_schema: json!({
            "type": "object", "required": ["article_id"],
            "properties": {
                "article_id": { "type": "string", "description": "Identifiant de l'article" }
            }
        }),
    }
}
async fn do_a_liens_service_public(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "article_id")?;
    // NOTE : même famille de paramètre que liens_service_public (articleCid).
    // Chemin /consult/hasLinksServicePublic non confirmé par la référence —
    // à vérifier en conditions réelles si ce outil échoue.
    let body = json!({ "articleCid": id });
    let data = client.post("/consult/hasLinksServicePublic", body).await?;
    let has  = data["hasLinks"].as_bool()
        .or_else(|| data["hasLinksServicePublic"].as_bool())
        .unwrap_or(false);
    if has {
        Ok(format!("✅ L'article `{id}` **possède** des liens service-public.fr. Utilisez `legifrance_liens_service_public` pour les récupérer."))
    } else {
        Ok(format!("ℹ️ L'article `{id}` **ne possède pas** de liens service-public.fr."))
    }
}

// ============================================================================
// ── LOIS & RÈGLEMENTS (LODA / LEGI) ──
// ============================================================================

fn tool_loi_decret() -> ToolDef {
    ToolDef {
        name: "legifrance_loi_decret".into(),
        description: "Consulte le contenu complet d'une loi, décret, ordonnance ou arrêté (fonds LODA). \
            Retourne les articles structurés du texte.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant du texte (LEGITEXT..., JORFTEXT...)" }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_legi_part() -> ToolDef {
    ToolDef {
        name: "legifrance_legi_part".into(),
        description: "Récupère une partie d'un texte LEGI par son identifiant de partie. \
            Utile pour naviguer dans les sous-sections d'un code ou texte volumineux.".into(),
        input_schema: json!({ "type": "object", "required": [], "properties": { "id": { "type": "string", "description": "Identifiant de la partie LEGI" }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_lister_loda() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_loda".into(),
        description: "Liste les lois, ordonnances et décrets autonomes disponibles dans Légifrance (fonds LODA).".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "natures":               { "type": "array", "items": { "type": "string" },
                    "description": "Natures à filtrer (ex: ['LOI', 'ORDONNANCE', 'DECRET', 'ARRETE'])" },
                "en_vigueur_seulement":  { "type": "boolean", "default": false },
                "date_debut":            { "type": "string", "description": "Date début YYYY-MM-DD" },
                "date_fin":              { "type": "string", "description": "Date fin YYYY-MM-DD" },
                "nb_resultats":          { "type": "integer", "default": 20 }
            }
        }),
    }
}
async fn do_lister_loda(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": 1, "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100) });
    if let Some(natures) = args["natures"].as_array() {
        body["natures"] = json!(natures);
    }
    if args["en_vigueur_seulement"].as_bool().unwrap_or(false) {
        body["legalStatus"] = json!(["VIGUEUR"]);
    }
    let dd = str_arg(args, "date_debut");
    let df = str_arg(args, "date_fin");
    if !dd.is_empty() || !df.is_empty() {
        let mut pub_date = json!({});
        if !dd.is_empty() { pub_date["start"] = json!(dd); }
        if !df.is_empty() { pub_date["end"]   = json!(df); }
        body["publicationDate"] = pub_date;
    }
    let data  = client.post("/list/loda", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun texte LODA trouvé.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(items.len() as i64);
    let mut lines = vec![format!("**{total} texte(s) LODA** — {} affiché(s)\n", items.len())];
    for t in &items {
        let titre  = t["titre"].as_str().or_else(|| t["title"].as_str()).unwrap_or("?");
        let nature = t["nature"].as_str().unwrap_or("");
        let date_p = t["dateParution"].as_str().unwrap_or("");
        let id     = t["id"].as_str().or_else(|| t["cid"].as_str()).unwrap_or("");
        let mut line = format!("- **{titre}**");
        if !nature.is_empty() { line.push_str(&format!(" [{nature}]")); }
        if !date_p.is_empty() { line.push_str(&format!(" — {date_p}")); }
        if !id.is_empty()     { line.push_str(&format!(" — `{id}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn tool_historique_texte() -> ToolDef {
    ToolDef {
        name: "legifrance_historique_texte".into(),
        description: "Récupère l'historique complet des versions d'un texte juridique sur une période. \
            Permet de suivre l'évolution législative d'un code ou d'une loi dans le temps.".into(),
        input_schema: json!({
            "type": "object", "required": ["text_cid", "annee_debut", "annee_fin"],
            "properties": {
                "text_cid":     { "type": "string",  "description": "CID du texte (ex: LEGITEXT...)" },
                "annee_debut":  { "type": "integer", "description": "Année de début" },
                "annee_fin":    { "type": "integer", "description": "Année de fin" },
                "date_consult": { "type": "string",  "description": "Date de consultation YYYY-MM-DD (défaut: aujourd'hui)" }
            }
        }),
    }
}
async fn do_historique_texte(args: &Value, client: &LegifranceClient) -> Result<String> {
    let text_cid    = require_str(args, "text_cid")?;
    let annee_debut = int_arg(args, "annee_debut", 0);
    let annee_fin   = int_arg(args, "annee_fin", 0);
    if annee_debut == 0 || annee_fin == 0 { anyhow::bail!("annee_debut et annee_fin sont obligatoires."); }
    let date_consult = date_arg(args, "date_consult", &today());
    let body = json!({
        "textCid":     text_cid,
        "dateConsult": date_consult,
        "startYear":   annee_debut,
        "endYear":     annee_fin,
    });
    let data = client.post("/chrono/textCid", body).await?;
    let regs = data["regroupements"].as_array().cloned().unwrap_or_default();
    if regs.is_empty() {
        return Ok(format!("Aucune version trouvée pour `{text_cid}` entre {annee_debut} et {annee_fin}."));
    }
    let mut lines = vec![format!("**{} regroupement(s) pour `{text_cid}` ({annee_debut}–{annee_fin})**\n", regs.len())];
    for rg in &regs {
        let title_rg = rg["title"].as_str().unwrap_or("");
        let vers     = rg["versions"].as_array().cloned().unwrap_or_default();
        if !title_rg.is_empty() { lines.push(format!("\n### {title_rg}")); }
        for v in &vers {
            let debut = fmt_date(&v["dateDebut"]);
            let fin   = fmt_date(&v["dateFin"]);
            let vid   = v["id"].as_str().unwrap_or("");
            let etat  = v["etat"].as_str().unwrap_or("");
            let range = match (debut.is_empty(), fin.is_empty()) {
                (false, false) => format!("{debut} → {fin}"),
                (false, true)  => format!("{debut} → en vigueur"),
                _              => String::new(),
            };
            lines.push(format!("- {range} *{etat}* `{vid}`"));
        }
    }
    Ok(lines.join("\n"))
}

fn tool_eli_alias_texte() -> ToolDef {
    ToolDef {
        name: "legifrance_eli_alias_texte".into(),
        description: "Résout un alias ELI vers le CID du texte correspondant dans Légifrance.".into(),
        input_schema: json!({
            "type": "object", "required": ["eli"],
            "properties": {
                "eli": { "type": "string", "description": "Identifiant ELI (ex: eli/loi/2016/10/7/2016-1321/jo/texte)" }
            }
        }),
    }
}
async fn do_eli_alias_texte(args: &Value, client: &LegifranceClient) -> Result<String> {
    let eli  = require_str(args, "eli")?;
    // CORRECTION vs référence : chemin /consult/eliAndAliasRedirectionTexte,
    // body {idEliOrAlias}, pas /consult/getEliAlias {eli}.
    let body = json!({ "idEliOrAlias": eli });
    let data = client.post("/consult/eliAndAliasRedirectionTexte", body).await?;
    let cid  = data["cid"].as_str().or_else(|| data["textCid"].as_str()).unwrap_or("");
    if !cid.is_empty() {
        Ok(format!("✅ L'ELI `{eli}` correspond au CID : `{cid}`"))
    } else {
        Ok(format!("Aucun texte trouvé pour l'ELI : `{eli}`"))
    }
}

fn tool_versions_element() -> ToolDef {
    ToolDef {
        name: "legifrance_versions_element".into(),
        description: "Récupère toutes les versions chronologiques d'un article ou d'une section. \
            Permet de voir toutes les modifications d'un article au sein d'un texte donné.".into(),
        input_schema: json!({
            "type": "object", "required": ["text_cid", "element_cid"],
            "properties": {
                "text_cid":    { "type": "string", "description": "CID du texte parent" },
                "element_cid": { "type": "string", "description": "CID de l'article ou section" }
            }
        }),
    }
}
async fn do_versions_element(args: &Value, client: &LegifranceClient) -> Result<String> {
    let text_cid    = require_str(args, "text_cid")?;
    let element_cid = require_str(args, "element_cid")?;
    let body = json!({ "textCid": text_cid, "elementCid": element_cid });
    let data = client.post("/chrono/textCidAndElementCid", body).await?;
    let regs = data["regroupements"].as_array().cloned().unwrap_or_default();
    if regs.is_empty() {
        return Ok(format!("Aucune version trouvée pour l'élément `{element_cid}`."));
    }
    let mut lines = vec![format!("**{} regroupement(s) pour `{element_cid}`**\n", regs.len())];
    for rg in &regs {
        let title_rg = rg["title"].as_str().unwrap_or("");
        let vers     = rg["versions"].as_array().cloned().unwrap_or_default();
        if !title_rg.is_empty() { lines.push(format!("\n### {title_rg}")); }
        for v in &vers {
            let debut = fmt_date(&v["dateDebut"]);
            let fin   = fmt_date(&v["dateFin"]);
            let vid   = v["id"].as_str().unwrap_or("");
            let etat  = v["etat"].as_str().unwrap_or("");
            lines.push(format!("- {debut} → {fin} *{etat}* `{vid}`"));
        }
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── JOURNAL OFFICIEL (JORF) ──
// ============================================================================

fn tool_jorf() -> ToolDef {
    ToolDef {
        name: "legifrance_jorf".into(),
        description: "Consulte et retourne le contenu complet d'un texte du Journal Officiel. \
            IMPORTANT : text_cid doit être un CID de type JORFTEXT... obtenu via \
            legifrance_sommaire_jorf ou legifrance_rechercher.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant JORFTEXT..." } } }),
    }
}
fn tool_jorf_part() -> ToolDef {
    ToolDef {
        name: "legifrance_jorf_part".into(),
        description: "Récupère une partie d'un texte JORF par son identifiant.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant de la partie JORF" } } }),
    }
}
fn tool_jo_par_nor() -> ToolDef {
    ToolDef {
        name: "legifrance_jo_par_nor".into(),
        description: "Consulte un texte du Journal Officiel par son numéro NOR. \
            Le NOR (Numéro d'Ordre du Registre) identifie uniquement chaque texte officiel \
            (ex: PRMD2117108D, JUSB2123813A).".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "NOR du texte ou identifiant JORF (ex: PRMD2117108D)" } } }),
    }
}
fn tool_derniers_jo() -> ToolDef {
    ToolDef {
        name: "legifrance_derniers_jo".into(),
        description: "Récupère les N derniers numéros du Journal Officiel publiés (les plus récents en premier), \
            avec leurs dates et identifiants. Ne prend pas de période : pour une date ou une période précise, \
            utiliser legifrance_sommaire_jorf.".into(),
        input_schema: json!({ "type": "object", "required": [], "properties": { "taille": { "type": "integer", "description": "Nombre de JO à récupérer (défaut: 10)" } } }),
    }
}
fn tool_sommaire_jorf() -> ToolDef {
    ToolDef {
        name: "legifrance_sommaire_jorf".into(),
        description: "Consulte le sommaire du Journal Officiel sur une date ou une période. \
            Les CID retournés (ex: JORFTEXT...) peuvent être passés à legifrance_jorf.".into(),
        input_schema: json!({ "type": "object", "required": [], "properties": { "date": { "type": "string", "description": "Date du JO YYYY-MM-DD (requis sauf si jorf_id fourni)" }, "date_fin": { "type": "string", "description": "Date de fin YYYY-MM-DD (pour une période)" }, "jorf_id": { "type": "string", "description": "Identifiant direct du JO (JORFCONT...)" }, "recherche": { "type": "string", "description": "Texte de recherche dans le sommaire" }, "nb_resultats": { "type": "integer", "description": "Nombre de résultats (défaut: 20)" } } }),
    }
}
fn tool_tables_annuelles() -> ToolDef {
    ToolDef {
        name: "legifrance_tables_annuelles".into(),
        description: "Récupère la liste des tables annuelles du Journal Officiel disponibles dans Légifrance. \
            Les tables annuelles recensent tous les textes publiés au JO pour une période donnée.".into(),
        input_schema: json!({
            "type": "object", "required": ["annee_fin"],
            "properties": {
                "annee_fin":   { "type": "integer", "description": "Année de fin" },
                "annee_debut": { "type": "integer", "description": "Année de début (optionnel)" }
            }
        }),
    }
}
async fn do_tables_annuelles(args: &Value, client: &LegifranceClient) -> Result<String> {
    let annee_fin = int_arg(args, "annee_fin", 0);
    if annee_fin == 0 { anyhow::bail!("annee_fin est obligatoire."); }
    let mut body = json!({ "endYear": annee_fin });
    let annee_debut = int_arg(args, "annee_debut", 0);
    if annee_debut > 0 { body["startYear"] = json!(annee_debut); }
    let data  = client.post("/consult/getTables", body).await?;
    let items = data["tables"].as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return Ok(format!("Aucune table annuelle trouvée pour la période demandée."));
    }
    let mut lines = vec![format!("**{} table(s) annuelle(s)**\n", items.len())];
    for t in &items {
        let annee = t["annee"].as_i64().or_else(|| t["year"].as_i64()).unwrap_or(0);
        let id    = t["id"].as_str().or_else(|| t["cid"].as_str()).unwrap_or("");
        let mut line = format!("- **{annee}**");
        if !id.is_empty() { line.push_str(&format!(" — `{id}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn tool_dates_sans_jo() -> ToolDef {
    ToolDef {
        name: "legifrance_dates_sans_jo".into(),
        description: "Retourne les dates sans Journal Officiel, éventuellement filtrées sur une année. \
            Utile pour connaître les jours sans publication du JO (week-ends, jours fériés...).".into(),
        input_schema: json!({
            "type": "object", "required": [],
            "properties": {
                "annee": { "type": "integer", "description": "Année pour filtrer les résultats (optionnel — toutes les années sinon)" }
            }
        }),
    }
}
async fn do_dates_sans_jo(args: &Value, client: &LegifranceClient) -> Result<String> {
    let annee = int_arg(args, "annee", 0);
    // CORRECTION vs référence : /misc/datesWithoutJo est un GET sans aucun
    // paramètre (pas de filtre par année côté API) — on filtre nous-mêmes
    // le résultat si une année a été demandée.
    let data  = client.get("/misc/datesWithoutJo").await?;
    let mut dates: Vec<Value> = data["dates"].as_array()
        .or_else(|| data["datesWithoutJo"].as_array())
        .or_else(|| data.as_array())
        .cloned()
        .unwrap_or_default();
    if annee > 0 {
        let prefix = annee.to_string();
        dates.retain(|d| d.as_str().map(|s| s.starts_with(&prefix)).unwrap_or(false));
    }
    if dates.is_empty() {
        return Ok(if annee > 0 {
            format!("Aucune date sans JO trouvée pour {annee}.")
        } else {
            "Aucune date sans JO trouvée.".to_string()
        });
    }
    let mut lines = vec![format!("**{} date(s) sans Journal Officiel**\n", dates.len())];
    for d in &dates {
        let s = d.as_str().unwrap_or("");
        if !s.is_empty() { lines.push(format!("- {s}")); }
    }
    Ok(lines.join("\n"))
}

fn tool_annees_sans_table() -> ToolDef {
    ToolDef {
        name: "legifrance_annees_sans_table".into(),
        description: "Retourne les années sans table annuelle du Journal Officiel disponible dans Légifrance. \
            Utile avant d'appeler legifrance_tables_annuelles.".into(),
        input_schema: json!({ "type": "object", "properties": {} }),
    }
}
async fn do_annees_sans_table(client: &LegifranceClient) -> Result<String> {
    // NOTE : par cohérence avec /misc/datesWithoutJo (confirmé GET sans body
    // par la référence), on suppose /misc/yearsWithoutTable de la même
    // famille — non confirmé explicitement, à vérifier si ce outil échoue.
    let data   = client.get("/misc/yearsWithoutTable").await?;
    let annees = data["years"].as_array()
        .or_else(|| data["annees"].as_array())
        .or_else(|| data.as_array())
        .cloned()
        .unwrap_or_default();
    if annees.is_empty() { return Ok("Toutes les années ont une table annuelle disponible.".into()); }
    let list: Vec<String> = annees.iter()
        .filter_map(|a| a.as_i64().map(|n| n.to_string()))
        .collect();
    Ok(format!("**Années sans table annuelle** : {}", list.join(", ")))
}

// ============================================================================
// ── CONVENTIONS COLLECTIVES (KALI) ──
// ============================================================================

fn tool_conventions() -> ToolDef {
    ToolDef {
        name: "legifrance_conventions".into(),
        description: "Liste et recherche les conventions collectives disponibles dans Légifrance (fonds KALI).".into(),
        input_schema: json!({ "type": "object", "required": [], "properties": { "query": { "type": "string", "description": "Recherche par titre de convention" }, "idcc": { "type": "string", "description": "Numéro IDCC exact" }, "taille": { "type": "integer", "description": "Nombre de résultats (défaut: 20)" } } }),
    }
}
fn tool_convention_par_idcc() -> ToolDef {
    ToolDef {
        name: "legifrance_convention_par_idcc".into(),
        description: "Consulte la structure principale d'une convention collective par son numéro IDCC.".into(),
        input_schema: json!({
            "type": "object", "required": ["idcc"],
            "properties": {
                "idcc": { "type": "string", "description": "Numéro IDCC (ex: '0016', '1486', '3248')" }
            }
        }),
    }
}
async fn do_convention_par_idcc(args: &Value, client: &LegifranceClient) -> Result<String> {
    let idcc = require_str(args, "idcc")?;
    // CORRECTION vs référence : POST /consult/kaliContIdcc attend {idcc}, pas {id}.
    let body = json!({ "idcc": idcc });
    let data = client.post("/consult/kaliContIdcc", body).await?;
    let default_titre = format!("Convention IDCC {idcc}");
    let titre  = data["titre"].as_str().unwrap_or(&default_titre);
    let rid    = data["id"].as_str().unwrap_or("");
    let num    = data["num"].as_str().unwrap_or("");
    let nature = data["nature"].as_str().unwrap_or("");
    let sects  = data["sections"].as_array().cloned().unwrap_or_default();
    let mut lines = vec![format!("# {titre}\n"), format!("**IDCC** : {idcc}")];
    if !rid.is_empty()    { lines.push(format!("**ID** : `{rid}`")); }
    if !num.is_empty()    { lines.push(format!("**Numéro** : {num}")); }
    if !nature.is_empty() { lines.push(format!("**Nature** : {nature}")); }
    if !sects.is_empty() {
        lines.push(format!("\n**{} section(s)**", sects.len()));
        for s in sects.iter().take(10) {
            let t = s["titre"].as_str().or_else(|| s["title"].as_str()).unwrap_or("");
            if !t.is_empty() { lines.push(format!("  - {t}")); }
        }
    }
    Ok(lines.join("\n"))
}

fn tool_convention_cont() -> ToolDef {
    ToolDef {
        name: "legifrance_convention_cont".into(),
        description: "Consulte le conteneur d'une convention collective par son identifiant KALI (KALICONT...).".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant KALICONT..." }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_convention_texte() -> ToolDef {
    ToolDef {
        name: "legifrance_convention_texte".into(),
        description: "Consulte le texte complet d'une convention collective par son identifiant KALI (KALITEXT...).".into(),
        input_schema: json!({
            "type": "object", "required": ["text_id"],
            "properties": {
                "text_id": { "type": "string", "description": "Identifiant KALITEXT..." }
            }
        }),
    }
}
async fn do_convention_texte(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "text_id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/kaliText", body).await?;
    let titre = data["title"].as_str().or_else(|| data["titre"].as_str()).unwrap_or(id);
    let nor   = data["nor"].as_str().unwrap_or("");
    let etat  = data["etat"].as_str().unwrap_or("");
    let mut lines = vec![format!("# {titre}\n")];
    if !nor.is_empty()  { lines.push(format!("**NOR** : {nor}")); }
    if !etat.is_empty() { lines.push(format!("**État** : {etat}")); }
    let articles = data["articles"].as_array()
        .or_else(|| data["sections"].as_array())
        .cloned()
        .unwrap_or_default();
    if !articles.is_empty() {
        lines.push(format!("\n**{} article(s)/section(s)**\n", articles.len()));
        for a in articles.iter().take(30) {
            lines.push(fmt_article(a));
            lines.push(String::new());
        }
        if articles.len() > 30 {
            lines.push(format!("*… {} élément(s) supplémentaire(s)*", articles.len() - 30));
        }
    }
    Ok(lines.join("\n"))
}

fn tool_convention_article() -> ToolDef {
    ToolDef {
        name: "legifrance_convention_article".into(),
        description: "Consulte un article spécifique d'une convention collective par son identifiant KALI (KALIARTI...).".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant KALIARTI..." }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_convention_section() -> ToolDef {
    ToolDef {
        name: "legifrance_convention_section".into(),
        description: "Consulte une section d'une convention collective par son identifiant KALI (KALISECT...).".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant KALISECT..." }, "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD" } } }),
    }
}
fn tool_jurisprudence() -> ToolDef {
    ToolDef {
        name: "legifrance_jurisprudence".into(),
        description: "Consulte une décision de justice française (jurisprudence). \
            Retourne le texte complet de la décision avec juridiction et date.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant de la décision (JURITEXT...)" } } }),
    }
}
fn tool_jurisprudence_plan_classement() -> ToolDef {
    ToolDef {
        name: "legifrance_jurisprudence_plan_classement".into(),
        description: "Consulte le plan de classement de la jurisprudence (arborescence thématique). \
            Permet de naviguer dans les catégories de jurisprudence par thème ou matière.".into(),
        input_schema: json!({ "type": "object", "required": ["query"], "properties": { "query": { "type": "string", "description": "Termes de recherche dans le plan de classement" }, "fond": { "type": "string", "description": "Fonds documentaire de jurisprudence" }, "niveau": { "type": "integer", "description": "Niveau hiérarchique" } } }),
    }
}
fn tool_jurisprudence_ancien_id() -> ToolDef {
    ToolDef {
        name: "legifrance_jurisprudence_ancien_id".into(),
        description: "Consulte une décision de jurisprudence par son ancien identifiant.".into(),
        input_schema: json!({
            "type": "object", "required": ["ancien_id"],
            "properties": {
                "ancien_id": { "type": "string", "description": "Ancien identifiant de la décision" }
            }
        }),
    }
}
async fn do_jurisprudence_ancien_id(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "ancien_id")?;
    let body = json!({ "oldId": id });
    let data = client.post("/consult/getJuriByOldId", body).await?;
    let txt = data["text"].as_array().and_then(|a| a.first()).cloned()
        .unwrap_or_else(|| data["text"].clone());
    let txt = if txt.is_null() { data.clone() } else { txt };
    let titre = txt["titre"].as_str().or_else(|| txt["title"].as_str()).unwrap_or(id);
    let raw   = txt["texteHtml"].as_str().or_else(|| txt["texte"].as_str()).unwrap_or("");
    let texte = strip_html(raw);
    let mut lines = vec![format!("# {titre}\n")];
    if !texte.is_empty() {
        let excerpt = if texte.len() > 5000 { &texte[..5000] } else { &texte };
        lines.push(excerpt.to_string());
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── CNIL ──
// ============================================================================

fn tool_cnil() -> ToolDef {
    ToolDef {
        name: "legifrance_cnil".into(),
        description: "Consulte une délibération ou décision CNIL par son identifiant (ex: CNILTEXT...).".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant CNILTEXT..." } } }),
    }
}
fn tool_cnil_ancien_id() -> ToolDef {
    ToolDef {
        name: "legifrance_cnil_ancien_id".into(),
        description: "Consulte une délibération CNIL par son ancien identifiant.".into(),
        input_schema: json!({
            "type": "object", "required": ["ancien_id"],
            "properties": {
                "ancien_id": { "type": "string", "description": "Ancien identifiant du texte CNIL" }
            }
        }),
    }
}
async fn do_cnil_ancien_id(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "ancien_id")?;
    let body = json!({ "oldId": id });
    let data = client.post("/consult/getCnilByOldId", body).await?;
    let txt = data["text"].as_array().and_then(|a| a.first()).cloned()
        .unwrap_or_else(|| data["text"].clone());
    let txt = if txt.is_null() { data.clone() } else { txt };
    let titre = txt["titre"].as_str().or_else(|| txt["title"].as_str()).unwrap_or(id);
    let raw   = txt["texteHtml"].as_str().or_else(|| txt["texte"].as_str()).unwrap_or("");
    let texte = strip_html(raw);
    let mut lines = vec![format!("# {titre}\n")];
    if !texte.is_empty() {
        let excerpt = if texte.len() > 5000 { &texte[..5000] } else { &texte };
        lines.push(excerpt.to_string());
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── ACCO (accords d'entreprise) ──
// ============================================================================

fn tool_acco() -> ToolDef {
    ToolDef {
        name: "legifrance_acco".into(),
        description: "Consulte un accord d'entreprise (fonds ACCO) par son identifiant.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant de l'accord (ex: T07523000123)" } } }),
    }
}
fn tool_circulaire() -> ToolDef {
    ToolDef {
        name: "legifrance_circulaire".into(),
        description: "Consulte une circulaire administrative (fonds CIRC) par son identifiant.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant de la circulaire" } } }),
    }
}
fn tool_dossier_legislatif() -> ToolDef {
    ToolDef {
        name: "legifrance_dossier_legislatif".into(),
        description: "Consulte un dossier législatif complet par son identifiant. \
            Regroupe tous les documents liés à l'élaboration d'une loi : \
            projets, amendements, rapports, textes adoptés.".into(),
        input_schema: json!({ "type": "object", "required": ["id"], "properties": { "id": { "type": "string", "description": "Identifiant du dossier législatif" } } }),
    }
}
fn tool_lister_dossiers_legislatifs() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_dossiers_legislatifs".into(),
        description: "Liste les dossiers législatifs d'une législature donnée. \
            Utilisez legifrance_lister_legislatures pour obtenir l'ID de législature.".into(),
        input_schema: json!({
            "type": "object", "required": ["legislature_id"],
            "properties": {
                "legislature_id": { "type": "integer", "description": "ID numérique de la législature (voir legifrance_lister_legislatures)" },
                "type_dossier":   { "type": "string", "description": "Type de dossier (ex: LOI_PUBLIEE)" }
            }
        }),
    }
}
fn tool_lister_legislatures() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_legislatures".into(),
        description: "Liste toutes les législatures disponibles dans Légifrance. \
            Retourne les identifiants nécessaires pour lister les dossiers législatifs.".into(),
        input_schema: json!({ "type": "object", "properties": {} }),
    }
}
async fn do_lister_legislatures(client: &LegifranceClient) -> Result<String> {
    let data  = client.post("/list/legislatures", json!({})).await?;
    let items = data["legislatures"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucune législature trouvée.".into()); }
    let mut lines = vec![format!("**{} législature(s)**\n", items.len())];
    for leg in &items {
        let num   = leg["numero"].as_i64()
            .or_else(|| leg["num"].as_i64())
            .or_else(|| leg["id"].as_i64())
            .map(|n| n.to_string())
            .unwrap_or_else(|| leg["id"].as_str().unwrap_or("?").to_string());
        let debut = leg["dateDebut"].as_str().or_else(|| leg["startDate"].as_str()).unwrap_or("");
        let fin   = leg["dateFin"].as_str().or_else(|| leg["endDate"].as_str()).unwrap_or("en cours");
        let rid   = leg["id"].as_str().unwrap_or("");
        let mut line = format!("- **Législature {num}**");
        if !debut.is_empty() { line.push_str(&format!(" ({debut} → {fin})")); }
        if !rid.is_empty()   { line.push_str(&format!(" — `{rid}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── DÉBATS & QUESTIONS PARLEMENTAIRES ──
// ============================================================================

fn tool_debat() -> ToolDef {
    ToolDef {
        name: "legifrance_debat".into(),
        description: "Consulte le compte-rendu d'un débat parlementaire par son identifiant. \
            Couvre les séances de l'Assemblée Nationale et du Sénat.".into(),
        input_schema: json!({
            "type": "object", "required": ["debat_id"],
            "properties": {
                "debat_id": { "type": "string", "description": "Identifiant du débat parlementaire" }
            }
        }),
    }
}
async fn do_debat(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "debat_id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/debat", body).await?;
    let deb = data["debat"].as_array().and_then(|a| a.first()).cloned()
        .unwrap_or_else(|| data["debat"].clone());
    let deb = if deb.is_null() { data.clone() } else { deb };
    let date_s    = deb["dateSeance"].as_str().unwrap_or("");
    let assemblee = deb["typeAssemblee"].as_str().unwrap_or("");
    let legis     = deb["legislature"].as_str().map(|s| s.to_string())
        .or_else(|| deb["legislature"].as_i64().map(|n| n.to_string()))
        .unwrap_or_default();
    let session   = deb["session"].as_str().unwrap_or("");
    let rid       = deb["id"].as_str().unwrap_or(id);
    let mut lines = vec![format!("# Débat parlementaire `{rid}`\n")];
    if !date_s.is_empty()    { lines.push(format!("**Séance du** : {date_s}")); }
    if !assemblee.is_empty() { lines.push(format!("**Assemblée** : {assemblee}")); }
    if !legis.is_empty()     { lines.push(format!("**Législature** : {legis}")); }
    if !session.is_empty()   { lines.push(format!("**Session** : {session}")); }
    let raw   = deb["texteHtml"].as_str().or_else(|| deb["texte"].as_str()).unwrap_or("");
    let texte = strip_html(raw);
    if !texte.is_empty() {
        lines.push(String::new());
        let excerpt = if texte.len() > 5000 { &texte[..5000] } else { &texte };
        lines.push(excerpt.to_string());
    }
    Ok(lines.join("\n"))
}

fn tool_lister_debats_parlementaires() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_debats_parlementaires".into(),
        description: "Liste les débats parlementaires disponibles dans Légifrance.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "types_publication": { "type": "array", "items": { "type": "string" },
                    "description": "Types de publication (['AN'] pour Assemblée Nationale, ['SENAT'])" },
                "date_parution":    { "type": "string",  "description": "Date de parution YYYY-MM-DD" },
                "nb_resultats":     { "type": "integer", "default": 20 }
            }
        }),
    }
}
async fn do_lister_debats_parlementaires(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": 1, "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100) });
    if let Some(types) = args["types_publication"].as_array() {
        body["typesPublication"] = json!(types);
    }
    let date_p = str_arg(args, "date_parution");
    if !date_p.is_empty() { body["dateParution"] = json!(date_p); }
    let data  = client.post("/list/debatsParlementaires", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun débat parlementaire trouvé.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(items.len() as i64);
    let mut lines = vec![format!("**{total} débat(s)** — {} affiché(s)\n", items.len())];
    for d in &items {
        let t   = d["titre"].as_str().or_else(|| d["title"].as_str()).unwrap_or("?");
        let rid = d["id"].as_str().unwrap_or("");
        let dt  = d["dateSeance"].as_str().or_else(|| d["dateParution"].as_str()).unwrap_or("");
        let mut line = format!("- **{t}**");
        if !dt.is_empty()  { line.push_str(&format!(" — {dt}")); }
        if !rid.is_empty() { line.push_str(&format!(" — `{rid}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn tool_lister_questions_parlementaires() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_questions_parlementaires".into(),
        description: "Liste les questions écrites parlementaires disponibles dans Légifrance.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "parlement_types":     { "type": "array", "items": { "type": "string" },
                    "description": "Types de parlement (['AN'], ['SENAT'])" },
                "periode_publication": { "type": "string",  "description": "Période de publication" },
                "nb_resultats":        { "type": "integer", "default": 20 }
            }
        }),
    }
}
async fn do_lister_questions_parlementaires(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": 1, "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100) });
    if let Some(types) = args["parlement_types"].as_array() {
        // CORRECTION vs référence : le champ attendu est "parlementTypes".
        body["parlementTypes"] = json!(types);
    }
    let periode = str_arg(args, "periode_publication");
    if !periode.is_empty() { body["periodePublication"] = json!(periode); }
    // CORRECTION vs référence : le chemin réel est
    // /list/questionsEcritesParlementaires (pas /list/questionsParlementaires).
    let data  = client.post("/list/questionsEcritesParlementaires", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucune question parlementaire trouvée.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(items.len() as i64);
    let mut lines = vec![format!("**{total} question(s)** — {} affiché(s)\n", items.len())];
    for q in &items {
        let t   = q["titre"].as_str().or_else(|| q["title"].as_str()).unwrap_or("?");
        let rid = q["id"].as_str().unwrap_or("");
        let dt  = q["datePublication"].as_str().unwrap_or("");
        let mut line = format!("- **{t}**");
        if !dt.is_empty()  { line.push_str(&format!(" — {dt}")); }
        if !rid.is_empty() { line.push_str(&format!(" — `{rid}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── BOCC / BODMR ──
// ============================================================================

fn tool_lister_bocc() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_bocc".into(),
        description: "Liste les numéros du BOCC (Bulletin Officiel des Conventions Collectives).".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "interval_publication": { "type": "string",
                    "description": "Intervalle de publication (ex: '01/01/2024 > 31/12/2024')" },
                "nb_resultats": { "type": "integer", "default": 20 }
            }
        }),
    }
}
async fn do_lister_bocc(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": 1, "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100) });
    let interval = str_arg(args, "interval_publication");
    if !interval.is_empty() { body["intervalPublication"] = json!(interval); }
    let data  = client.post("/list/bocc", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun BOCC trouvé.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(items.len() as i64);
    let mut lines = vec![format!("**{total} BOCC** — {} affiché(s)\n", items.len())];
    for b in &items {
        let rid  = b["id"].as_str().unwrap_or("");
        let glob = b["globalTitle"].as_str().or_else(|| b["titre"].as_str()).unwrap_or("");
        let date = b["datePublication"].as_str().unwrap_or("");
        let mut line = if !glob.is_empty() { format!("- **{glob}**") } else { "- BOCC".to_string() };
        if !date.is_empty() { line.push_str(&format!(" — {date}")); }
        if !rid.is_empty()  { line.push_str(&format!(" — `{rid}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn tool_lister_bocc_textes() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_bocc_textes".into(),
        description: "Liste les textes d'un numéro de BOCC donné.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "bocc_id":      { "type": "string", "description": "Identifiant du BOCC principal" },
                "idccs":        { "type": "array",  "items": { "type": "string" },
                    "description": "Liste d'IDCC pour filtrer (ex: ['0016', '1486'])" },
                "nb_resultats": { "type": "integer", "default": 20 }
            }
        }),
    }
}
async fn do_lister_bocc_textes(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": 1, "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100) });
    let bocc_id = str_arg(args, "bocc_id");
    if !bocc_id.is_empty() { body["idMainBocc"] = json!(bocc_id); }
    if let Some(idccs) = args["idccs"].as_array() { body["idccs"] = json!(idccs); }
    let data  = client.post("/list/boccTexts", body).await?;
    let items = data["texts"].as_array()
        .or_else(|| data["results"].as_array())
        .cloned()
        .unwrap_or_default();
    if items.is_empty() { return Ok("Aucun texte BOCC trouvé.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(items.len() as i64);
    let mut lines = vec![format!("**{total} texte(s) BOCC** — {} affiché(s)\n", items.len())];
    for t in &items {
        let titre = t["titre"].as_str().or_else(|| t["title"].as_str()).unwrap_or("?");
        let id    = t["id"].as_str().or_else(|| t["cid"].as_str()).unwrap_or("");
        let idcc  = t["idcc"].as_str().unwrap_or("");
        let mut line = format!("- **{titre}**");
        if !idcc.is_empty() { line.push_str(&format!(" (IDCC {idcc})")); }
        if !id.is_empty()   { line.push_str(&format!(" — `{id}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn tool_lister_boccs_et_textes() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_boccs_et_textes".into(),
        description: "Recherche combinée dans les BOCCs et leurs textes associés.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "idcc":                 { "type": "string", "description": "Numéro IDCC (ex: '0016')" },
                "titre":                { "type": "string", "description": "Texte à rechercher dans les titres" },
                "interval_publication": { "type": "string", "description": "Intervalle de publication" },
                "nb_resultats":         { "type": "integer", "default": 20 }
            }
        }),
    }
}
async fn do_lister_boccs_et_textes(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": 1, "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100) });
    let idcc     = str_arg(args, "idcc");
    let titre    = str_arg(args, "titre");
    let interval = str_arg(args, "interval_publication");
    // CORRECTION vs référence : le schéma officiel attend "idccs" (tableau),
    // pas "idcc" (scalaire) — un champ inconnu fait planter l'API en 500.
    if !idcc.is_empty()     { body["idccs"]               = json!([idcc]); }
    if !titre.is_empty()    { body["titre"]               = json!(titre); }
    if !interval.is_empty() { body["intervalPublication"] = json!(interval); }
    let data  = client.post("/list/boccsAndTexts", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun résultat BOCC/texte trouvé.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(items.len() as i64);
    let mut lines = vec![format!("**{total} résultat(s)** — {} affiché(s)\n", items.len())];
    for r in &items {
        let titre = r["titre"].as_str().or_else(|| r["title"].as_str()).unwrap_or("?");
        let id    = r["id"].as_str().or_else(|| r["cid"].as_str()).unwrap_or("");
        lines.push(format!("- **{titre}**{}", if id.is_empty() { String::new() } else { format!(" — `{id}`") }));
    }
    Ok(lines.join("\n"))
}

fn tool_bocc_pdf_metadata() -> ToolDef {
    ToolDef {
        name: "legifrance_bocc_pdf_metadata".into(),
        description: "Récupère les métadonnées du PDF associé à un texte unitaire du BOCC \
            (Bulletin Officiel des Conventions Collectives).".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "bocc_id":          { "type": "string",  "description": "Identifiant du texte BOCC" },
                "pour_bocc_global": { "type": "boolean", "description": "Si true, récupère le PDF du BOCC global" }
            }
        }),
    }
}
async fn do_bocc_pdf_metadata(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({});
    let bocc_id = str_arg(args, "bocc_id");
    if !bocc_id.is_empty() { body["id"] = json!(bocc_id); }
    if let Some(global) = args["pour_bocc_global"].as_bool() { body["forGlobalBocc"] = json!(global); }
    let data = client.post("/consult/getBoccTextPdfMetadata", body).await?;
    let mut lines = vec!["**Métadonnées PDF BOCC**\n".to_string()];
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            if !v.is_null() {
                let val = match v { Value::String(s) => s.clone(), other => other.to_string() };
                if !val.is_empty() { lines.push(format!("- **{k}** : {val}")); }
            }
        }
    }
    if lines.len() > 1 { Ok(lines.join("\n")) }
    else { Ok("Aucune métadonnée PDF BOCC disponible.".into()) }
}

fn tool_lister_bodmr() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_bodmr".into(),
        description: "Liste les Bulletins Officiels du Ministère des Relations avec le Parlement (BODMR).".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "annee":        { "type": "integer", "description": "Année à filtrer (ex: 2024)" },
                "nb_resultats": { "type": "integer", "default": 20 }
            }
        }),
    }
}
async fn do_lister_bodmr(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": 1, "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100) });
    let annee = int_arg(args, "annee", 0);
    if annee > 0 { body["year"] = json!(annee); }
    let data  = client.post("/list/bodmr", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun BODMR trouvé.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(items.len() as i64);
    let mut lines = vec![format!("**{total} BODMR** — {} affiché(s)\n", items.len())];
    for b in &items {
        let titre   = b["titre"].as_str().or_else(|| b["title"].as_str()).unwrap_or("Sans titre");
        let annee_b = b["annee"].as_i64().or_else(|| b["year"].as_i64()).unwrap_or(0);
        let id      = b["id"].as_str().unwrap_or("");
        let mut line = format!("- **{titre}**");
        if annee_b > 0    { line.push_str(&format!(" ({annee_b})")); }
        if !id.is_empty() { line.push_str(&format!(" — `{id}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── DOCUMENTS ADMINISTRATIFS ──
// ============================================================================

fn tool_lister_docs_admins() -> ToolDef {
    ToolDef {
        name: "legifrance_lister_docs_admins".into(),
        description: "Liste les documents administratifs disponibles dans Légifrance pour une ou plusieurs années.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "annees": { "type": "array", "items": { "type": "integer" },
                    "description": "Liste d'années à rechercher (ex: [2023, 2024])" }
            }
        }),
    }
}
async fn do_lister_docs_admins(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({});
    if let Some(annees) = args["annees"].as_array() { body["years"] = json!(annees); }
    let data  = client.post("/list/docsAdmins", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun document administratif trouvé.".into()); }
    let mut lines = vec![format!("**{} document(s) administratif(s)**\n", items.len())];
    for d in &items {
        let titre = d["titre"].as_str().or_else(|| d["title"].as_str()).unwrap_or("Sans titre");
        let annee = d["annee"].as_i64().or_else(|| d["year"].as_i64()).unwrap_or(0);
        let id    = d["id"].as_str().unwrap_or("");
        let mut line = format!("- **{titre}**");
        if annee > 0      { line.push_str(&format!(" ({annee})")); }
        if !id.is_empty() { line.push_str(&format!(" — `{id}`")); }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── UTILITAIRES ──
// ============================================================================

fn tool_commit_id() -> ToolDef {
    ToolDef {
        name: "legifrance_commit_id".into(),
        description: "Retourne l'identifiant du commit de déploiement courant de l'API Légifrance. \
            Utile pour identifier la version de l'API en production.".into(),
        input_schema: json!({ "type": "object", "properties": {} }),
    }
}
async fn do_commit_id(client: &LegifranceClient) -> Result<String> {
    let data = client.get("/misc/commitId").await?;
    let cid  = data["commitId"].as_str()
        .or_else(|| data["commit_id"].as_str())
        .or_else(|| data["content"].as_str())
        .unwrap_or("(inconnu)");
    Ok(format!("**Commit ID de l'API Légifrance** : `{cid}`"))
}

async fn do_code_toc(args: &Value, client: &LegifranceClient) -> Result<String> {
    let code = require_str(args, "code")?;
    let code_id = resolve_code(code)?;
    let date = date_arg(args, "date", &today());
    // CORRECTION vs référence : /consult/code/tableMatieres attend {textId},
    // pas {sctCid}.
    let body = json!({ "textId": code_id, "date": date });
    let data = client.post("/consult/code/tableMatieres", body).await?;
    Ok(fmt_toc(&data, code))
}

async fn do_code_article(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let date = date_arg(args, "date", &today());
    let body = json!({ "id": id, "date": date });
    let data = client.post("/consult/getArticle", body).await?;
    let article = &data["article"];
    if article.is_null() { return Ok(format!("Article `{id}` introuvable.")); }
    Ok(fmt_article(article))
}

async fn do_code_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    // CORRECTION vs référence : le chemin /consult/code/section n'existe pas.
    // L'endpoint générique correct est /consult/getSectionByCid { cid }.
    let body = json!({ "cid": id });
    let data = client.post("/consult/getSectionByCid", body).await?;
    fmt_section_result(&data, id)
}

async fn do_code_article_by_num(args: &Value, client: &LegifranceClient) -> Result<String> {
    let code   = require_str(args, "code")?;
    let numero = require_str(args, "numero")?;
    let code_id = resolve_code(code)?;
    // CORRECTION vs référence : /consult/code/articleNum n'existe pas.
    // L'endpoint correct est /consult/getArticleWithIdAndNum { id, num }.
    let body   = json!({ "id": code_id, "num": numero });
    let data   = client.post("/consult/getArticleWithIdAndNum", body).await?;
    let article = data["article"].as_array()
        .and_then(|a| a.first())
        .unwrap_or(&data["article"]);
    if article.is_null() { return Ok(format!("Article {numero} introuvable dans le code {code}.")); }
    Ok(fmt_article(article))
}

async fn do_code_article_versions(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/getArticleVersions", body).await?;
    fmt_versions(&data, id)
}

async fn do_code_article_liens(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    // CORRECTION vs référence : chemin /consult/concordanceLinksArticle,
    // body {articleId}, pas /consult/getArticleLinks {id, typeLinks}.
    let body = json!({ "articleId": id });
    let data = client.post("/consult/concordanceLinksArticle", body).await?;
    fmt_liens(&data, id)
}

async fn do_code_list(_args: &Value, client: &LegifranceClient) -> Result<String> {
    let body = json!({ "pageNumber": 1, "pageSize": 200 });
    let data = client.post("/list/code", body).await?;
    let results = data["results"].as_array()
        .or_else(|| data.as_array())
        .cloned()
        .unwrap_or_default();
    if results.is_empty() { return Ok("Aucun code trouvé.".into()); }
    let mut lines = vec![format!("**{} code(s) disponibles**\n", results.len())];
    for r in &results {
        let title = r["title"].as_str().or_else(|| r["titre"].as_str()).unwrap_or("");
        let id    = r["id"].as_str().or_else(|| r["cid"].as_str()).unwrap_or("");
        let short = r["shortTitle"].as_str().unwrap_or("");
        let label = if !short.is_empty() { short } else { title };
        lines.push(format!("- **{label}** — `{id}`"));
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── LOIS & RÈGLEMENTS (LEGI) ──
// ============================================================================

async fn do_legi_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let date = date_arg(args, "date", &today());
    // CORRECTION vs référence : pour le texte complet d'une loi/décret, c'est
    // /consult/legiPart { textId, date } (LegiConsultRequest) qu'il faut
    // appeler — /consult/legi/tableMatieres ne retourne qu'un sommaire, et
    // attend de toute façon {textId} et non {id}.
    let body = json!({ "textId": id, "date": date });
    let data = client.post("/consult/legiPart", body).await?;
    let titre = data["titre"].as_str().or_else(|| data["title"].as_str()).unwrap_or(id);
    let nature = data["nature"].as_str().unwrap_or("");
    let etat   = data["etat"].as_str().unwrap_or("");
    let dv = fmt_date(&data["dateVigueur"]);
    let dp = fmt_date(&data["datePublication"]);
    let mut lines = vec![format!("# {titre}\n")];
    if !nature.is_empty() { lines.push(format!("**Nature** : {nature}")); }
    if !etat.is_empty()   { lines.push(format!("**État** : {etat}")); }
    if !dv.is_empty()     { lines.push(format!("**Date de vigueur** : {dv}")); }
    if !dp.is_empty()     { lines.push(format!("**Date de publication** : {dp}")); }
    lines.push(format!("**ID** : `{id}`\n"));
    lines.push(fmt_toc(&data, titre));
    Ok(lines.join("\n"))
}

async fn do_legi_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    do_code_section(args, client).await
}

async fn do_search_all(args: &Value, client: &LegifranceClient) -> Result<String> {
    let query = require_str(args, "query")?;
    let body  = build_search_body(args, "ALL");
    let data  = client.post("/search", body).await?;
    Ok(fmt_search(&data, query))
}

async fn do_search_suggest(args: &Value, client: &LegifranceClient) -> Result<String> {
    let query = require_str(args, "query")?;
    let fond  = str_arg(args, "fond");
    // CORRECTION vs référence PISTE : l'endpoint POST /suggest attend un
    // SuggestSupplyRequest { searchText, supplies: [...] }, pas { query, fond }.
    let supplies = if fond.is_empty() { vec!["ALL_SUGGEST".to_string()] } else { vec![fond.to_string()] };
    let body  = json!({ "searchText": query, "supplies": supplies });
    let data  = client.post("/suggest", body).await?;
    let results = data["suggestions"].as_array()
        .or_else(|| data["results"].as_array())
        .or_else(|| data.as_array())
        .cloned()
        .unwrap_or_default();
    if results.is_empty() { return Ok(format!("Aucune suggestion pour : « {query} »")); }
    let mut lines = vec![format!("**Suggestions pour « {query} »** :\n")];
    for r in results.iter().take(20) {
        let r_str = r.to_string();
        let title = r["title"].as_str().or_else(|| r["titre"].as_str()).unwrap_or(&r_str);
        let id    = r["id"].as_str().or_else(|| r["cid"].as_str()).unwrap_or("");
        lines.push(format!("- {title}{}",
            if id.is_empty() { String::new() } else { format!(" — `{id}`") }));
    }
    Ok(lines.join("\n"))
}

// ============================================================================
// ── JORF ──
// ============================================================================

async fn do_jorf_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/jorf", body).await?;
    fmt_jorf_text_result(&data, id)
}

async fn do_jorf_article(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/jorf/article", body).await?;
    let article = data.get("article").unwrap_or(&data);
    Ok(fmt_article(article))
}

async fn do_jorf_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/jorf/section", body).await?;
    fmt_section_result(&data, id)
}

async fn do_jorf_jo_list(args: &Value, client: &LegifranceClient) -> Result<String> {
    // CORRECTION : /consult/jorf/listJo n'existe pas dans l'API Légifrance —
    // c'est très probablement la cause des erreurs 403 observées (une route
    // non enregistrée renvoie 403 côté passerelle PISTE plutôt que 404).
    // Le bon point d'entrée pour « les N derniers JO » est /consult/lastNJo,
    // qui prend un simple nbElement (pas de pagination ni de période — pour
    // une période précise, voir legifrance_sommaire_jorf / /consult/jorfCont).
    let nb = int_arg(args, "taille", 10);
    let body = json!({ "nbElement": nb });
    let data = client.post("/consult/lastNJo", body).await?;
    fmt_jo_list(&data)
}

async fn do_kali_list(args: &Value, client: &LegifranceClient) -> Result<String> {
    let body = json!({
        "query":      str_arg(args, "query"),
        "pageNumber": int_arg(args, "page", 1),
        "pageSize":   int_arg(args, "taille", 20)
    });
    let data = client.post("/list/kali", body).await?;
    let results = data["results"].as_array().cloned().unwrap_or_default();
    if results.is_empty() { return Ok("Aucune convention collective trouvée.".into()); }
    let total = data["totalResultNumber"].as_i64().unwrap_or(results.len() as i64);
    let mut lines = vec![format!("**{total} convention(s) collective(s)**\n")];
    for r in &results {
        let titre = r["title"].as_str().or_else(|| r["titre"].as_str()).unwrap_or("?");
        let id    = r["id"].as_str().or_else(|| r["cid"].as_str()).unwrap_or("");
        let idcc  = r["idcc"].as_str().or_else(|| r["numero"].as_str()).unwrap_or("");
        let idcc_str = if !idcc.is_empty() { format!(" (IDCC {idcc})") } else { String::new() };
        lines.push(format!("- **{titre}**{idcc_str}"));
        if !id.is_empty() { lines.push(format!("  `{id}`")); }
    }
    Ok(lines.join("\n"))
}

async fn do_kali_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    // CORRECTION vs référence : le chemin /consult/kali/tableMatieres
    // n'existe pas. Le conteneur d'une convention se consulte via
    // /consult/kaliCont { id } (KaliContConsultRequest, sans date).
    let body = json!({ "id": id });
    let data = client.post("/consult/kaliCont", body).await?;
    let titre = data["titre"].as_str().or_else(|| data["title"].as_str()).unwrap_or(id);
    Ok(fmt_toc(&data, titre))
}

async fn do_kali_article(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    // CORRECTION vs référence : un article KALI se consulte via
    // /consult/kaliArticle { id } (KaliTextConsultArticleRequest), pas
    // /consult/getArticle (générique, réservé aux articles LEGIARTI/JORFARTI).
    let body = json!({ "id": id });
    let data = client.post("/consult/kaliArticle", body).await?;
    let article = data.get("article").unwrap_or(&data);
    Ok(fmt_article(article))
}

async fn do_kali_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    // CORRECTION vs référence : le chemin /consult/kali/section n'existe pas.
    // Une section KALI se consulte via /consult/kaliSection { id }.
    let body = json!({ "id": id });
    let data = client.post("/consult/kaliSection", body).await?;
    fmt_section_result(&data, id)
}

// ============================================================================
// ── CNIL ──
// ============================================================================

async fn do_cnil_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    // CORRECTION vs référence : POST /consult/cnil attend {textId}, pas {id}.
    let body = json!({ "textId": id });
    let data = client.post("/consult/cnil", body).await?;
    fmt_generic_text(&data, id)
}

// ============================================================================
// ── ACCO ──
// ============================================================================

async fn do_acco_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/acco", body).await?;
    fmt_generic_text(&data, id)
}

// ============================================================================
// ── CIRCULAIRES ──
// ============================================================================

async fn do_circ_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let body = json!({ "id": id });
    // CORRECTION vs référence : le chemin réel est /consult/circulaire.
    let data = client.post("/consult/circulaire", body).await?;
    fmt_generic_text(&data, id)
}

// ============================================================================
// ── DOSSIERS LÉGISLATIFS ──
// ============================================================================

async fn do_lister_dossiers_legislatifs(args: &Value, client: &LegifranceClient) -> Result<String> {
    // CORRECTION vs référence : le listing des dossiers législatifs se fait
    // via /list/dossiersLegislatifs { type, legislatureId }, pas via /search
    // avec un fond "DOSSIER_LEGISLATIF" qui n'existe pas dans l'API.
    let legislature_id = int_arg(args, "legislature_id", 0);
    if legislature_id == 0 { anyhow::bail!("legislature_id est obligatoire."); }
    let type_dossier = str_arg(args, "type_dossier");
    let type_dossier = if type_dossier.is_empty() { "LOI_PUBLIEE" } else { type_dossier };
    let body = json!({ "type": type_dossier, "legislatureId": legislature_id });
    let data = client.post("/list/dossiersLegislatifs", body).await?;
    let items = data["results"].as_array()
        .or_else(|| data["dossiers"].as_array())
        .cloned()
        .unwrap_or_default();
    if items.is_empty() { return Ok(format!("Aucun dossier législatif trouvé pour la législature {legislature_id}.")); }
    let mut lines = vec![format!("**{} dossier(s) législatif(s)** — législature {legislature_id}\n", items.len())];
    for d in &items {
        let titre = d["titre"].as_str().or_else(|| d["title"].as_str()).unwrap_or("?");
        let id    = d["id"].as_str().unwrap_or("");
        lines.push(format!("- **{titre}**{}", if id.is_empty() { String::new() } else { format!(" — `{id}`") }));
    }
    Ok(lines.join("\n"))
}

async fn do_dossier_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    let body = json!({ "id": id });
    let data = client.post("/consult/dossierLegislatif", body).await?;
    fmt_generic_text(&data, id)
}

// ============================================================================
// ── JURISPRUDENCE ADMINISTRATIVE (JADE) ──
// ============================================================================

async fn do_jade_search(args: &Value, client: &LegifranceClient) -> Result<String> {
    let query = require_str(args, "query")?;
    let body  = build_search_body(args, "JADE");
    let data  = client.post("/search", body).await?;
    Ok(fmt_search(&data, query))
}

async fn do_jade_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id   = require_str(args, "id")?;
    // CORRECTION vs référence : POST /consult/juri attend {textId}, pas {id}.
    let body = json!({ "textId": id });
    let data = client.post("/consult/juri", body).await?;
    fmt_generic_text(&data, id)
}

// ============================================================================
// ── TABLEAUX ──
// ============================================================================

async fn do_consult_jorf_date(args: &Value, client: &LegifranceClient) -> Result<String> {
    // CORRECTION : /consult/jorf/jo n'existe pas dans l'API Légifrance — le
    // vrai point d'entrée pour consulter le sommaire d'un JO (par date, par
    // identifiant JORFCONT, ou par recherche textuelle) est /consult/jorfCont.
    // C'était très probablement la seconde cause des erreurs 403.
    let date = str_arg(args, "date");
    let date_fin = str_arg(args, "date_fin");
    let jorf_id = str_arg(args, "jorf_id");
    let recherche = str_arg(args, "recherche");

    if date.is_empty() && jorf_id.is_empty() {
        anyhow::bail!("Fournissez au moins 'date' (YYYY-MM-DD) ou 'jorf_id'.");
    }

    let mut body = json!({
        "pageNumber": 1,
        "pageSize": int_arg(args, "nb_resultats", 20),
    });
    if !jorf_id.is_empty() {
        body["id"] = json!(jorf_id);
    } else if !date_fin.is_empty() {
        // Période : l'API attend deux objets {dayOfMonth, month, year}.
        body["start"] = json!(date_parts(date));
        body["end"]   = json!(date_parts(&date_fin));
    } else {
        // Date unique : champ "date" au format date-time ISO 8601.
        body["date"] = json!(to_iso_datetime(date));
    }
    if !recherche.is_empty() {
        body["searchText"] = json!(recherche);
    }

    let data = client.post("/consult/jorfCont", body).await?;
    let label = if !date.is_empty() { date } else { jorf_id };
    fmt_jorf_cont(&data, label)
}

/// Découpe une date "YYYY-MM-DD" en {dayOfMonth, month, year} attendu par
/// les champs start/end de /consult/jorfCont.
fn date_parts(date: &str) -> Value {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
            return json!({ "year": y, "month": m, "dayOfMonth": d });
        }
    }
    json!({})
}

/// Complète une date "YYYY-MM-DD" en date-time ISO 8601 si nécessaire
/// (l'API Légifrance type ses champs date en date-time, pas en simple date).
fn to_iso_datetime(date: &str) -> String {
    if date.contains('T') { date.to_string() } else { format!("{date}T00:00:00Z") }
}

fn fmt_section_result(data: &Value, id: &str) -> Result<String> {
    let titre = data["titre"].as_str().or_else(|| data["title"].as_str()).unwrap_or(id);
    let mut lines = vec![format!("## {titre}\n")];
    let articles = data["articles"].as_array().cloned().unwrap_or_default();
    if articles.is_empty() {
        lines.push(fmt_toc(data, titre));
    } else {
        for a in &articles {
            lines.push(fmt_article(a));
            lines.push(String::new());
        }
    }
    Ok(lines.join("\n"))
}

fn fmt_versions(data: &Value, id: &str) -> Result<String> {
    let versions = data["versions"].as_array()
        .or_else(|| data.as_array())
        .cloned()
        .unwrap_or_default();
    if versions.is_empty() { return Ok(format!("Aucune version trouvée pour `{id}`.")); }
    let mut lines = vec![format!("**{} version(s) de `{id}`**\n", versions.len())];
    for (i, v) in versions.iter().enumerate() {
        let debut = fmt_date(&v["dateDebut"]);
        let fin   = fmt_date(&v["dateFin"]);
        let etat  = v["etat"].as_str().unwrap_or("");
        let vid   = v["id"].as_str().unwrap_or("");
        let range = match (debut.is_empty(), fin.is_empty()) {
            (false, false) => format!("{debut} → {fin}"),
            (false, true)  => format!("{debut} → en vigueur"),
            _              => String::new(),
        };
        lines.push(format!("{}. {range} — *{etat}* — `{vid}`", i + 1));
    }
    Ok(lines.join("\n"))
}

fn fmt_liens(data: &Value, id: &str) -> Result<String> {
    let liens = data["links"].as_array()
        .or_else(|| data["liens"].as_array())
        .cloned()
        .unwrap_or_default();
    if liens.is_empty() { return Ok(format!("Aucun lien trouvé pour `{id}`.")); }
    let mut lines = vec![format!("**{} lien(s) pour `{id}`**\n", liens.len())];
    for l in liens.iter().take(50) {
        let ltype   = l["typeLink"].as_str().or_else(|| l["typeLien"].as_str()).unwrap_or("");
        let titre   = l["title"].as_str().or_else(|| l["titre"].as_str()).unwrap_or("");
        let lid     = l["id"].as_str().unwrap_or("");
        let num     = l["num"].as_str().unwrap_or("");
        let label   = if !titre.is_empty() { titre } else { num };
        lines.push(format!("- [{ltype}] {label} — `{lid}`"));
    }
    if liens.len() > 50 { lines.push(format!("*… {} liens supplémentaires*", liens.len() - 50)); }
    Ok(lines.join("\n"))
}

fn fmt_jorf_text_result(data: &Value, id: &str) -> Result<String> {
    let titre = data["titre"].as_str()
        .or_else(|| data["title"].as_str())
        .or_else(|| data["titreTxt"].as_str())
        .unwrap_or(id);
    let nature = data["nature"].as_str().unwrap_or("");
    let date_pub = fmt_date(&data["datePublication"]);
    let date_sig = fmt_date(&data["dateSignature"]);
    let jorf_num = data["numJO"].as_str().or_else(|| data["numParution"].as_str()).unwrap_or("");

    let mut lines = vec![format!("# {titre}\n")];
    if !nature.is_empty()   { lines.push(format!("**Nature** : {nature}")); }
    if !date_pub.is_empty() { lines.push(format!("**Date de publication** : {date_pub}")); }
    if !date_sig.is_empty() { lines.push(format!("**Date de signature** : {date_sig}")); }
    if !jorf_num.is_empty() { lines.push(format!("**Numéro JORF** : {jorf_num}")); }
    lines.push(format!("**ID** : `{id}`\n"));

    // Articles dans le texte
    let articles = data["articles"].as_array().cloned().unwrap_or_default();
    if !articles.is_empty() {
        lines.push(format!("\n**{} article(s)**\n", articles.len()));
        for a in articles.iter().take(10) {
            lines.push(fmt_article(a));
            lines.push(String::new());
        }
        if articles.len() > 10 {
            lines.push(format!("*… {} article(s) supplémentaire(s)*", articles.len() - 10));
        }
    } else {
        // Essayer le contenu brut
        let raw = data["texte"].as_str().or_else(|| data["content"].as_str()).unwrap_or("");
        if !raw.is_empty() {
            let texte = strip_html(raw);
            let excerpt = if texte.len() > 4000 { &texte[..4000] } else { &texte };
            lines.push(format!("\n{excerpt}"));
            if texte.len() > 4000 { lines.push("*[Texte tronqué]*".into()); }
        }
    }
    Ok(lines.join("\n"))
}

fn fmt_jo_list(data: &Value) -> Result<String> {
    // Réponse de /consult/lastNJo : { "containers": [...], "totalNbResult": N }
    let results = data["containers"].as_array().cloned().unwrap_or_default();
    if results.is_empty() { return Ok("Aucun Journal Officiel trouvé.".into()); }
    let total = data["totalNbResult"].as_i64().unwrap_or(results.len() as i64);
    let mut lines = vec![format!("**{total} Journal(ux) Officiel(s)**\n")];
    for r in &results {
        let date = fmt_date(&r["datePubli"]);
        let num  = r["num"].as_str().or_else(|| r["numero"].as_str()).unwrap_or("");
        let id   = r["id"].as_str().unwrap_or("");
        lines.push(format!("- **JO n°{num}** du {date} — `{id}`"));
    }
    Ok(lines.join("\n"))
}

fn fmt_jorf_cont(data: &Value, label: &str) -> Result<String> {
    // Réponse réelle de /consult/jorfCont :
    // { "items": [ { "joCont": { "titre", "num", "datePubli",
    //     "structure": { "tms": [ { "titre", "liensTxt": [...], "tms": [...] } ] } } } ],
    //   "totalNbResult": N }
    let items = data["items"].as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return Ok(format!("Aucun sommaire trouvé pour « {label} ». \
            (Vérifiez le format de date YYYY-MM-DD, ou qu'un JO existe bien à cette date — \
            voir legifrance_dates_sans_jo.)"));
    }

    let mut lines = Vec::new();
    for item in &items {
        let jo = &item["joCont"];
        let titre = jo["titre"].as_str().unwrap_or(label);
        let date  = fmt_date(&jo["datePubli"]);
        lines.push(format!("# {titre}"));
        if !date.is_empty() { lines.push(format!("*Publié le {date}*\n")); }

        let mut textes = Vec::new();
        if let Some(tms) = jo["structure"]["tms"].as_array() {
            collect_liens_txt(tms, &mut textes, 0);
        }
        if textes.is_empty() {
            lines.push("*(Aucun texte listé dans le sommaire de cette édition.)*".into());
        } else {
            lines.push(format!("**{} texte(s) au sommaire**\n", textes.len()));
            lines.extend(textes);
        }
        lines.push(String::new());
    }
    Ok(lines.join("\n"))
}

/// Parcourt récursivement l'arbre structure.tms d'un joCont pour en extraire
/// tous les textes listés (structure.tms[].liensTxt[], avec sous-sections
/// structure.tms[].tms[] imbriquées par ministère/rubrique).
fn collect_liens_txt(tms: &[Value], out: &mut Vec<String>, depth: usize) {
    if depth > 4 { return; } // garde-fou anti-récursion excessive
    for section in tms {
        let section_titre = section["titre"].as_str().unwrap_or("");
        if let Some(liens) = section["liensTxt"].as_array() {
            if !liens.is_empty() && !section_titre.is_empty() {
                let indent = "  ".repeat(depth);
                out.push(format!("{indent}**{section_titre}**"));
            }
            for lien in liens {
                let titre = lien["titre"].as_str().unwrap_or("?");
                let id    = lien["id"].as_str().unwrap_or("");
                let indent = "  ".repeat(depth + 1);
                if !id.is_empty() {
                    out.push(format!("{indent}- {titre} (`{id}`)"));
                } else {
                    out.push(format!("{indent}- {titre}"));
                }
            }
        }
        if let Some(sub) = section["tms"].as_array() {
            collect_liens_txt(sub, out, depth + 1);
        }
    }
}

fn fmt_generic_text(data: &Value, id: &str) -> Result<String> {
    // Essayer plusieurs structures possibles
    if let Some(article) = data.get("article") {
        if !article.is_null() { return Ok(fmt_article(article)); }
    }
    let titre = data["titre"].as_str()
        .or_else(|| data["title"].as_str())
        .or_else(|| data["titreTxt"].as_str())
        .unwrap_or(id);
    let raw = data["texteHtml"].as_str()
        .or_else(|| data["texte"].as_str())
        .or_else(|| data["content"].as_str())
        .or_else(|| data["texteIntegral"].as_str())
        .unwrap_or("");
    let texte = strip_html(raw);
    let mut lines = vec![format!("# {titre}\n")];
    let date_pub = fmt_date(&data["datePublication"]);
    let date_sig = fmt_date(&data["dateSignature"]);
    let nature   = data["nature"].as_str().unwrap_or("");
    if !nature.is_empty()   { lines.push(format!("**Nature** : {nature}")); }
    if !date_pub.is_empty() { lines.push(format!("**Date de publication** : {date_pub}")); }
    if !date_sig.is_empty() { lines.push(format!("**Date de signature** : {date_sig}")); }
    lines.push(format!("**ID** : `{id}`\n"));
    if !texte.is_empty() {
        let excerpt = if texte.len() > 6000 { &texte[..6000] } else { &texte };
        lines.push(excerpt.to_string());
        if texte.len() > 6000 { lines.push("\n*[Texte tronqué à 6000 caractères]*".into()); }
    } else if let Some(sections) = data["sections"].as_array() {
        if !sections.is_empty() { lines.push(fmt_toc(data, titre)); }
    }
    Ok(lines.join("\n"))
}
