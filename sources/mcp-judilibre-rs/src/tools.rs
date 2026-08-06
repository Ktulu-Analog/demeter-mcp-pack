// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
use crate::client::JudiLibreClient;
use crate::fmt::{fmt_decision_full, fmt_decision_short};
use crate::mcp::{ToolCallResult, ToolDef};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers pour construire les paramètres de requête
// ---------------------------------------------------------------------------

fn add_opt_str(p: &mut HashMap<String, Vec<String>>, key: &str, v: &Value) {
    if let Some(s) = v.as_str() {
        if !s.is_empty() {
            p.insert(key.to_string(), vec![s.to_string()]);
        }
    }
}

fn add_opt_bool(p: &mut HashMap<String, Vec<String>>, key: &str, v: &Value) {
    if let Some(b) = v.as_bool() {
        p.insert(key.to_string(), vec![b.to_string()]);
    }
}

/// Tableau JSON → plusieurs valeurs pour un même paramètre GET.
fn add_opt_array(p: &mut HashMap<String, Vec<String>>, key: &str, v: &Value) {
    if let Some(arr) = v.as_array() {
        let vals: Vec<String> = arr
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect();
        if !vals.is_empty() {
            p.insert(key.to_string(), vals);
        }
    }
}

// ---------------------------------------------------------------------------
// Registre des outils
// ---------------------------------------------------------------------------

pub fn all_tools() -> Vec<ToolDef> {
    vec![
        tool_rechercher(),
        tool_decision(),
        tool_scan(),
        tool_export(),
        tool_taxonomie(),
        tool_stats(),
        tool_historique(),
        tool_healthcheck(),
    ]
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub async fn dispatch(
    name: &str,
    args: &Value,
    client: &JudiLibreClient,
) -> ToolCallResult {
    let result: Result<String> = match name {
        "judilibre_rechercher"  => do_rechercher(args, client).await,
        "judilibre_decision"    => do_decision(args, client).await,
        "judilibre_scan"        => do_scan(args, client).await,
        "judilibre_export"      => do_export(args, client).await,
        "judilibre_taxonomie"   => do_taxonomie(args, client).await,
        "judilibre_stats"       => do_stats(args, client).await,
        "judilibre_historique"  => do_historique(args, client).await,
        "judilibre_healthcheck" => do_healthcheck(client).await,
        other => Err(anyhow::anyhow!("Outil inconnu : {other}")),
    };
    match result {
        Ok(text) => ToolCallResult::text(text),
        Err(e)   => ToolCallResult::error(format!("❌ Erreur : {e}")),
    }
}

// ===========================================================================
// 1. judilibre_rechercher
// ===========================================================================

fn tool_rechercher() -> ToolDef {
    ToolDef {
        name: "judilibre_rechercher".into(),
        description: "Recherche plein texte dans la base Open Data des décisions de justice françaises (JUDILIBRE). \
            Couvre la Cour de cassation, les cours d'appel et les tribunaux judiciaires. \
            Retourne une liste paginée avec métadonnées et sommaires. \
            Utilisez judilibre_decision pour le texte intégral.".into(),
        input_schema: json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": { "type": "string", "description": "Termes de recherche en texte libre (ex: 'responsabilité civile')" },
                "operateur": { "type": "string", "enum": ["or","and","exact"], "default": "or" },
                "champs": { "type": "array", "items": { "type": "string" },
                    "description": "Zones : expose, moyens, motivations, dispositif, annexes, sommaire, titrage" },
                "juridiction": { "type": "array", "items": { "type": "string" },
                    "description": "'cc' (Cour de cassation), 'ca' (cour d'appel), 'tj' (tribunal judiciaire)" },
                "chambre": { "type": "array", "items": { "type": "string" } },
                "formation": { "type": "array", "items": { "type": "string" } },
                "type_decision": { "type": "array", "items": { "type": "string" } },
                "solution": { "type": "array", "items": { "type": "string" } },
                "publication": { "type": "array", "items": { "type": "string" } },
                "matiere": { "type": "array", "items": { "type": "string" } },
                "localisation": { "type": "array", "items": { "type": "string" } },
                "date_debut": { "type": "string", "description": "YYYY-MM-DD" },
                "date_fin": { "type": "string", "description": "YYYY-MM-DD" },
                "tri": { "type": "string", "enum": ["scorepub","score","date"], "default": "scorepub" },
                "ordre": { "type": "string", "enum": ["desc","asc"], "default": "desc" },
                "nb_resultats": { "type": "integer", "minimum": 1, "maximum": 50, "default": 10 },
                "page": { "type": "integer", "minimum": 0, "default": 0 },
                "type_fichier": { "type": "array", "items": { "type": "string" } },
                "interet_particulier": { "type": "boolean" }
            }
        }),
    }
}

