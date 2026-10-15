// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================

use crate::client::{code_title, resolve_code, LegifranceClient};
use crate::fmt::*;
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
    v.push(tool_ping());
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
        "legifrance_ping"                          => do_ping(client).await,
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
        "query":        { "type": "string",  "description": "Termes de recherche" },
        "nb_resultats": { "type": "integer", "minimum": 1, "maximum": 100, "default": 5 },
        "page":         { "type": "integer", "minimum": 1, "default": 1, "description": "Numéro de page" },
        "fond":         { "type": "string",  "description": "ALL (défaut), CODE_DATE, CODE_ETAT, LODA_DATE, LODA_ETAT, JORF, KALI, CIRC, ACCO, CNIL, CONSTIT, CETAT (jurisprudence administrative), JURI (judiciaire), JUFI (financière). Alias acceptés : JADE→CETAT, CODE→CODE_DATE, LODA→LODA_DATE." },
        "date_debut":   { "type": "string",  "description": "Date début YYYY-MM-DD (pour CODE_DATE : date de version recherchée)" },
        "date_fin":     { "type": "string",  "description": "Date fin YYYY-MM-DD" },
        "tri":          { "type": "string",  "description": "PERTINENCE (défaut), DATE_DESC, DATE_ASC — traduits selon le fond ; les tris natifs du fond (ex. SIGNATURE_DATE_DESC) sont aussi acceptés" },
        "code":         { "type": "string",  "description": "Restreindre à un code (ex: 'civil', 'travail', 'Code de la route'). Bascule automatiquement sur le fond CODE_DATE si fond=ALL." },
        "nature":       { "type": "string",  "description": "Nature du texte (LOI, DECRET, ORDONNANCE, ARRETE…) — fonds JORF, LODA_DATE, LODA_ETAT uniquement" },
        "statut":       { "type": "string",  "description": "État juridique (VIGUEUR, ABROGE…) — fonds LODA_ETAT, CODE_ETAT, KALI" },
        "type_champ":   { "type": "string",  "description": "Champ visé : ALL (défaut), TITLE, ARTICLE, NUM_ARTICLE, NUM, NOR, TEXTE, IDCC, NUM_AFFAIRE, ECLI… (voir doc API)" },
        "type_recherche": { "type": "string", "description": "UN_DES_MOTS (défaut), TOUS_LES_MOTS_DANS_UN_CHAMP, EXACTE, AUCUN_DES_MOTS, AUCUNE_CORRESPONDANCE_A_CETTE_EXPRESSION" },
        "proximite":    { "type": "integer", "description": "Distance maximale en mots entre les termes (optionnel)" }
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
        "CODE_ETAT"                          => None,
        "CODE_DATE"                          => Some("DATE_VERSION"),
        "LODA_ETAT"                          => Some("TEXT_DATE_SIGNATURE"),
        "CONSTIT" | "CETAT" | "JURI" | "JUFI" => Some("DATE_DECISION"),
        "CNIL"                               => Some("DATE_DELIB"),
        // JORF, LODA_DATE, CIRC, KALI, ACCO
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

const FONDS: &[&str] = &[
    "JORF", "CNIL", "CETAT", "JURI", "JUFI", "CONSTIT", "KALI", "CODE_DATE", "CODE_ETAT",
    "LODA_DATE", "LODA_ETAT", "ALL", "CIRC", "ACCO",
];
const TYPES_CHAMP: &[&str] = &[
    "ALL", "TITLE", "TABLE", "NOR", "NUM", "ADVANCED_TEXTE_ID", "NUM_DELIB", "NUM_DEC", "NUM_ARTICLE",
    "ARTICLE", "MINISTERE", "VISA", "NOTICE", "VISA_NOTICE", "TRAVAUX_PREP", "SIGNATURE", "NOTA",
    "NUM_AFFAIRE", "ABSTRATS", "RESUMES", "TEXTE", "ECLI", "NUM_LOI_DEF", "TYPE_DECISION",
    "NUMERO_INTERNE", "REF_PUBLI", "RESUME_CIRC", "TEXTE_REF", "TITRE_LOI_DEF", "RAISON_SOCIALE",
    "MOTS_CLES", "IDCC",
];
const TYPES_RECHERCHE: &[&str] = &[
    "UN_DES_MOTS", "EXACTE", "TOUS_LES_MOTS_DANS_UN_CHAMP", "AUCUN_DES_MOTS",
    "AUCUNE_CORRESPONDANCE_A_CETTE_EXPRESSION",
];

/// Valide le fond contre l'enum du swagger (SearchRequestDTO.fond) et traduit
/// les alias courants. « JADE » ou « DOSSIER_LEGISLATIF » ne sont PAS des
/// valeurs valides de /search (→ HTTP 400).
fn norm_fond(f: &str) -> Result<String> {
    let u = f.trim().to_uppercase().replace(['-', ' '], "_");
    if u.is_empty() { return Ok("ALL".into()); }
    let m = match u.as_str() {
        "JADE" | "CE" | "CONSEIL_ETAT" => "CETAT",
        "CODE" | "CODES" => "CODE_DATE",
        "LODA" | "LEGI" => "LODA_DATE",
        "CASS" | "JUDICIAIRE" => "JURI",
        "CONSTITUTIONNEL" => "CONSTIT",
        "CIRCULAIRE" | "CIRCULAIRES" => "CIRC",
        other => other,
    };
    if FONDS.contains(&m) { return Ok(m.to_string()); }
    anyhow::bail!(
        "Fond '{f}' non pris en charge par /search. Valeurs valides : {}. \
         Dossiers législatifs, débats, questions écrites, BOCC, BODMR et documents \
         administratifs ont leurs propres outils legifrance_lister_*.",
        FONDS.join(", ")
    )
}

/// Tris valides par fond (source : description-des-tris-et-filtres-de-l-api.xlsx).
fn valid_sorts(fond: &str) -> &'static [&'static str] {
    match fond {
        "JORF" | "LODA_DATE" | "LODA_ETAT" => &["PERTINENCE", "SIGNATURE_DATE_DESC", "SIGNATURE_DATE_ASC", "PUBLICATION_DATE_DESC", "PUBLICATION_DATE_ASC"],
        "CIRC" => &["PERTINENCE", "SIGNATURE_DATE_DESC", "SIGNATURE_DATE_ASC", "PUBLI_DATE_DESC", "PUBLI_DATE_ASC"],
        "JURI" | "JUFI" | "CETAT" | "CONSTIT" | "ACCO" => &["PERTINENCE", "DATE_DESC", "DATE_ASC"],
        "KALI" => &["PERTINENCE", "MODIFICATION_DATE_DESC", "SIGNATURE_DATE_DESC", "SIGNATURE_DATE_ASC"],
        "CNIL" => &["PERTINENCE", "DATE_DECISION_DESC", "DATE_DECISION_ASC"],
        _ => &["PERTINENCE"],
    }
}

/// Traduit le tri générique (PERTINENCE / DATE_DESC / DATE_ASC) vers la valeur
/// native du fond ; repli sur PERTINENCE (le champ `sort` est obligatoire).
fn sort_for_fond(fond: &str, tri: &str) -> String {
    let t = tri.trim().to_uppercase();
    if t.is_empty() { return "PERTINENCE".into(); }
    let valid = valid_sorts(fond);
    if valid.contains(&t.as_str()) { return t; }
    let desc = t == "DATE_DESC";
    let asc = t == "DATE_ASC";
    if !desc && !asc { return "PERTINENCE".into(); }
    let dir = if desc { "DESC" } else { "ASC" };
    let cand = match fond {
        "JORF" | "LODA_DATE" | "LODA_ETAT" | "CIRC" | "KALI" => format!("SIGNATURE_DATE_{dir}"),
        "CNIL" => format!("DATE_DECISION_{dir}"),
        _ => format!("DATE_{dir}"),
    };
    if valid.contains(&cand.as_str()) { cand } else { "PERTINENCE".into() }
}

