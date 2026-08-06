// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
use serde_json::Value;

// ---------------------------------------------------------------------------
// Résumé court (une décision)
// ---------------------------------------------------------------------------
pub fn fmt_decision_short(d: &Value) -> String {
    let rid      = d["id"].as_str().unwrap_or("");
    let juri     = d["jurisdiction"].as_str().unwrap_or("");
    let chamber  = d["chamber"].as_str().unwrap_or("");
    let number   = d["number"].as_str().unwrap_or("");
    let date     = d["decision_date"].as_str().unwrap_or("");
    let solution = d["solution"].as_str().unwrap_or("");
    let summary  = d["summary"].as_str().unwrap_or("");
    let ecli     = d["ecli"].as_str().unwrap_or("");
    let themes   = d["themes"].as_array();

    let mut lines = Vec::new();

    // Ligne titre
    let mut header = if number.is_empty() {
        "**Décision**".to_string()
    } else {
        format!("**{number}**")
    };
    if !date.is_empty()     { header.push_str(&format!(" — {date}")); }
    if !solution.is_empty() { header.push_str(&format!(" — *{solution}*")); }
    lines.push(header);

    // Métadonnées
    let mut meta = Vec::new();
    if !juri.is_empty()    { meta.push(juri.to_string()); }
    if !chamber.is_empty() { meta.push(chamber.to_string()); }
    if !ecli.is_empty()    { meta.push(format!("ECLI : {ecli}")); }
    if !rid.is_empty()     { meta.push(format!("ID : `{rid}`")); }
    if !meta.is_empty()    { lines.push(format!("  {}", meta.join(" | "))); }

    // Thèmes
    if let Some(th) = themes {
        let items: Vec<&str> = th.iter()
            .take(5)
            .filter_map(|v| v.as_str())
            .collect();
        if !items.is_empty() {
            lines.push(format!("  Matières : {}", items.join(", ")));
        }
    }

    // Sommaire
    if !summary.is_empty() {
        let truncated = if summary.len() > 300 {
            format!("{}…", &summary[..300])
        } else {
            summary.to_string()
        };
        lines.push(format!("  {truncated}"));
    }

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Contenu complet (une décision)
// ---------------------------------------------------------------------------
pub fn fmt_decision_full(d: &Value) -> String {
    let mut lines = vec![fmt_decision_short(d)];

    // Textes appliqués (visa)
    if let Some(visa) = d["visa"].as_array() {
        if !visa.is_empty() {
            lines.push(format!("\n**Textes appliqués ({})** :", visa.len()));
            for v in visa.iter().take(10) {
                let title = v["title"].as_str().unwrap_or("");
                lines.push(format!("  - {title}"));
            }
        }
    }

    // Rapprochements
    if let Some(rappr) = d["rapprochements"].as_array() {
        if !rappr.is_empty() {
            lines.push(format!("\n**Rapprochements ({})** :", rappr.len()));
            for r in rappr.iter().take(5) {
                let title  = r["title"].as_str().unwrap_or("");
                let number = r["number"].as_str().unwrap_or("");
                lines.push(format!("  - {title} ({number})"));
            }
        }
    }

    // Documents associés
    if let Some(files) = d["files"].as_array() {
        if !files.is_empty() {
            lines.push(format!("\n**Documents associés ({})** :", files.len()));
            for f in files {
                let name  = f["name"].as_str().unwrap_or("");
                let ftype = f["type"].as_str().unwrap_or("");
                lines.push(format!("  - {name} ({ftype})"));
            }
        }
    }

    // Texte intégral
    let text = d["text"].as_str()
        .or_else(|| d["text_highlight"].as_str())
        .unwrap_or("");
    if !text.is_empty() {
        let total_len = text.len();
        let excerpt = if total_len > 6000 { &text[..6000] } else { text };
        lines.push(format!("\n**Texte intégral** :\n\n{excerpt}"));
        if total_len > 6000 {
            lines.push(format!(
                "\n*[Texte tronqué — {total_len} caractères au total. \
                 Consultez judilibre.fr pour le texte intégral]*"
            ));
        }
    }

    lines.join("\n")
}