async fn do_rechercher(args: &Value, client: &JudiLibreClient) -> Result<String> {
    let query = args["query"].as_str().unwrap_or("").trim().to_string();
    if query.is_empty() {
        anyhow::bail!("Le paramètre 'query' est obligatoire.");
    }

    let page     = args["page"].as_i64().unwrap_or(0);
    let nb       = args["nb_resultats"].as_i64().unwrap_or(10).clamp(1, 50);
    let tri      = args["tri"].as_str().unwrap_or("scorepub");
    let ordre    = args["ordre"].as_str().unwrap_or("desc");
    let operateur = args["operateur"].as_str().unwrap_or("or");

    let mut p: HashMap<String, Vec<String>> = HashMap::new();
    p.insert("query".into(),              vec![query.clone()]);
    p.insert("operator".into(),           vec![operateur.to_string()]);
    p.insert("sort".into(),               vec![tri.to_string()]);
    p.insert("order".into(),              vec![ordre.to_string()]);
    p.insert("page_size".into(),          vec![nb.to_string()]);
    p.insert("page".into(),               vec![page.to_string()]);
    p.insert("resolve_references".into(), vec!["true".into()]);

    add_opt_array(&mut p, "field",            &args["champs"]);
    add_opt_array(&mut p, "jurisdiction",     &args["juridiction"]);
    add_opt_array(&mut p, "chamber",          &args["chambre"]);
    add_opt_array(&mut p, "formation",        &args["formation"]);
    add_opt_array(&mut p, "type",             &args["type_decision"]);
    add_opt_array(&mut p, "solution",         &args["solution"]);
    add_opt_array(&mut p, "publication",      &args["publication"]);
    add_opt_array(&mut p, "theme",            &args["matiere"]);
    add_opt_array(&mut p, "location",         &args["localisation"]);
    add_opt_array(&mut p, "withFileOfType",   &args["type_fichier"]);
    add_opt_str  (&mut p, "date_start",       &args["date_debut"]);
    add_opt_str  (&mut p, "date_end",         &args["date_fin"]);
    add_opt_bool (&mut p, "particularInterest", &args["interet_particulier"]);

    let data = client.request("/search", &p).await?;

    let total   = data["total"].as_i64().unwrap_or(0);
    let took    = data["took"].as_i64().unwrap_or(0);
    let results = data["results"].as_array().cloned().unwrap_or_default();

    if results.is_empty() {
        return Ok(format!("Aucun résultat pour : « {query} »"));
    }

    let mut lines = vec![format!(
        "**{total} décision(s) pour « {query} »** — {} affichée(s) — page {page} — {took} ms\n",
        results.len()
    )];
    for (i, r) in results.iter().enumerate() {
        lines.push(format!("{}. {}", i + 1, fmt_decision_short(r)));
        lines.push(String::new());
    }
    if data["next_page"].as_str().is_some() {
        lines.push(format!("*Page suivante disponible (page {})*", page + 1));
    }
    Ok(lines.join("\n"))
}