fn build_search_body(args: &Value, default_fond: &str) -> Result<Value> {
    let f = str_arg(args, "fond");
    let mut fond = norm_fond(if f.is_empty() { default_fond } else { f })?;
    let dd = str_arg(args, "date_debut");
    let df = str_arg(args, "date_fin");

    // Filtre « code » : l'API attend le NOM du code (valeurs: ["Code civil"]).
    let code = str_arg(args, "code").trim().to_string();
    let mut code_name: Option<String> = None;
    if !code.is_empty() {
        let name = match code_title(&code) {
            Some(t) => t.to_string(),
            None if code.starts_with("LEGITEXT") => anyhow::bail!(
                "Le filtre 'code' attend un nom de code (ex: 'civil' ou 'Code de la route'), pas un ID LEGITEXT."),
            None => code.clone(),
        };
        if fond == "ALL" { fond = "CODE_DATE".into(); }
        if fond == "CODE_DATE" || fond == "CODE_ETAT" { code_name = Some(name); }
    }

    let type_champ = { let c = str_arg(args, "type_champ").to_uppercase(); if c.is_empty() { "ALL".to_string() } else { c } };
    if !TYPES_CHAMP.contains(&type_champ.as_str()) {
        anyhow::bail!("type_champ '{type_champ}' invalide. Valeurs : {}", TYPES_CHAMP.join(", "));
    }
    let type_rech = { let c = str_arg(args, "type_recherche").to_uppercase(); if c.is_empty() { "UN_DES_MOTS".to_string() } else { c } };
    if !TYPES_RECHERCHE.contains(&type_rech.as_str()) {
        anyhow::bail!("type_recherche '{type_rech}' invalide. Valeurs : {}", TYPES_RECHERCHE.join(", "));
    }
    let mut critere = json!({ "typeRecherche": type_rech, "valeur": str_arg(args, "query"), "operateur": "ET" });
    let prox = int_arg(args, "proximite", 0);
    if prox > 0 { critere["proximite"] = json!(prox); }

    let mut filtres: Vec<Value> = Vec::new();
    if let Some(name) = &code_name {
        let facette = if fond == "CODE_ETAT" { "TEXT_NOM_CODE" } else { "NOM_CODE" };
        filtres.push(json!({ "facette": facette, "valeurs": [name] }));
    }
    // Facette de date selon le fond. DATE_VERSION (CODE_DATE) est une date
    // unique (singleDate) ; les autres sont des périodes (dates.start/end).
    if fond == "CODE_DATE" {
        let d = if !dd.is_empty() { dd } else if !df.is_empty() { df } else { "" };
        let d = if d.is_empty() { today() } else { d.to_string() };
        filtres.push(json!({ "facette": "DATE_VERSION", "singleDate": d }));
    } else if (!dd.is_empty() || !df.is_empty()) && fond != "ALL" {
        if let Some(facette) = date_facette(&fond) {
            let mut dates = json!({});
            if !dd.is_empty() { dates["start"] = json!(dd); }
            if !df.is_empty() { dates["end"] = json!(df); }
            filtres.push(json!({ "facette": facette, "dates": dates }));
        }
    }
    let nature = str_arg(args, "nature");
    if !nature.is_empty() {
        if let Some(facette) = nature_facette(&fond) {
            filtres.push(json!({ "facette": facette, "valeurs": [nature.to_uppercase()] }));
        }
    }
    let statut = str_arg(args, "statut");
    if !statut.is_empty() {
        let facette = match fond.as_str() {
            "LODA_ETAT" | "CODE_ETAT" => Some("TEXT_LEGAL_STATUS"),
            "KALI" => Some("LEGAL_STATUS"),
            _ => None,
        };
        if let Some(fa) = facette {
            filtres.push(json!({ "facette": fa, "valeurs": [statut.to_uppercase()] }));
        }
    }

    Ok(json!({
        "fond": fond,
        "recherche": {
            "champs": [{ "typeChamp": type_champ, "criteres": [critere], "operateur": "ET" }],
            "operateur": "ET",
            "pageNumber": int_arg(args, "page", 1).max(1),
            "pageSize":   int_arg(args, "nb_resultats", 5).clamp(1, 100),
            "sort":       sort_for_fond(&fond, str_arg(args, "tri")),
            "typePagination": if type_champ == "NUM_ARTICLE" { "ARTICLE" } else { "DEFAUT" },
            "fromAdvancedRecherche": false,
            "filtres": filtres
        }
    }))
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
    let query = require_any(args, &["query", "texte"])?;
    let data = client.post("/suggest/acco", json!({ "searchText": query })).await?;
    Ok(fmt_suggest(&data, query, "d'accords"))
}

fn tool_suggerer_pdc() -> ToolDef {
    mk("legifrance_suggerer_pdc",
       "Suggestions d'entrées du plan de classement (jurisprudence, etc.) à partir d'un texte partiel.",
       json!({ "type": "object", "required": ["query"], "properties": {
           "query":  { "type": "string", "description": "Texte partiel" },
           "origin": { "type": "string", "description": "Origine du plan de classement (optionnel)" },
           "fond":   { "type": "string", "description": "Fond concerné (optionnel, ex. CETAT, JURI)" }
       }}))
}
async fn do_suggerer_pdc(args: &Value, client: &LegifranceClient) -> Result<String> {
    let query = require_str(args, "query")?;
    let mut body = json!({ "searchText": query });
    for k in ["origin", "fond"] {
        let v = str_arg(args, k);
        if !v.is_empty() { body[k] = json!(if k == "fond" { v.to_uppercase() } else { v.to_string() }); }
    }
    let data = client.post("/suggest/pdc", body).await?;
    Ok(fmt_suggest(&data, query, "du plan de classement"))
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
    mk("legifrance_consulter_code",
       "Table des matières d'un code (parties, livres, titres, chapitres, sections). \
        Les identifiants de section retournés s'utilisent avec legifrance_section_par_cid.",
       json!({ "type": "object", "required": ["code"], "properties": {
           "code": { "type": "string", "description": "Nom court (civil, penal, travail, commerce, consommation…) ou ID LEGITEXT ; tout autre nom de code est résolu automatiquement" },
           "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD (défaut : aujourd'hui)" },
           "profondeur": { "type": "integer", "minimum": 1, "maximum": 10, "default": 4, "description": "Niveaux d'arborescence affichés" },
           "inclure_articles": { "type": "boolean", "default": false, "description": "Lister aussi les articles (numéro + ID)" }
       }}))
}
fn tool_code_complet() -> ToolDef {
    mk("legifrance_code_complet",
       "Consulte un code via /consult/code. Sans section_cid : structure du code. \
        Avec section_cid : contenu de la section (articles inclus), dans la limite d'environ 40 000 caractères.",
       json!({ "type": "object", "required": ["code"], "properties": {
           "code": { "type": "string", "description": "Nom court ou ID LEGITEXT" },
           "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD (défaut : aujourd'hui)" },
           "section_cid": { "type": "string", "description": "ID de section (LEGISCTA…) dont on veut le contenu" },
           "abrogees": { "type": "boolean", "description": "Inclure les parties abrogées" }
       }}))
}
async fn do_code_complet(args: &Value, client: &LegifranceClient) -> Result<String> {
    let code = require_str(args, "code")?;
    let text_id = resolve_code_dyn(client, code).await?;
    let mut body = json!({ "textId": text_id, "date": date_arg(args, "date", &today()) });
    let sct = str_arg(args, "section_cid");
    if !sct.is_empty() { body["sctCid"] = json!(sct); }
    if args["abrogees"].as_bool().unwrap_or(false) { body["abrogated"] = json!(true); }
    let data = client.post("/consult/code", body).await?;
    Ok(fmt_legi_text(&data, &text_id))
}

fn tool_code_par_ancien_id() -> ToolDef {
    mk("legifrance_code_par_ancien_id",
       "Récupère un code à partir de son ancien identifiant (ancien ID Légifrance).",
       json!({ "type": "object", "required": ["ancien_id"], "properties": {
           "ancien_id": { "type": "string", "description": "Ancien identifiant du code" }
       }}))
}
async fn do_code_par_ancien_id(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["ancien_id", "id"])?;
    let data = client.post("/consult/getCodeWithAncienId", json!({ "ancienId": id })).await?;
    Ok(fmt_toc(&data, id))
}

