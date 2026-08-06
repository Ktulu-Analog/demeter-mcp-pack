// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
use serde_json::Value;

// ---------------------------------------------------------------------------
// Strip HTML basique
// ---------------------------------------------------------------------------
pub fn strip_html(text: &str) -> String {
    if text.is_empty() { return String::new(); }
    let mut s = text.to_string();
    s = s.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
    // <p ...> → \n
    let re_p_open  = regex_replace(&s, r"<p[^>]*>", "\n");
    let re_p_close = re_p_open.replace("</p>", "");
    // Supprimer toutes les autres balises
    let no_tags = remove_tags(&re_p_close);
    // Réduire les sauts de ligne multiples
    let mut result = String::new();
    let mut blank_count = 0u32;
    for line in no_tags.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count < 2 { result.push('\n'); }
        } else {
            blank_count = 0;
            result.push_str(line);
            result.push('\n');
        }
    }
    result.trim().to_string()
}

fn regex_replace(s: &str, pat: &str, replacement: &str) -> String {
    // Implémentation manuelle simple sans crate regex
    // Cherche <p suivi de > en ignorant les attributs
    if pat == r"<p[^>]*>" {
        let mut result = String::new();
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i+1] == b'p' {
                // Vérifier si c'est bien une balise <p...>
                let mut j = i + 2;
                // autoriser <p> ou <p attribut>
                if j >= bytes.len() || (bytes[j] != b'>' && bytes[j] != b' ') {
                    result.push(bytes[i] as char);
                    i += 1;
                    continue;
                }
                while j < bytes.len() && bytes[j] != b'>' {
                    j += 1;
                }
                if j < bytes.len() {
                    result.push_str(replacement);
                    i = j + 1;
                    continue;
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        }
        result
    } else {
        s.to_string()
    }
}