// ===========================================================================
// 2. judilibre_decision
// ===========================================================================

fn tool_decision() -> ToolDef {
    ToolDef {
        name: "judilibre_decision".into(),
        description: "Récupère le texte intégral et toutes les métadonnées d'une décision par son identifiant. \
            L'identifiant s'obtient via judilibre_rechercher.".into(),
        input_schema: json!({
            "type": "object",
            "required": ["id"],
            "properties": {
                "id": { "type": "string", "description": "Identifiant unique (ex: '5fca7d162a251e6bf9c78514')" },
                "query": { "type": "string", "description": "Termes à surligner dans le texte retourné" },
                "operateur": { "type": "string", "enum": ["or","and","exact"], "default": "or" }
            }
        }),
    }
}

async fn do_decision(args: &Value, client: &JudiLibreClient) -> Result<String> {
    let id = args["id"].as_str().unwrap_or("").trim().to_string();
    if id.is_empty() { anyhow::bail!("Le paramètre 'id' est obligatoire."); }

    let mut p: HashMap<String, Vec<String>> = HashMap::new();
    p.insert("id".into(),                 vec![id.clone()]);
    p.insert("resolve_references".into(), vec!["true".into()]);
    add_opt_str(&mut p, "query",    &args["query"]);
    add_opt_str(&mut p, "operator", &args["operateur"]);

    let data = client.request("/decision", &p).await?;
    if data.is_null() || data == Value::Null {
        return Ok(format!("Décision `{id}` introuvable."));
    }
    Ok(fmt_decision_full(&data))
}

// ===========================================================================
// 3. judilibre_scan
// ===========================================================================

fn tool_scan() -> ToolDef {
    ToolDef {
        name: "judilibre_scan".into(),
        description: "Export par lot de décisions complètes (texte intégral inclus) via curseur (search_after). \
            Jusqu'à 1000 décisions par lot. Remplace /export (déprécié).".into(),
        input_schema: export_scan_schema(true),
    }
}

async fn do_scan(args: &Value, client: &JudiLibreClient) -> Result<String> {
    let ordre     = args["ordre"].as_str().unwrap_or("asc");
    let taille    = args["taille_lot"].as_i64().unwrap_or(10).clamp(1, 1000);
    let abrege    = args["abrege"].as_bool().unwrap_or(false);

    let mut p: HashMap<String, Vec<String>> = HashMap::new();
    p.insert("order".into(),              vec![ordre.to_string()]);
    p.insert("batch_size".into(),         vec![taille.to_string()]);
    p.insert("resolve_references".into(), vec!["true".into()]);
    p.insert("abridged".into(),           vec![abrege.to_string()]);

    add_common_filters(&mut p, args);
    add_opt_str (&mut p, "date_type",    &args["type_date"]);
    add_opt_str (&mut p, "search_after", &args["search_after"]);
    add_opt_bool(&mut p, "particularInterest", &args["interet_particulier"]);

    let data = client.request("/scan", &p).await?;
    fmt_batch_result(&data, None, "scan")
}

// ===========================================================================
// 4. judilibre_export
// ===========================================================================

fn tool_export() -> ToolDef {
    ToolDef {
        name: "judilibre_export".into(),
        description: "Export par lot de décisions via pagination par numéro de lot (offset). \
            Contrairement à judilibre_scan (curseur), utilise un entier `lot` pour naviguer.".into(),
        input_schema: export_scan_schema(false),
    }
}