fn tool_section_par_cid() -> ToolDef {
    mk("legifrance_section_par_cid",
       "Contenu d'une section d'un texte (code, loi) : sous-sections et articles, à partir de son identifiant (LEGISCTA…).",
       json!({ "type": "object", "required": ["id"], "properties": {
           "id": { "type": "string", "description": "Identifiant de section (LEGISCTA…)" }
       }}))
}
fn tool_obtenir_article() -> ToolDef {
    mk("legifrance_obtenir_article",
       "Contenu d'un article à partir de son identifiant (LEGIARTI…, JORFARTI…, KALIARTI…). \
        Retourne la version désignée par l'ID ; voir legifrance_versions_article pour l'historique.",
       json!({ "type": "object", "required": ["id"], "properties": {
           "id": { "type": "string", "description": "Identifiant de l'article" }
       }}))
}
fn tool_article_par_numero() -> ToolDef {
    mk("legifrance_article_par_numero",
       "Récupère un article d'un code par son numéro (ex. 1240, L1234-1, R. 123-4). \
        Sans date : version en vigueur. Avec date : version en vigueur à cette date.",
       json!({ "type": "object", "required": ["code", "numero"], "properties": {
           "code":   { "type": "string", "description": "Nom court (civil, penal, travail…), nom complet ou ID LEGITEXT" },
           "numero": { "type": "string", "description": "Numéro de l'article" },
           "date":   { "type": "string", "description": "Date YYYY-MM-DD (optionnel) : version applicable à cette date" }
       }}))
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
    mk("legifrance_articles_meme_numero",
       "Articles portant le même numéro dans d'autres textes/codes (/consult/sameNumArticle). \
        Fournir soit (code + numero), soit (article_cid + text_cid + numero).",
       json!({ "type": "object", "properties": {
           "code":        { "type": "string", "description": "Code de l'article (nom court ou LEGITEXT)" },
           "numero":      { "type": "string", "description": "Numéro de l'article" },
           "article_cid": { "type": "string", "description": "CID de l'article (alternative à code)" },
           "text_cid":    { "type": "string", "description": "CID du texte (alternative à code)" },
           "date":        { "type": "string", "description": "Date YYYY-MM-DD (défaut : aujourd'hui)" }
       }}))
}
async fn do_articles_meme_numero(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut art_cid = str_arg(args, "article_cid").to_string();
    let mut text_cid = str_arg(args, "text_cid").to_string();
    let mut num = normalize_article_num(str_arg(args, "numero"));
    if art_cid.is_empty() || text_cid.is_empty() {
        let code = require_str(args, "code")?;
        if num.is_empty() { anyhow::bail!("Paramètre 'numero' obligatoire."); }
        let id = resolve_code_dyn(client, code).await?;
        let d = client.post("/consult/getArticleWithIdAndNum", json!({ "id": id, "num": num })).await?;
        let a = d.get("article").unwrap_or(&d);
        let c = s(a, "cid");
        art_cid = if c.is_empty() { s(a, "id").to_string() } else { c.to_string() };
        text_cid = { let x = s(a, "cidTexte"); if x.is_empty() { id } else { x.to_string() } };
        if num.is_empty() { num = s(a, "num").to_string(); }
        if art_cid.is_empty() { anyhow::bail!("Article introuvable."); }
    }
    if num.is_empty() { anyhow::bail!("Paramètre 'numero' obligatoire."); }
    let body = json!({ "date": date_arg(args, "date", &today()), "articleCid": art_cid, "textCid": text_cid, "articleNum": num });
    let data = client.post("/consult/sameNumArticle", body).await?;
    Ok(fmt_revision_links(&data, &art_cid, "Articles de même numéro"))
}

fn tool_versions_article() -> ToolDef {
    mk("legifrance_versions_article",
       "Historique des versions d'un article (dates de début/fin, état, ID de chaque version).",
       json!({ "type": "object", "required": ["id"], "properties": {
           "id": { "type": "string", "description": "Identifiant (CID ou ID) de l'article" }
       }}))
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
    let id = require_any(args, &["id", "article_id"])?;
    let data = client.post("/search/canonicalArticleVersion", json!({ "id": id })).await?;
    let vs = data["articleVersions"].as_array().cloned().unwrap_or_default();
    Ok(fmt_article_versions(&vs, id))
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
    let date = require_str(args, "date")?;
    let mut body = json!({ "cidText": cid_text, "date": date });
    let cid_section = str_arg(args, "cid_section");
    if !cid_section.is_empty() { body["cidSection"] = json!(cid_section); }
    let data = client.post("/search/canonicalVersion", body).await?;
    let lines = kv_lines(&data);
    if lines.is_empty() { return Ok("Aucune information de version canonique trouvée.".into()); }
    Ok(format!("**Version canonique de `{cid_text}` au {date}**\n\n{}", lines.join("\n")))
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
    let date = require_str(args, "date")?;
    let mut body = json!({ "cidText": cid_text, "date": date });
    let cid_section = str_arg(args, "cid_section");
    if !cid_section.is_empty() { body["cidSection"] = json!(cid_section); }
    let data = client.post("/search/nearestVersion", body).await?;
    let lines = kv_lines(&data);
    if lines.is_empty() { return Ok("Aucune version proche trouvée.".into()); }
    Ok(format!("**Version la plus proche de `{cid_text}` autour du {date}**\n\n{}", lines.join("\n")))
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
    let id = require_any(args, &["id", "text_cid"])?;
    let data = client.get(&format!("/chrono/textCid/{id}")).await?;
    let has = data["chronolegiExists"].as_bool().unwrap_or(false);
    Ok(if has { format!("✅ `{id}` possède un historique de versions (voir legifrance_historique_texte).") }
       else { format!("❌ `{id}` n'a pas d'historique de versions.") })
}