fn remove_tags(s: &str) -> String {
    let mut result = String::new();
    let mut inside = false;
    for c in s.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _   => if !inside { result.push(c); }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Conversion date : timestamp ms (i64) ou string ISO → string lisible
// ---------------------------------------------------------------------------
pub fn fmt_date(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(ms) = n.as_i64() {
                // timestamp ms → YYYY-MM-DD
                let secs = ms / 1000;
                // Calcul de date sans chrono : approximation (≥ 1970)
                ts_to_date(secs)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

fn ts_to_date(secs: i64) -> String {
    // Algorithme de conversion timestamp → date civile (depuis 1970-01-01)
    if secs < 0 { return String::new(); }
    let days_since_epoch = secs / 86400;
    let (y, m, d) = days_to_ymd(days_since_epoch as u32);
    format!("{y:04}-{m:02}-{d:02}")
}

fn days_to_ymd(mut z: u32) -> (u32, u32, u32) {
    z += 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
    let y   = yoe + era * 400;
    let doy = doe - (365*yoe + yoe/4 - yoe/100);
    let mp  = (5*doy + 2) / 153;
    let d   = doy - (153*mp + 2)/5 + 1;
    let m   = if mp < 10 { mp + 3 } else { mp - 9 };
    let y   = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ---------------------------------------------------------------------------
// Formatage d'un article
// ---------------------------------------------------------------------------
pub fn fmt_article(a: &Value) -> String {
    if a.is_null() { return String::new(); }
    let num    = a["num"].as_str()
        .or_else(|| a["numero"].as_str())
        .or_else(|| a["number"].as_str())
        .unwrap_or("");
    let titre  = a["titre"].as_str().or_else(|| a["title"].as_str()).unwrap_or("");
    let raw    = a["texteHtml"].as_str()
        .or_else(|| a["texte"].as_str())
        .or_else(|| a["content"].as_str())
        .unwrap_or("");
    let texte  = strip_html(raw);
    let rid    = a["id"].as_str().unwrap_or("");
    let etat   = a["etat"].as_str().or_else(|| a["legalStatus"].as_str()).unwrap_or("");
    let debut  = a["dateDebut"].as_str().or_else(|| a["dateVigueur"].as_str()).unwrap_or("");

    let mut lines = Vec::new();
    let mut header = if num.is_empty() {
        "### Article".to_string()
    } else {
        format!("### Article {num}")
    };
    if !titre.is_empty() { header.push_str(&format!(" — {titre}")); }
    lines.push(header);
    if !etat.is_empty()  { lines.push(format!("*État : {etat}*")); }
    if !debut.is_empty() { lines.push(format!("*En vigueur depuis : {debut}*")); }
    if !rid.is_empty()   { lines.push(format!("*ID : `{rid}`*")); }
    if !texte.is_empty() {
        let excerpt = if texte.len() > 3000 { &texte[..3000] } else { &texte };
        lines.push(format!("\n{excerpt}"));
        if texte.len() > 3000 {
            lines.push("\n*[Texte tronqué à 3000 caractères]*".to_string());
        }
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Formatage d'un résultat de recherche
// ---------------------------------------------------------------------------
pub fn fmt_search(data: &Value, query: &str) -> String {
    let results = match data["results"].as_array() {
        Some(r) => r,
        None    => return format!("Aucun résultat pour : « {query} »"),
    };
    if results.is_empty() {
        return format!("Aucun résultat pour : « {query} »");
    }
    let total = data["totalResultNumber"].as_i64().unwrap_or(0);
    let mut lines = vec![format!("{total} résultat(s) — {} affiché(s)\n", results.len())];
    for (i, r) in results.iter().enumerate() {
        let t_list = r["titles"].as_array();
        let first  = t_list.and_then(|a| a.first()).unwrap_or(r);
        let title  = first["title"].as_str()
            .or_else(|| r["title"].as_str())
            .unwrap_or("Sans titre");
        let rid    = first["cid"].as_str()
            .or_else(|| first["id"].as_str())
            .or_else(|| r["cid"].as_str())
            .or_else(|| r["id"].as_str())
            .unwrap_or("");
        let date_p = first["datePubliTexte"].as_str().unwrap_or("");
        let title  = if title.len() > 120 { format!("{}…", &title[..120]) } else { title.to_string() };
        lines.push(format!("**{}**. {title}", i + 1));
        if !date_p.is_empty() { lines.push(format!("   Date : {date_p}")); }
        if !rid.is_empty()    { lines.push(format!("   ID : `{rid}`")); }
        lines.push(String::new());
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Table des matières
// ---------------------------------------------------------------------------
pub fn fmt_toc(data: &Value, code: &str) -> String {
    let mut lines = vec![format!("# Table des matières — {code}\n")];
    walk_toc(
        data["sections"].as_array()
            .or_else(|| data["articles"].as_array()),
        &mut lines,
        0,
    );
    if lines.len() > 1 { lines.join("\n") } else { format!("Structure vide pour {code}.") }
}

fn walk_toc(items: Option<&Vec<Value>>, lines: &mut Vec<String>, depth: usize) {
    let items = match items { Some(v) => v, None => return };
    let indent = "  ".repeat(depth);
    for item in items {
        let t   = item["titre"].as_str().or_else(|| item["title"].as_str()).unwrap_or("");
        let rid = item["id"].as_str().or_else(|| item["cid"].as_str()).unwrap_or("");
        if !t.is_empty() {
            if depth == 0 {
                lines.push(format!("\n## {t}"));
            } else {
                let id_str = if !rid.is_empty() { format!(" (`{rid}`)") } else { String::new() };
                lines.push(format!("{indent}- {t}{id_str}"));
            }
        }
        if depth < 3 {
            let children = item["sections"].as_array()
                .or_else(|| item["articles"].as_array())
                .or_else(|| item["children"].as_array());
            walk_toc(children, lines, depth + 1);
        }
    }
}

// ---------------------------------------------------------------------------
// today() → "YYYY-MM-DD"
// ---------------------------------------------------------------------------
pub fn today() -> String {
    // Calcul depuis UNIX_EPOCH sans dépendance chrono
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    ts_to_date(secs)
}