async fn do_export(args: &Value, client: &JudiLibreClient) -> Result<String> {
    let ordre     = args["ordre"].as_str().unwrap_or("asc");
    let taille    = args["taille_lot"].as_i64().unwrap_or(10).clamp(1, 1000);
    let lot       = args["lot"].as_i64().unwrap_or(0).max(0);
    let abrege    = args["abrege"].as_bool().unwrap_or(false);

    let mut p: HashMap<String, Vec<String>> = HashMap::new();
    p.insert("order".into(),              vec![ordre.to_string()]);
    p.insert("batch_size".into(),         vec![taille.to_string()]);
    p.insert("batch".into(),              vec![lot.to_string()]);
    p.insert("resolve_references".into(), vec!["true".into()]);
    p.insert("abridged".into(),           vec![abrege.to_string()]);

    add_common_filters(&mut p, args);
    add_opt_str (&mut p, "date_type",        &args["type_date"]);
    add_opt_bool(&mut p, "particularInterest", &args["interet_particulier"]);

    let data = client.request("/export", &p).await?;
    fmt_batch_result(&data, Some(lot), "export")
}

// ===========================================================================
// 5. judilibre_taxonomie
// ===========================================================================

fn tool_taxonomie() -> ToolDef {
    ToolDef {
        name: "judilibre_taxonomie".into(),
        description: "Récupère les listes de termes (taxonomie) utilisés par l'API JUDILIBRE : \
            juridictions, chambres, formations, types, solutions, publications, matières, etc. \
            Indispensable pour connaître les valeurs valides des filtres.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "id": { "type": "string",
                    "description": "'type','jurisdiction','chamber','formation','publication','theme','solution','field','zones','location','filetype'" },
                "cle": { "type": "string", "description": "Clé dont on veut l'intitulé (nécessite id)" },
                "valeur": { "type": "string", "description": "Intitulé dont on veut la clé (nécessite id)" },
                "contexte": { "type": "string", "description": "'cc' ou 'ca' (défaut: 'cc')" }
            }
        }),
    }
}

async fn do_taxonomie(args: &Value, client: &JudiLibreClient) -> Result<String> {
    let mut p: HashMap<String, Vec<String>> = HashMap::new();
    add_opt_str(&mut p, "id",            &args["id"]);
    add_opt_str(&mut p, "key",           &args["cle"]);
    add_opt_str(&mut p, "value",         &args["valeur"]);
    add_opt_str(&mut p, "context_value", &args["contexte"]);

    let data = client.request("/taxonomy", &p).await?;

    // Résolution clé/valeur unique
    if args["cle"].is_string() || args["valeur"].is_string() {
        let result = &data["result"];
        if !result.is_null() {
            let k = result["key"].as_str().unwrap_or(args["cle"].as_str().unwrap_or(""));
            let v = result["value"].as_str().unwrap_or("");
            return Ok(format!("**{k}** → {v}"));
        }
        return Ok("Aucun résultat de taxonomie trouvé.".into());
    }

    // Liste d'une taxonomie
    if let Some(id) = args["id"].as_str() {
        let items_val = data.get("result")
            .or_else(|| data.get("results"))
            .cloned()
            .unwrap_or_else(|| data.clone());

        if let Some(items) = items_val.as_array() {
            let mut lines = vec![format!("**Taxonomie '{id}'** ({} entrée(s))\n", items.len())];
            for item in items {
                let k = item["key"].as_str().unwrap_or("");
                let v = item["value"].as_str().unwrap_or("");
                lines.push(format!("- `{k}` — {v}"));
            }
            return Ok(lines.join("\n"));
        }
    }

    // Liste complète des taxonomies
    if let Some(items) = data.as_array() {
        let mut lines = vec![format!("**Taxonomies disponibles ({})**\n", items.len())];
        for item in items {
            let k = item["key"].as_str().or_else(|| item["id"].as_str()).unwrap_or("");
            let v = item["value"].as_str().unwrap_or("");
            lines.push(format!("- `{k}` — {v}"));
        }
        return Ok(lines.join("\n"));
    }

    Ok(format!("{data}"))
}

// ===========================================================================
// 6. judilibre_stats
// ===========================================================================