fn tool_liens_concordance() -> ToolDef {
    mk("legifrance_liens_concordance",
       "Liens de concordance d'un article (anciens/nouveaux textes et numéros — ex. recodification).",
       json!({ "type": "object", "required": ["id"], "properties": {
           "id": { "type": "string", "description": "Identifiant de l'article (LEGIARTI…)" }
       }}))
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
    let id = require_any(args, &["id", "article_id"])?;
    let data = client.post("/consult/relatedLinksArticle", json!({ "articleId": id })).await?;
    let mut lines = vec![format!("**Liens relatifs de `{id}`**\n")];
    let mut n = 0;
    for (key, label) in [("liensCitePar", "Cité par"), ("liensCite", "Cite"), ("liensConcordeAvec", "Concorde avec")] {
        if let Some(arr) = data[key].as_array() {
            if arr.is_empty() { continue; }
            lines.push(format!("**{label}** ({})", arr.len()));
            for l in arr.iter().take(60) {
                n += 1;
                let name = ["name", "titre", "title"].iter().map(|k| s(l, k)).find(|x| !x.is_empty()).unwrap_or("");
                let nat = s(l, "nature");
                lines.push(format!("- {name}{} — `{}`", if nat.is_empty() { String::new() } else { format!(" [{nat}]") }, ["id", "cidText"].iter().map(|k| s(l, k)).find(|x| !x.is_empty()).unwrap_or("")));
            }
            lines.push(String::new());
        }
    }
    if n == 0 {
        let kv = kv_lines(&data);
        if kv.is_empty() { return Ok(format!("Aucun lien relatif pour `{id}`.")); }
        lines.extend(kv);
    }
    Ok(lines.join("\n"))
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
    let id = require_any(args, &["id", "article_cid", "article_id"])?;
    let mut body = json!({ "articleCid": id });
    let fond = str_arg(args, "fond");
    if !fond.is_empty() { body["fond"] = json!(fond.to_uppercase()); }
    let data = client.post("/consult/servicePublicLinksArticle", body).await?;
    let sp = data["liensSP"].as_object().cloned().unwrap_or_default();
    if sp.is_empty() { return Ok(format!("Aucun lien service-public.fr pour `{id}`.")); }
    let mut lines = vec![format!("**Liens service-public.fr pour `{id}`**\n")];
    for (k, v) in &sp {
        match v {
            Value::Array(a) => for x in a { lines.push(format!("- {} — {}", k, x.as_str().map(|s| s.to_string()).unwrap_or_else(|| x.to_string()))); },
            other => lines.push(format!("- {k} — {}", other.as_str().map(|s| s.to_string()).unwrap_or_else(|| other.to_string()))),
        }
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
    let id = require_any(args, &["id", "article_cid", "article_id"])?;
    // Le corps de cette opération est un TABLEAU de CID ; la réponse liste ceux qui ont des liens.
    let data = client.post("/consult/hasServicePublicLinksArticle", json!([id])).await?;
    let has = data.as_array().map(|a| a.iter().any(|x| x.as_str() == Some(id))).unwrap_or(false);
    Ok(if has { format!("✅ `{id}` a des liens service-public.fr.") } else { format!("❌ `{id}` n'a pas de lien service-public.fr.") })
}

// ============================================================================
// ── LOIS & RÈGLEMENTS (LODA / LEGI) ──
// ============================================================================

fn tool_loi_decret() -> ToolDef {
    mk("legifrance_loi_decret",
       "Texte complet d'une loi, ordonnance, décret ou arrêté (LEGITEXT… ou JORFTEXT…) : articles et sections avec leur contenu. \
        Pour un texte JORFTEXT la date est ignorée.",
       json!({ "type": "object", "required": ["id"], "properties": {
           "id":   { "type": "string", "description": "Identifiant du texte (LEGITEXT… ou JORFTEXT…)" },
           "date": { "type": "string", "description": "Date de vigueur YYYY-MM-DD (défaut : aujourd'hui)" },
           "sommaire_seulement": { "type": "boolean", "default": false, "description": "Ne retourner que la table des matières" }
       }}))
}
fn tool_legi_part() -> ToolDef {
    mk("legifrance_legi_part",
       "Contenu d'un texte du fonds LEGI (/consult/legiPart) : structure et articles à une date donnée.",
       json!({ "type": "object", "required": ["id"], "properties": {
           "id":   { "type": "string", "description": "Identifiant (chronical ID) du texte (LEGITEXT…)" },
           "date": { "type": "string", "description": "Date de consultation YYYY-MM-DD (défaut : aujourd'hui)" }
       }}))
}
fn tool_lister_loda() -> ToolDef {
    mk("legifrance_lister_loda",
       "Liste paginée des textes LODA (lois, ordonnances, décrets, arrêtés) filtrable par nature, état, période de publication ou de signature.",
       json!({ "type": "object", "properties": {
           "natures": { "type": "array", "items": { "type": "string" }, "description": "LOI, ORDONNANCE, DECRET, DECRET_LOI, ARRETE, CONSTITUTION, DECISION, CONVENTION, DECLARATION, ACCORD_FONCTION_PUBLIQUE" },
           "en_vigueur_seulement": { "type": "boolean", "default": false },
           "statuts": { "type": "array", "items": { "type": "string" }, "description": "États juridiques (VIGUEUR, ABROGE…) — prioritaire sur en_vigueur_seulement" },
           "date_debut": { "type": "string", "description": "Publication : début YYYY-MM-DD" },
           "date_fin":   { "type": "string", "description": "Publication : fin YYYY-MM-DD" },
           "signature_debut": { "type": "string", "description": "Signature : début YYYY-MM-DD" },
           "signature_fin":   { "type": "string", "description": "Signature : fin YYYY-MM-DD" },
           "tri": { "type": "string", "description": "PUBLICATION_DATE_DESC|ASC, SIGNATURE_DATE_DESC|ASC, ID_ASC|DESC" },
           "nb_resultats": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 },
           "page": { "type": "integer", "minimum": 1, "default": 1 }
       }}))
}
async fn do_lister_loda(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = list_body(args, 20);
    if let Some(n) = args["natures"].as_array() {
        body["natures"] = json!(n.iter().filter_map(|x| x.as_str()).map(|x| x.to_uppercase()).collect::<Vec<_>>());
    }
    if let Some(st) = args["statuts"].as_array() { body["legalStatus"] = json!(st); }
    else if args["en_vigueur_seulement"].as_bool().unwrap_or(false) { body["legalStatus"] = json!(["VIGUEUR"]); }
    let period = |a: &str, b: &str| { let mut p = json!({}); if !a.is_empty() { p["start"] = json!(a); } if !b.is_empty() { p["end"] = json!(b); } p };
    let (dd, df) = (str_arg(args, "date_debut"), str_arg(args, "date_fin"));
    if !dd.is_empty() || !df.is_empty() { body["publicationDate"] = period(dd, df); }
    let (sd, sf) = (str_arg(args, "signature_debut"), str_arg(args, "signature_fin"));
    if !sd.is_empty() || !sf.is_empty() { body["signatureDate"] = period(sd, sf); }
    let tri = str_arg(args, "tri");
    if !tri.is_empty() { body["sort"] = json!(tri.to_uppercase()); }
    let data = client.post("/list/loda", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun texte LODA trouvé.".into()); }
    let mut lines = vec![format!("**{} texte(s) LODA** — {} affiché(s)\n", total_of(&data, items.len()), items.len())];
    for r in &items {
        let (d, f) = (fmt_date(&r["dateDebut"]), fmt_date(&r["dateFin"]));
        let mut l = format!("- **{}**", { let x = s(r, "titre"); if x.is_empty() { "?" } else { x } });
        if !s(r, "etat").is_empty() { l.push_str(&format!(" [{}]", s(r, "etat"))); }
        if !d.is_empty() { l.push_str(&format!(" — {d} → {}", if f.is_empty() || is_open_end(&f) { "en vigueur".into() } else { f })); }
        let id = { let c = s(r, "cid"); if c.is_empty() { s(r, "id") } else { c } };
        if !id.is_empty() { l.push_str(&format!(" — `{id}`")); }
        lines.push(l);
    }
    Ok(lines.join("\n"))
}

fn tool_historique_texte() -> ToolDef {
    mk("legifrance_historique_texte",
       "Historique des versions d'un texte (code, loi) sur une période d'années : date de chaque version et textes modificateurs. \
        Permet de suivre l'évolution législative dans le temps.",
       json!({ "type": "object", "required": ["text_cid", "annee_debut", "annee_fin"], "properties": {
           "text_cid":     { "type": "string",  "description": "CID du texte (ex: LEGITEXT...)" },
           "annee_debut":  { "type": "integer", "description": "Année de début" },
           "annee_fin":    { "type": "integer", "description": "Année de fin" },
           "date_consult": { "type": "string",  "description": "Date de référence YYYY-MM-DD (défaut : aujourd'hui)" }
       }}))
}
async fn do_historique_texte(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["text_cid", "id", "cid"])?;
    let (y0, y1) = (int_arg(args, "annee_debut", 0), int_arg(args, "annee_fin", 0));
    if y0 == 0 || y1 == 0 { anyhow::bail!("Paramètres 'annee_debut' et 'annee_fin' obligatoires."); }
    // POST /chrono/textCid : ChronoLegiTextRequest (textCid, startYear, endYear, dateConsult tous requis).
    let body = json!({ "textCid": id, "startYear": y0, "endYear": y1, "dateConsult": date_arg(args, "date_consult", &today()) });
    let data = client.post("/chrono/textCid", body).await?;
    Ok(fmt_chrono(&data, id))
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
    let eli = require_str(args, "eli")?;
    let data = client.post("/consult/eliAndAliasRedirectionTexte", json!({ "idEliOrAlias": eli })).await?;
    let cid = ["cid", "textCid", "id"].iter().map(|k| s(&data, k)).find(|x| !x.is_empty()).unwrap_or("");
    if cid.is_empty() { return Ok(format!("Aucun texte trouvé pour l'ELI : `{eli}`")); }
    let title = ["title", "titre"].iter().map(|k| s(&data, k)).find(|x| !x.is_empty()).unwrap_or("");
    Ok(format!("✅ L'ELI `{eli}` correspond au texte `{cid}`{}", if title.is_empty() { String::new() } else { format!(" — {title}") }))
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
    let text_cid = require_str(args, "text_cid")?;
    let element_cid = require_str(args, "element_cid")?;
    let body = json!({ "textCid": text_cid, "elementCid": element_cid });
    let data = client.post("/chrono/textCidAndElementCid", body).await?;
    Ok(fmt_chrono(&data, element_cid))
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
    let end = int_arg(args, "annee_fin", int_arg(args, "annee", 0));
    let end = if end == 0 { today()[..4].parse::<i64>().unwrap_or(2026) } else { end };
    let start = int_arg(args, "annee_debut", end);
    let data = client.post("/consult/getTables", json!({ "startYear": start, "endYear": end })).await?;
    let tables = data["tables"].as_array().cloned().unwrap_or_default();
    if tables.is_empty() { return Ok(format!("Aucune table annuelle pour {start}–{end}.")); }
    let mut lines = vec![format!("**{} table(s) annuelle(s)** ({start}–{end})\n", tables.len())];
    for tb in &tables {
        let name = { let f = s(tb, "fileName"); if f.is_empty() { s(tb, "id") } else { f } };
        let d = fmt_date(&tb["datePubli"]);
        lines.push(format!("- {name}{}{} — `{}`{}", if d.is_empty() { String::new() } else { format!(" ({d})") }, if s(tb, "num").is_empty() { String::new() } else { format!(" n°{}", s(tb, "num")) }, s(tb, "id"), if s(tb, "pathToFile").is_empty() { String::new() } else { format!(" — {}", s(tb, "pathToFile")) }));
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
async fn do_dates_sans_jo(_args: &Value, client: &LegifranceClient) -> Result<String> {
    let data = client.get("/misc/datesWithoutJo").await?;
    let dates = data["lstDateDisabled"].as_array().cloned().unwrap_or_default();
    if dates.is_empty() { return Ok("Aucune date sans JO retournée.".into()); }
    let list: Vec<String> = dates.iter().map(fmt_date).filter(|d| !d.is_empty()).collect();
    Ok(format!("**{} date(s) sans Journal Officiel**\n\n{}", list.len(), list.join(", ")))
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
    let data = client.get("/misc/yearsWithoutTable").await?;
    let ys = data["lstYearDisabled"].as_array().cloned().unwrap_or_default();
    if ys.is_empty() { return Ok("Aucune année sans table annuelle.".into()); }
    let list: Vec<String> = ys.iter().map(|y| y.as_i64().map(|n| n.to_string()).or_else(|| y.as_str().map(String::from)).unwrap_or_default()).collect();
    Ok(format!("**{} année(s) sans table annuelle**\n\n{}", list.len(), list.join(", ")))
}

// ============================================================================
// ── CONVENTIONS COLLECTIVES (KALI) ──
// ============================================================================

fn tool_conventions() -> ToolDef {
    mk("legifrance_conventions",
       "Liste paginée des conventions collectives (/list/conventions), filtrable par titre, IDCC ou état.",
       json!({ "type": "object", "properties": {
           "query":  { "type": "string", "description": "Texte recherché dans le titre" },
           "idcc":   { "type": "string", "description": "Numéro IDCC (ex. 1486)" },
           "statuts": { "type": "array", "items": { "type": "string" }, "description": "États juridiques (VIGUEUR, ABROGE…)" },
           "page":   { "type": "integer", "minimum": 1, "default": 1 },
           "taille": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
       }}))
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
    // KaliContConsultIdccRequest : champ obligatoire « id » (ID ou numéro IDCC).
    let data = client.post("/consult/kaliContIdcc", json!({ "id": idcc })).await?;
    let mut head = vec![format!("# {}\n", { let x = s(&data, "titre"); if x.is_empty() { idcc } else { x } })];
    for (k, l) in [("num", "IDCC"), ("id", "ID"), ("nature", "Nature"), ("etat", "État")] {
        let v = s(&data, k);
        if !v.is_empty() { head.push(format!("**{l}** : {v}")); }
    }
    Ok(format!("{}\n\n{}", head.join("\n"), fmt_toc_opts(&data, idcc, 4, false, 400)))
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
    let id = require_any(args, &["id", "text_id"])?;
    let data = client.post("/consult/kaliText", json!({ "id": id })).await?;
    Ok(fmt_legi_text(&data, id))
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
    mk("legifrance_jurisprudence_plan_classement",
       "Navigue dans le plan de classement (JURINOME) de la jurisprudence : recherche par libellé ou par identifiant.",
       json!({ "type": "object", "properties": {
           "query":  { "type": "string", "description": "Libellé recherché" },
           "id":     { "type": "string", "description": "Identifiant JURINOME (pour descendre dans l'arborescence)" },
           "fond":   { "type": "string", "description": "Fond (ex. JURI, CETAT)" },
           "niveau": { "type": "integer", "description": "Niveau courant dans l'arborescence" },
           "page":   { "type": "integer", "minimum": 1, "default": 1 },
           "par_niveau": { "type": "boolean", "default": false },
           "suggestion": { "type": "boolean", "default": false }
       }}))
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
    let id = require_any(args, &["ancien_id", "id"])?;
    let data = client.post("/consult/getJuriWithAncienId", json!({ "ancienId": id })).await?;
    Ok(fmt_texte_simple(data.get("text").unwrap_or(&data), id))
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
    let id = require_any(args, &["ancien_id", "id"])?;
    let data = client.post("/consult/getCnilWithAncienId", json!({ "ancienId": id })).await?;
    Ok(fmt_texte_simple(data.get("text").unwrap_or(&data), id))
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
    mk("legifrance_lister_dossiers_legislatifs",
       "Liste les dossiers législatifs d'une législature (voir legifrance_lister_legislatures pour les identifiants).",
       json!({ "type": "object", "required": ["legislature_id"], "properties": {
           "legislature_id": { "type": "integer", "description": "Identifiant de la législature" },
           "type_dossier": { "type": "string", "enum": ["LOI_PUBLIEE", "ORDONNANCE_PUBLIEE", "PROJET_LOI", "PROPOSITION_LOI"], "default": "LOI_PUBLIEE" }
       }}))
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
    let id = require_any(args, &["id", "debat_id", "text_id"])?;
    let data = client.post("/consult/debat", json!({ "id": id })).await?;
    Ok(fmt_debat(data.get("debat").unwrap_or(&data), id))
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
    let mut body = list_body(args, 20);
    if let Some(types) = args["types_publication"].as_array() { body["typesPublication"] = json!(types); }
    let dp = str_arg(args, "date_parution");
    if !dp.is_empty() { body["dateParution"] = json!(dp); }
    let data = client.post("/list/debatsParlementaires", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun débat parlementaire trouvé.".into()); }
    let mut lines = vec![format!("**{} débat(s)** — {} affiché(s)\n", total_of(&data, items.len()), items.len())];
    for d in &items {
        let dt = { let a = fmt_date(&d["dateSeance"]); if a.is_empty() { fmt_date(&d["dateParution"]) } else { a } };
        let mut meta = vec![];
        for k in ["typeAssemblee", "nomSession"] { if !s(d, k).is_empty() { meta.push(s(d, k).to_string()); } }
        if let Some(l) = d["legislature"].as_i64() { meta.push(format!("législature {l}")); }
        lines.push(format!("- {dt} — {} — `{}`", meta.join(" · "), s(d, "id")));
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
    let mut body = list_body(args, 20);
    if let Some(types) = args["parlement_types"].as_array() { body["parlementTypes"] = json!(types); }
    let p = str_arg(args, "periode_publication");
    if !p.is_empty() { body["periodePublication"] = json!(p); }
    let data = client.post("/list/questionsEcritesParlementaires", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucune question parlementaire trouvée.".into()); }
    let mut lines = vec![format!("**{} question(s)** — {} affiché(s)\n", total_of(&data, items.len()), items.len())];
    for q in &items {
        let mut l = format!("- {}", fmt_date(&q["dateParution"]));
        if !s(q, "emetteur").is_empty() { l.push_str(&format!(" — {}", s(q, "emetteur"))); }
        if !s(q, "numeroParution").is_empty() { l.push_str(&format!(" — n°{}", s(q, "numeroParution"))); }
        l.push_str(&format!(" — `{}`", s(q, "id")));
        let f = { let x = s(q, "pdfName"); if x.is_empty() { s(q, "url") } else { x } };
        if !f.is_empty() { l.push_str(&format!(" ({f})")); }
        lines.push(l);
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
    let mut body = list_body(args, 20);
    let iv = str_arg(args, "interval_publication");
    if !iv.is_empty() { body["intervalPublication"] = json!(iv); }
    let data = client.post("/list/bocc", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun BOCC trouvé.".into()); }
    let mut lines = vec![format!("**{} BOCC** — {} affiché(s)\n", total_of(&data, items.len()), items.len())];
    for b in &items {
        let g = &b["globalBocc"];
        lines.push(format!("- BOCC n°{} du {} — `{}` {}", s(g, "numParution"), fmt_date(&g["dateParution"]), s(b, "id"), s(g, "fileName")));
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
    let mut body = list_body(args, 20);
    let bid = str_arg(args, "bocc_id");
    if !bid.is_empty() { body["idMainBocc"] = json!(bid); }
    if let Some(i) = args["idccs"].as_array() { body["idccs"] = json!(i); }
    let data = client.post("/list/boccTexts", body).await?;
    let items = data["texts"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun texte BOCC trouvé.".into()); }
    let mut lines = vec![format!("**{} texte(s) BOCC** — {} affiché(s)\n", total_of(&data, items.len()), items.len())];
    for t in &items {
        let idcc: Vec<&str> = t["idccs"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
        lines.push(format!("- **{}**{} — {} — BOCC `{}` ({})", s(t, "title"), if idcc.is_empty() { String::new() } else { format!(" (IDCC {})", idcc.join(", ")) }, fmt_date(&t["texteDate"]), s(t, "idMainBocc"), s(t, "fileName")));
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
    let mut body = list_body(args, 20);
    // BoccAndTextListRequest : « idcc » est un scalaire (et non « idccs »).
    let (idcc, titre, iv) = (str_arg(args, "idcc"), str_arg(args, "titre"), str_arg(args, "interval_publication"));
    if !idcc.is_empty() { body["idcc"] = json!(idcc); }
    if !titre.is_empty() { body["titre"] = json!(titre); }
    if !iv.is_empty() { body["intervalPublication"] = json!(iv); }
    let data = client.post("/list/boccsAndTexts", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun résultat BOCC/texte trouvé.".into()); }
    let mut lines = vec![format!("**{} résultat(s)** — {} affiché(s)\n", total_of(&data, items.len()), items.len())];
    for r in &items {
        let g = &r["globalBocc"];
        lines.push(format!("- BOCC n°{} du {} — `{}`", s(g, "numParution"), fmt_date(&g["dateParution"]), s(r, "id")));
        for tx in r["texts"].as_array().map(|a| a.iter().take(5).collect::<Vec<_>>()).unwrap_or_default() {
            lines.push(format!("  ↳ {}", s(tx, "title")));
        }
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
    let mut body = list_body(args, 20);
    let a = int_arg(args, "annee", 0);
    if a > 0 { body["years"] = json!([a.to_string()]); }
    let data = client.post("/list/bodmr", body).await?;
    let metas = data["pdfMetadatas"].as_array().cloned().unwrap_or_default();
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if metas.is_empty() && items.is_empty() { return Ok("Aucun BODMR trouvé.".into()); }
    let mut lines = vec![format!("**{} BODMR**\n", total_of(&data, items.len().max(metas.len())))];
    if !metas.is_empty() {
        for m in &metas { lines.push(format!("- {} ({}) — `{}`", s(m, "fileName"), fmt_date(&m["datePubli"]), s(m, "id"))); }
    } else {
        for b in &items { lines.push(format!("- `{}`", s(b, "id"))); }
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
    // DocsAdminsListRequest.years : tableau de CHAÎNES.
    if let Some(an) = args["annees"].as_array() {
        body["years"] = json!(an.iter().map(|y| y.as_i64().map(|n| n.to_string()).or_else(|| y.as_str().map(String::from)).unwrap_or_default()).collect::<Vec<_>>());
    }
    let data = client.post("/list/docsAdmins", body).await?;
    let items = data["results"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucun document administratif trouvé.".into()); }
    let mut lines = vec![format!("**{} document(s) administratif(s)**\n", total_of(&data, items.len()))];
    for d in &items {
        let mut l = format!("- **{}**", { let x = s(d, "titre"); if x.is_empty() { "Sans titre" } else { x } });
        if !s(d, "nature").is_empty() { l.push_str(&format!(" [{}]", s(d, "nature"))); }
        let dt = fmt_date(&d["dateDocument"]);
        if !dt.is_empty() { l.push_str(&format!(" — {dt}")); }
        l.push_str(&format!(" — `{}`", s(d, "id")));
        lines.push(l);
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
    let lines = kv_lines(&data);
    if lines.is_empty() { return Ok(format!("Réponse : {data}")); }
    Ok(format!("**Version de l'API Légifrance**\n\n{}", lines.join("\n")))
}

async fn do_code_toc(args: &Value, client: &LegifranceClient) -> Result<String> {
    let code = require_str(args, "code")?;
    let text_id = resolve_code_dyn(client, code).await?;
    let date = date_arg(args, "date", &today());
    // /consult/code/tableMatieres est déprécié → /consult/legi/tableMatieres.
    let body = json!({ "textId": text_id, "date": date, "nature": "CODE" });
    let data = client.post("/consult/legi/tableMatieres", body).await?;
    let depth = int_arg(args, "profondeur", 4).clamp(1, 10) as usize;
    let with_art = args["inclure_articles"].as_bool().unwrap_or(false);
    Ok(fmt_toc_opts(&data, code, depth, with_art, 800))
}

async fn do_code_article(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "article_id"])?;
    let data = client.post("/consult/getArticle", json!({ "id": id })).await?;
    let a = data.get("article").unwrap_or(&data);
    if a.is_null() { anyhow::bail!("Article introuvable : {id}"); }
    Ok(fmt_article(a))
}

async fn do_code_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "cid"])?;
    let data = client.post("/consult/getSectionByCid", json!({ "cid": id })).await?;
    Ok(fmt_list_section(&data, id))
}

async fn do_code_article_by_num(args: &Value, client: &LegifranceClient) -> Result<String> {
    let code = require_str(args, "code")?;
    let numero = normalize_article_num(require_str(args, "numero")?);
    let id = resolve_code_dyn(client, code).await?;
    let data = client.post("/consult/getArticleWithIdAndNum", json!({ "id": id, "num": numero })).await?;
    let a = data.get("article").unwrap_or(&data);
    if a.is_null() || (a.is_object() && a.as_object().map(|o| o.is_empty()).unwrap_or(false)) {
        anyhow::bail!("Article '{numero}' introuvable dans {code} (version en vigueur).");
    }
    let date = str_arg(args, "date");
    if date.is_empty() { return Ok(fmt_article(a)); }
    let v = article_at_date(client, a, date).await?;
    Ok(fmt_article(&v))
}

async fn do_code_article_versions(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "cid"])?;
    // Opération dédiée : /consult/getArticleByCid → listArticle.
    let data = client.post("/consult/getArticleByCid", json!({ "cid": id })).await?;
    let mut versions = data["listArticle"].as_array().cloned().unwrap_or_default();
    if versions.is_empty() {
        // Repli : Article.articleVersions renvoyé par getArticle.
        let d = client.post("/consult/getArticle", json!({ "id": id })).await?;
        versions = d["article"]["articleVersions"].as_array().cloned().unwrap_or_default();
    }
    Ok(fmt_article_versions(&versions, id))
}

async fn do_code_article_liens(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "article_id"])?;
    let data = client.post("/consult/concordanceLinksArticle", json!({ "articleId": id })).await?;
    Ok(fmt_revision_links(&data, id, "Liens de concordance"))
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
    let id = require_any(args, &["id", "text_id"])?;
    let data = if id.starts_with("JORFTEXT") {
        client.post("/consult/jorf", json!({ "textCid": id })).await?
    } else {
        let body = json!({ "textId": id, "date": date_arg(args, "date", &today()) });
        match client.post("/consult/lawDecree", body.clone()).await {
            Ok(d) => d,
            Err(_) => client.post("/consult/legiPart", body).await?,
        }
    };
    if args["sommaire_seulement"].as_bool().unwrap_or(false) { return Ok(fmt_toc(&data, id)); }
    Ok(fmt_legi_text(&data, id))
}

async fn do_legi_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_id"])?;
    let body = json!({ "textId": id, "date": date_arg(args, "date", &today()) });
    let data = client.post("/consult/legiPart", body).await?;
    Ok(fmt_legi_text(&data, id))
}

async fn do_search_all(args: &Value, client: &LegifranceClient) -> Result<String> {
    let query = require_str(args, "query")?;
    let body = build_search_body(args, "ALL")?;
    let data = client.post("/search", body).await?;
    Ok(fmt_search(&data, query))
}

async fn do_search_suggest(args: &Value, client: &LegifranceClient) -> Result<String> {
    let query = require_str(args, "query")?;
    const SUPPLIES: &[&str] = &["ALL","JORF","CNIL","CETAT","JURI","JUFI","CONSTIT","KALI","CODE_DATE","CODE_ETAT","LODA_DATE","LODA_ETAT","CIRC","ACCO"];
    let f = norm_fond(str_arg(args, "fond"))?;
    let supply = if SUPPLIES.contains(&f.as_str()) { f } else { "ALL".to_string() };
    let body = json!({ "searchText": query, "supplies": [supply] });
    let data = client.post("/suggest", body).await?;
    Ok(fmt_suggest(&data, query, ""))
}

// ============================================================================
// ── JORF ──
// ============================================================================

async fn do_jorf_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_cid", "textCid"])?;
    // JorfConsultRequest : champ obligatoire textCid (et non « id »).
    let data = client.post("/consult/jorf", json!({ "textCid": id })).await?;
    Ok(fmt_legi_text(&data, id))
}

async fn do_jorf_article(args: &Value, client: &LegifranceClient) -> Result<String> {
    let nor = require_any(args, &["nor", "id"])?;
    if nor.starts_with("JORFTEXT") {
        let d = client.post("/consult/jorf", json!({ "textCid": nor })).await?;
        return Ok(fmt_legi_text(&d, nor));
    }
    let data = client.post("/consult/getJoWithNor", json!({ "nor": nor })).await?;
    Ok(fmt_legi_text(&data, nor))
}

async fn do_jorf_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_cid", "textCid"])?;
    let data = client.post("/consult/jorfPart", json!({ "textCid": id })).await?;
    Ok(fmt_legi_text(&data, id))
}

async fn do_jorf_jo_list(args: &Value, client: &LegifranceClient) -> Result<String> {
    let n = int_arg(args, "nb_elements", int_arg(args, "nb", 10)).clamp(1, 2499);
    let data = client.post("/consult/lastNJo", json!({ "nbElement": n })).await?;
    fmt_jo_list(&data)
}

async fn do_kali_list(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "pageNumber": int_arg(args, "page", 1).max(1), "pageSize": int_arg(args, "taille", 20).clamp(1, 100) });
    let q = str_arg(args, "query");
    if !q.is_empty() { body["titre"] = json!(q); }
    let idcc = str_arg(args, "idcc");
    if !idcc.is_empty() { body["idcc"] = json!(idcc); }
    if let Some(st) = args["statuts"].as_array() { body["legalStatus"] = json!(st); }
    let data = client.post("/list/conventions", body).await?;
    let results = data["results"].as_array().cloned().unwrap_or_default();
    if results.is_empty() { return Ok("Aucune convention collective trouvée.".into()); }
    let mut lines = vec![format!("**{} convention(s) collective(s)**\n", total_of(&data, results.len()))];
    for r in &results {
        let idcc = s(r, "idcc");
        lines.push(format!("- **{}**{}{}", { let x = s(r, "titre"); if x.is_empty() { "?" } else { x } },
            if idcc.is_empty() { String::new() } else { format!(" (IDCC {idcc})") },
            if s(r, "etat").is_empty() { String::new() } else { format!(" [{}]", s(r, "etat")) }));
        let (id, cont) = (s(r, "id"), s(r, "cidConteneur"));
        if !id.is_empty() || !cont.is_empty() {
            lines.push(format!("  texte de base `{id}` — conteneur `{cont}`"));
        }
    }
    Ok(lines.join("\n"))
}

async fn do_kali_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "cid"])?;
    let data = client.post("/consult/kaliCont", json!({ "id": id })).await?;
    let mut head = vec![format!("# {}\n", { let x = s(&data, "titre"); if x.is_empty() { id } else { x } })];
    for (k, l) in [("num", "IDCC"), ("id", "ID"), ("etat", "État")] {
        let v = s(&data, k);
        if !v.is_empty() { head.push(format!("**{l}** : {v}")); }
    }
    Ok(format!("{}\n\n{}", head.join("\n"), fmt_toc_opts(&data, id, 4, false, 400)))
}

async fn do_kali_article(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "article_id"])?;
    let data = client.post("/consult/kaliArticle", json!({ "id": id })).await?;
    // ConsultKaliTextResponse / article : tente l'objet article direct, sinon la structure.
    if data["article"].is_object() { return Ok(fmt_article(&data["article"])); }
    Ok(fmt_legi_text(&data, id))
}

async fn do_kali_section(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "cid"])?;
    let data = client.post("/consult/kaliSection", json!({ "id": id })).await?;
    Ok(fmt_legi_text(&data, id))
}

// ============================================================================
// ── CNIL ──
// ============================================================================

async fn do_cnil_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_id"])?;
    let data = client.post("/consult/cnil", json!({ "textId": id })).await?;
    Ok(fmt_texte_simple(data.get("text").unwrap_or(&data), id))
}

// ============================================================================
// ── ACCO ──
// ============================================================================

async fn do_acco_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_id"])?;
    let data = client.post("/consult/acco", json!({ "id": id })).await?;
    Ok(fmt_acco(data.get("acco").unwrap_or(&data), id))
}

// ============================================================================
// ── CIRCULAIRES ──
// ============================================================================

async fn do_circ_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_id"])?;
    let data = client.post("/consult/circulaire", json!({ "id": id })).await?;
    Ok(fmt_circulaire(data.get("circulaire").unwrap_or(&data), id))
}