fn tool_stats() -> ToolDef {
    ToolDef {
        name: "judilibre_stats".into(),
        description: "Statistiques sur le contenu de la base JUDILIBRE : total indexé, \
            dates extrêmes, répartition par juridiction, chambre, solution, etc. \
            Mise à jour quotidienne.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "juridiction": { "type": "string", "description": "'cc','ca','tj','tcom'" },
                "localisation": { "type": "string" },
                "date_debut": { "type": "string", "description": "YYYY-MM-DD" },
                "date_fin": { "type": "string", "description": "YYYY-MM-DD" },
                "agregations": { "type": "string",
                    "description": "Séparées par virgule : year,month,jurisdiction,source,location,theme,formation,chamber,solution,type,publication" },
                "interet_particulier": { "type": "string", "description": "'true'" }
            }
        }),
    }
}

async fn do_stats(args: &Value, client: &JudiLibreClient) -> Result<String> {
    let mut p: HashMap<String, Vec<String>> = HashMap::new();
    add_opt_str(&mut p, "jurisdiction",       &args["juridiction"]);
    add_opt_str(&mut p, "location",           &args["localisation"]);
    add_opt_str(&mut p, "date_start",         &args["date_debut"]);
    add_opt_str(&mut p, "date_end",           &args["date_fin"]);
    add_opt_str(&mut p, "keys",               &args["agregations"]);
    add_opt_str(&mut p, "particularInterest", &args["interet_particulier"]);

    let data = client.request("/stats", &p).await?;
    let results = &data["results"];
    let query   = &data["query"];

    let mut lines = vec!["**Statistiques JUDILIBRE**\n".to_string()];

    // Contexte
    if query.is_object() {
        let mut ctx = Vec::new();
        if let Some(j) = query["jurisdiction"].as_str() { ctx.push(format!("juridiction : {j}")); }
        if let Some(k) = query["keys"].as_str()         { ctx.push(format!("agrégation : {k}")); }
        if !ctx.is_empty() { lines.push(format!("*Filtres : {}*\n", ctx.join(", "))); }
    }

    // Chiffres globaux
    let total = results["total_decisions"].as_i64().unwrap_or(0);
    let min_d = results["min_decision_date"].as_str().unwrap_or("");
    let max_d = results["max_decision_date"].as_str().unwrap_or("");
    lines.push(format!("**Total décisions indexées** : {}", format_number(total)));
    if !min_d.is_empty() { lines.push(format!("**Décision la plus ancienne** : {min_d}")); }
    if !max_d.is_empty() { lines.push(format!("**Décision la plus récente** : {max_d}")); }

    // Agrégations
    if let Some(agg) = results["aggregated_data"].as_array() {
        if !agg.is_empty() {
            lines.push(format!("\n**Répartition ({} bucket(s))**\n", agg.len()));
            for bucket in agg.iter().take(50) {
                let count = bucket["decisions_count"].as_i64().unwrap_or(0);
                if let Some(key_obj) = bucket["key"].as_object() {
                    let label: Vec<String> = key_obj.iter()
                        .map(|(k, v)| format!("{k}: {}", v.as_str().unwrap_or(&v.to_string())))
                        .collect();
                    lines.push(format!("- {} → **{count}** décision(s)", label.join(" | ")));
                }
            }
            if agg.len() > 50 {
                lines.push(format!("*… {} bucket(s) supplémentaire(s)*", agg.len() - 50));
            }
        }
    }

    Ok(lines.join("\n"))
}

// ===========================================================================
// 7. judilibre_historique
// ===========================================================================

fn tool_historique() -> ToolDef {
    ToolDef {
        name: "judilibre_historique".into(),
        description: "Historique transactionnel de la base : opérations create/update/delete \
            depuis une date donnée. Permet aux réutilisateurs de synchroniser leur propre index.".into(),
        input_schema: json!({
            "type": "object",
            "required": ["date"],
            "properties": {
                "date": { "type": "string", "description": "ISO-8601 ex: '2024-01-01T00:00:00Z'" },
                "taille_page": { "type": "integer", "minimum": 10, "maximum": 500, "default": 500 },
                "from_id": { "type": "string", "description": "ID pour paginer (fourni par la réponse précédente)" }
            }
        }),
    }
}