// ============================================================================
// ── DOSSIERS LÉGISLATIFS ──
// ============================================================================

async fn do_lister_dossiers_legislatifs(args: &Value, client: &LegifranceClient) -> Result<String> {
    let leg = int_arg(args, "legislature_id", 0);
    if leg == 0 { anyhow::bail!("legislature_id est obligatoire."); }
    let ty = { let x = str_arg(args, "type_dossier").to_uppercase(); if x.is_empty() { "LOI_PUBLIEE".to_string() } else { x } };
    let data = client.post("/list/dossiersLegislatifs", json!({ "type": ty, "legislatureId": leg })).await?;
    let items = data["dossiersLegislatifs"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok(format!("Aucun dossier législatif ({ty}) pour la législature {leg}.")); }
    let mut lines = vec![format!("**{} dossier(s) législatif(s)** — législature {leg}, {ty}\n", items.len())];
    for d in items.iter().take(300) {
        let dc = fmt_date(&d["dateCreation"]);
        lines.push(format!("- **{}**{} — `{}`", { let x = s(d, "titre"); if x.is_empty() { "?" } else { x } }, if dc.is_empty() { String::new() } else { format!(" ({dc})") }, s(d, "id")));
    }
    Ok(lines.join("\n"))
}

async fn do_dossier_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_id"])?;
    let data = client.post("/consult/dossierLegislatif", json!({ "id": id })).await?;
    Ok(fmt_dossier(data.get("dossierLegislatif").unwrap_or(&data), id))
}