async fn do_historique(args: &Value, client: &JudiLibreClient) -> Result<String> {
    let date = args["date"].as_str().unwrap_or("").trim().to_string();
    if date.is_empty() { anyhow::bail!("Le paramètre 'date' est obligatoire."); }

    let mut p: HashMap<String, Vec<String>> = HashMap::new();
    p.insert("date".into(), vec![date.clone()]);

    if let Some(n) = args["taille_page"].as_i64() {
        p.insert("page_size".into(), vec![n.clamp(10, 500).to_string()]);
    }
    add_opt_str(&mut p, "from_id", &args["from_id"]);

    let data = client.request("/transactionalhistory", &p).await?;

    let transactions = match data["transactions"].as_array() {
        Some(t) => t.clone(),
        None => return Ok(format!("Aucune opération trouvée depuis le {date}.")),
    };
    if transactions.is_empty() {
        return Ok(format!("Aucune opération trouvée depuis le {date}."));
    }

    let total      = data["total"].as_i64().unwrap_or(0);
    let query_date = data["query_date"].as_str().unwrap_or("");
    let next_page  = data["next_page"].as_str().unwrap_or("");

    // Comptage par type
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for t in &transactions {
        let action = t["action"].as_str().unwrap_or("unknown");
        *counts.entry(action).or_default() += 1;
    }
    let summary: Vec<String> = {
        let mut v: Vec<(&str, usize)> = counts.into_iter().collect();
        v.sort_by_key(|(k, _)| *k);
        v.iter().map(|(k, n)| format!("{n} {k}")).collect()
    };

    let mut lines = vec![format!(
        "**{total} opération(s) depuis le {date}** — {} affichée(s) — requête du {query_date}\n",
        transactions.len()
    )];
    lines.push(format!("*Répartition : {}*\n", summary.join(", ")));

    for t in transactions.iter().take(100) {
        let action = t["action"].as_str().unwrap_or("");
        let tid    = t["id"].as_str().unwrap_or("");
        let tdate  = t["date"].as_str().unwrap_or("");
        let icon   = match action {
            "created" => "✅",
            "updated" => "🔄",
            "deleted" => "🗑️",
            _         => "•",
        };
        lines.push(format!("{icon} **{action}** — `{tid}` — {tdate}"));
    }
    if transactions.len() > 100 {
        lines.push(format!("\n*… {} opération(s) supplémentaire(s) non affichée(s)*",
            transactions.len() - 100));
    }
    if !next_page.is_empty() {
        lines.push("\n*Suite disponible — utilisez `from_id` extrait du paramètre next_page*".into());
    }
    Ok(lines.join("\n"))
}

// ===========================================================================
// 8. judilibre_healthcheck
// ===========================================================================

fn tool_healthcheck() -> ToolDef {
    ToolDef {
        name: "judilibre_healthcheck".into(),
        description: "Vérifie que l'API JUDILIBRE est opérationnelle. \
            Utile pour diagnostiquer les problèmes de connectivité.".into(),
        input_schema: json!({ "type": "object", "properties": {} }),
    }
}

async fn do_healthcheck(client: &JudiLibreClient) -> Result<String> {
    let p = HashMap::new();
    let data = client.request("/healthcheck", &p).await?;
    let status = data["status"].as_str()
        .or_else(|| data["healthy"].as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| data.to_string());
    Ok(format!("**JUDILIBRE healthcheck** : {status}"))
}

// ===========================================================================
// Helpers partagés entre scan et export
// ===========================================================================

fn add_common_filters(p: &mut HashMap<String, Vec<String>>, args: &Value) {
    add_opt_array(p, "jurisdiction",   &args["juridiction"]);
    add_opt_array(p, "chamber",        &args["chambre"]);
    add_opt_array(p, "formation",      &args["formation"]);
    add_opt_array(p, "type",           &args["type_decision"]);
    add_opt_array(p, "solution",       &args["solution"]);
    add_opt_array(p, "publication",    &args["publication"]);
    add_opt_array(p, "theme",          &args["matiere"]);
    add_opt_array(p, "location",       &args["localisation"]);
    add_opt_array(p, "withFileOfType", &args["type_fichier"]);
    add_opt_str  (p, "date_start",     &args["date_debut"]);
    add_opt_str  (p, "date_end",       &args["date_fin"]);
}

fn export_scan_schema(with_search_after: bool) -> Value {
    let mut props = json!({
        "juridiction":   { "type": "array", "items": { "type": "string" } },
        "chambre":       { "type": "array", "items": { "type": "string" } },
        "formation":     { "type": "array", "items": { "type": "string" } },
        "type_decision": { "type": "array", "items": { "type": "string" } },
        "solution":      { "type": "array", "items": { "type": "string" } },
        "publication":   { "type": "array", "items": { "type": "string" } },
        "matiere":       { "type": "array", "items": { "type": "string" } },
        "localisation":  { "type": "array", "items": { "type": "string" } },
        "type_fichier":  { "type": "array", "items": { "type": "string" } },
        "date_debut":    { "type": "string" },
        "date_fin":      { "type": "string" },
        "type_date":     { "type": "string", "enum": ["creation","update"] },
        "ordre":         { "type": "string", "enum": ["asc","desc"], "default": "asc" },
        "taille_lot":    { "type": "integer", "minimum": 1, "maximum": 1000, "default": 10 },
        "abrege":        { "type": "boolean", "default": false },
        "interet_particulier": { "type": "boolean" }
    });

    if with_search_after {
        props["search_after"] = json!({ "type": "string",
            "description": "ID de la dernière décision du lot précédent pour paginer" });
    } else {
        props["lot"] = json!({ "type": "integer", "minimum": 0, "default": 0,
            "description": "Numéro du lot (commence à 0)" });
    }

    json!({ "type": "object", "properties": props })
}

fn fmt_batch_result(data: &Value, lot: Option<i64>, kind: &str) -> Result<String> {
    let total   = data["total"].as_i64().unwrap_or(0);
    let took    = data["took"].as_i64().unwrap_or(0);
    let results = data["results"].as_array().cloned().unwrap_or_default();

    if results.is_empty() {
        return Ok("Aucune décision trouvée pour ces critères.".into());
    }

    let lot_info = lot.map(|n| format!(" (lot n°{n})")).unwrap_or_default();
    let mut lines = vec![format!(
        "**{total} décision(s) au total** — {} dans ce lot{lot_info} — {took} ms\n",
        results.len()
    )];
    for (i, r) in results.iter().enumerate() {
        lines.push(format!("{}. {}", i + 1, fmt_decision_short(r)));
        lines.push(String::new());
    }

    match kind {
        "scan" => {
            if data["next_batch"].as_str().is_some() {
                if let Some(last_id) = results.last().and_then(|r| r["id"].as_str()) {
                    lines.push(format!(
                        "*Lot suivant disponible — utilisez `search_after='{last_id}'` pour continuer*"
                    ));
                }
            }
        }
        "export" => {
            if let Some(n) = lot {
                let taille = results.len() as i64;
                let nb_lots = (total + taille - 1) / taille;
                if n + 1 < nb_lots {
                    lines.push(format!(
                        "*Lot suivant disponible — utilisez `lot={}` pour continuer*", n + 1
                    ));
                }
            }
        }
        _ => {}
    }

    Ok(lines.join("\n"))
}

fn format_number(n: i64) -> String {
    // Séparateur milliers avec espace insécable
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 { result.push('\u{202F}'); }
        result.push(c);
    }
    result
}