// ============================================================================
// ── JURISPRUDENCE ADMINISTRATIVE (JADE) ──
// ============================================================================

async fn do_jade_search(args: &Value, client: &LegifranceClient) -> Result<String> {
    let mut body = json!({ "page": int_arg(args, "page", 1).max(1) });
    let q = str_arg(args, "query");
    if !q.is_empty() { body["libelle"] = json!(q); }
    let id = str_arg(args, "id");
    if !id.is_empty() { body["id"] = json!(id); }
    let fond = str_arg(args, "fond");
    if !fond.is_empty() { body["fond"] = json!(norm_fond(fond)?); }
    if args["niveau"].is_number() { body["niveau"] = json!(int_arg(args, "niveau", 0)); }
    if args["par_niveau"].as_bool().unwrap_or(false) { body["searchByNiveau"] = json!(true); }
    if args["suggestion"].as_bool().unwrap_or(false) { body["searchSuggest"] = json!(true); }
    let data = client.post("/consult/getJuriPlanClassement", body).await?;
    let items = data["listPlanClassementJuri"].as_array().cloned().unwrap_or_default();
    if items.is_empty() { return Ok("Aucune entrée de plan de classement.".into()); }
    let mut lines = vec![format!("**{} entrée(s) du plan de classement**\n", items.len())];
    for it in items.iter().take(100) {
        let lib = ["libelle", "titre", "title"].iter().map(|k| s(it, k)).find(|x| !x.is_empty()).unwrap_or("?");
        lines.push(format!("- {lib} — `{}`", s(it, "id")));
    }
    Ok(lines.join("\n"))
}

async fn do_jade_text(args: &Value, client: &LegifranceClient) -> Result<String> {
    let id = require_any(args, &["id", "text_id"])?;
    let data = client.post("/consult/juri", json!({ "textId": id })).await?;
    Ok(fmt_texte_simple(data.get("text").unwrap_or(&data), id))
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
        "pageNumber": int_arg(args, "page", 1).max(1),
        "pageSize": int_arg(args, "nb_resultats", 20).clamp(1, 100),
        "highlightActivated": false,
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

// ============================================================================
// Aides communes (v2)
// ============================================================================

/// Premier argument non vide parmi plusieurs noms (alias).
fn arg_any<'a>(args: &'a Value, keys: &[&str]) -> &'a str {
    for k in keys {
        if let Some(v) = args[*k].as_str() { if !v.trim().is_empty() { return v.trim(); } }
    }
    ""
}
fn require_any<'a>(args: &'a Value, keys: &[&str]) -> Result<&'a str> {
    let v = arg_any(args, keys);
    if v.is_empty() { anyhow::bail!("Paramètre '{}' obligatoire.", keys[0]); }
    Ok(v)
}
fn list_body(args: &Value, default_size: i64) -> Value {
    json!({
        "pageNumber": int_arg(args, "page", 1).max(1),
        "pageSize":   int_arg(args, "nb_resultats", default_size).clamp(1, 100),
    })
}
fn mk(name: &str, description: &str, schema: Value) -> ToolDef {
    ToolDef { name: name.into(), description: description.into(), input_schema: schema }
}
fn total_of(data: &Value, n: usize) -> i64 { data["totalResultNumber"].as_i64().unwrap_or(n as i64) }

/// Résolution d'un code : table statique, sinon interrogation de /list/code.
async fn resolve_code_dyn(client: &LegifranceClient, code: &str) -> Result<String> {
    if let Ok(id) = resolve_code(code) { return Ok(id); }
    let body = json!({ "pageNumber": 1, "pageSize": 20, "codeName": code.trim(), "states": ["VIGUEUR"], "sort": "TITLE_ASC" });
    let data = client.post("/list/code", body).await?;
    let res = data["results"].as_array().cloned().unwrap_or_default();
    let q = code.trim().to_lowercase();
    let exact: Vec<&Value> = res.iter().filter(|r| s(r, "titre").to_lowercase() == q).collect();
    let cand: Vec<&Value> = if exact.len() == 1 { exact } else { res.iter().collect() };
    let id_of = |r: &Value| { let c = s(r, "cid"); (if c.is_empty() { s(r, "id") } else { c }).to_string() };
    match cand.len() {
        0 => anyhow::bail!("Aucun code ne correspond à '{code}'. Utilisez legifrance_lister_codes."),
        1 => Ok(id_of(cand[0])),
        _ => {
            let l: Vec<String> = cand.iter().take(8).map(|r| format!("{} ({})", s(r, "titre"), id_of(r))).collect();
            anyhow::bail!("'{code}' est ambigu. Précisez ou passez l'ID LEGITEXT : {}", l.join(" ; "))
        }
    }
}

/// « L. 1234-1 », « Article L1234-1 », « art. 1382 » → forme attendue par l'API (« L1234-1 », « 1382 »).
fn normalize_article_num(n: &str) -> String {
    let mut t = n.trim().replace('\u{a0}', " ");
    let low = t.to_lowercase();
    for p in ["article ", "art. ", "art "] {
        if low.starts_with(p) { if let Some(r) = t.get(p.len()..) { t = r.trim().to_string(); } break; }
    }
    let cs: Vec<char> = t.chars().collect();
    if cs.len() > 2 && cs[0].is_ascii_uppercase() && (cs[1] == '.' || cs[1] == ' ') {
        let rest: String = cs[2..].iter().collect::<String>().trim_start().to_string();
        if rest.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return format!("{}{}", cs[0], rest);
        }
    }
    t
}

/// Version d'un article en vigueur à `date`, via Article.articleVersions.
async fn article_at_date(client: &LegifranceClient, article: &Value, date: &str) -> Result<Value> {
    let Some(ms) = ymd_to_ms(date) else { anyhow::bail!("Date invalide '{date}' (format YYYY-MM-DD).") };
    if date == today() { return Ok(article.clone()); }
    let cur_id = s(article, "id");
    if let Some(vs) = article["articleVersions"].as_array() {
        for v in vs {
            let d = value_to_ms(&v["dateDebut"]).unwrap_or(i64::MAX);
            let f = value_to_ms(&v["dateFin"]).unwrap_or(i64::MAX);
            if d <= ms && ms <= f {
                let id = s(v, "id");
                if id.is_empty() || id == cur_id { return Ok(article.clone()); }
                let data = client.post("/consult/getArticle", json!({ "id": id })).await?;
                return Ok(data["article"].clone());
            }
        }
    }
    anyhow::bail!("Aucune version de cet article n'était en vigueur au {date} (voir legifrance_versions_article).")
}

fn flatten_results(data: &Value) -> Vec<(String, Value)> {
    match &data["results"] {
        Value::Object(m) => m.iter().flat_map(|(k, v)| v.as_array().cloned().unwrap_or_default().into_iter().map(move |x| (k.clone(), x))).collect(),
        Value::Array(a) => a.iter().map(|x| (String::new(), x.clone())).collect(),
        _ => vec![],
    }
}

fn fmt_suggest(data: &Value, query: &str, what: &str) -> String {
    let items = flatten_results(data);
    if items.is_empty() { return format!("Aucune suggestion {what} pour : « {query} »"); }
    let mut lines = vec![format!("**Suggestions {what} pour « {query} »**\n")];
    for (grp, r) in items.iter().take(30) {
        let title = ["title", "titre", "label", "libelle", "name", "raisonSociale", "rs"].iter().map(|k| s(r, k)).find(|x| !x.is_empty()).unwrap_or("");
        let title = if title.is_empty() { r.to_string() } else { title.to_string() };
        let id = ["id", "cid", "idText", "textId"].iter().map(|k| s(r, k)).find(|x| !x.is_empty()).unwrap_or("");
        let siret = s(r, "siret");
        let mut l = format!("- {title}");
        if !siret.is_empty() { l.push_str(&format!(" — SIRET {siret}")); }
        if !id.is_empty() { l.push_str(&format!(" — `{id}`")); }
        if !grp.is_empty() { l.push_str(&format!(" [{grp}]")); }
        lines.push(l);
    }
    lines.join("\n")
}

fn tool_ping() -> ToolDef {
    mk("legifrance_ping",
       "Vérifie la disponibilité des 5 contrôleurs de l'API Légifrance (chrono, consult, list, search, suggest) et l'authentification PISTE.",
       json!({ "type": "object", "properties": {} }))
}
async fn do_ping(client: &LegifranceClient) -> Result<String> {
    let mut lines = vec!["**État de l'API Légifrance**\n".to_string()];
    for c in ["chrono", "consult", "list", "search", "suggest"] {
        match client.get(&format!("/{c}/ping")).await {
            Ok(v) => lines.push(format!("- ✅ `/{c}/ping` : {}", v.as_str().map(String::from).unwrap_or_else(|| v.to_string()))),
            Err(e) => lines.push(format!("- ❌ `/{c}/ping` : {e}")),
        }
    }
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tris_par_fond() {
        assert_eq!(sort_for_fond("JORF", "DATE_DESC"), "SIGNATURE_DATE_DESC");
        assert_eq!(sort_for_fond("CETAT", "DATE_ASC"), "DATE_ASC");
        assert_eq!(sort_for_fond("CNIL", "DATE_DESC"), "DATE_DECISION_DESC");
        assert_eq!(sort_for_fond("ALL", "DATE_DESC"), "PERTINENCE");
        assert_eq!(sort_for_fond("CODE_DATE", ""), "PERTINENCE");
        assert_eq!(sort_for_fond("JORF", "SIGNATURE_DATE_ASC"), "SIGNATURE_DATE_ASC");
    }
    #[test]
    fn fonds_valides_et_alias() {
        assert_eq!(norm_fond("jade").unwrap(), "CETAT");
        assert_eq!(norm_fond("").unwrap(), "ALL");
        assert_eq!(norm_fond("code").unwrap(), "CODE_DATE");
        assert!(norm_fond("DOSSIER_LEGISLATIF").is_err());
    }
    #[test]
    fn numero_article() {
        assert_eq!(normalize_article_num("L. 1234-1"), "L1234-1");
        assert_eq!(normalize_article_num("Article R123-4"), "R123-4");
        assert_eq!(normalize_article_num("art. 1382"), "1382");
        assert_eq!(normalize_article_num("6 nonies"), "6 nonies");
    }
    #[test]
    fn corps_de_recherche_code() {
        let b = build_search_body(&json!({"query":"bail","code":"civil","date_debut":"2020-05-01"}), "ALL").unwrap();
        assert_eq!(b["fond"], "CODE_DATE");
        let f = &b["recherche"]["filtres"];
        assert_eq!(f[0], json!({"facette":"NOM_CODE","valeurs":["Code civil"]}));
        assert_eq!(f[1], json!({"facette":"DATE_VERSION","singleDate":"2020-05-01"}));
        assert_eq!(b["recherche"]["sort"], "PERTINENCE");
    }
    #[test]
    fn corps_de_recherche_jorf() {
        let b = build_search_body(&json!({"query":"x","fond":"JORF","tri":"DATE_DESC","nature":"loi","date_debut":"2020-01-01"}), "ALL").unwrap();
        let r = &b["recherche"];
        assert_eq!(r["sort"], "SIGNATURE_DATE_DESC");
        assert_eq!(r["filtres"][0], json!({"facette":"DATE_SIGNATURE","dates":{"start":"2020-01-01"}}));
        assert_eq!(r["filtres"][1], json!({"facette":"NATURE","valeurs":["LOI"]}));
    }
}
