// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ----------------------------------------------------------------------------
// Formatage des réponses de l'API Légifrance en Markdown lisible par un LLM.
// Les noms de champs lus ici suivent le swagger officiel (définitions
// Article, ConsultArticle, ConsultSection, ConsultTextResponse, TexteSimple…).
// ============================================================================
use serde_json::Value;

// ---------------------------------------------------------------------------
// Troncature sûre (UTF-8) — ne JAMAIS slicer une &str par octets : les textes
// juridiques français contiennent des caractères multi-octets (é, è, œ, «, »…)
// et `&s[..n]` panique si `n` tombe au milieu d'un caractère.
// ---------------------------------------------------------------------------
pub fn truncate_chars(s: &str, max: usize) -> (String, bool) {
    if s.chars().count() <= max {
        return (s.to_string(), false);
    }
    (s.chars().take(max).collect(), true)
}

// ---------------------------------------------------------------------------
// Nettoyage HTML (conserve l'UTF-8, décode les entités courantes)
// ---------------------------------------------------------------------------
pub fn strip_html(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '<' {
            out.push(c);
            continue;
        }
        // Lecture de la balise jusqu'à '>'
        let mut tag = String::new();
        let mut closed = false;
        for t in chars.by_ref() {
            if t == '>' {
                closed = true;
                break;
            }
            tag.push(t);
        }
        if !closed {
            // '<' isolé (ex. « a < b ») : on le restitue tel quel.
            out.push('<');
            out.push_str(&tag);
            break;
        }
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        let closing = tag.starts_with('/');
        match name.as_str() {
            "br" | "p" | "div" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "table"
            | "ul" | "ol" | "blockquote" => out.push('\n'),
            "li" => {
                if closing {
                    out.push('\n');
                } else {
                    out.push_str("\n- ");
                }
            }
            "td" | "th" => {
                if closing {
                    out.push_str(" | ");
                }
            }
            _ => {}
        }
    }
    let decoded = decode_entities(&out);
    // Réduction des lignes vides multiples + trim des fins de ligne.
    let mut result = String::new();
    let mut blank = 0u32;
    for line in decoded.lines() {
        let l = line.trim_end();
        if l.trim().is_empty() {
            blank += 1;
            if blank < 2 && !result.is_empty() {
                result.push('\n');
            }
        } else {
            blank = 0;
            result.push_str(l);
            result.push('\n');
        }
    }
    result.trim().to_string()
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        // Entité : &xxx; (≤ 10 caractères)
        if let Some(end) = tail.find(';').filter(|e| *e <= 10) {
            let ent = &tail[1..end];
            let rep: Option<String> = match ent {
                "nbsp" => Some(" ".into()),
                "amp" => Some("&".into()),
                "lt" => Some("<".into()),
                "gt" => Some(">".into()),
                "quot" => Some("\"".into()),
                "apos" => Some("'".into()),
                "laquo" => Some("«".into()),
                "raquo" => Some("»".into()),
                "hellip" => Some("…".into()),
                "ndash" => Some("–".into()),
                "mdash" => Some("—".into()),
                "rsquo" | "lsquo" => Some("'".into()),
                "oelig" => Some("œ".into()),
                "euro" => Some("€".into()),
                _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                    u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32).map(|c| c.to_string())
                }
                _ if ent.starts_with('#') => {
                    ent[1..].parse::<u32>().ok().and_then(char::from_u32).map(|c| c.to_string())
                }
                _ => None,
            };
            if let Some(r) = rep {
                out.push_str(&r);
                rest = &tail[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = &tail[1..];
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// Dates : timestamp ms (i64, éventuellement négatif), ISO 8601 ou "YYYY-MM-DD"
// ---------------------------------------------------------------------------
pub fn fmt_date(v: &Value) -> String {
    match v {
        Value::String(s) => {
            let t = s.trim();
            // "2024-01-31T00:00:00.000+0000" → "2024-01-31"
            if t.len() >= 10 && t.as_bytes().get(4) == Some(&b'-') && t.as_bytes().get(7) == Some(&b'-') {
                t.chars().take(10).collect()
            } else {
                t.to_string()
            }
        }
        Value::Number(n) => n.as_i64().map(|ms| ts_to_date(ms.div_euclid(1000))).unwrap_or_default(),
        _ => String::new(),
    }
}

/// Date « sans fin » : l'API utilise 2999-01-01 comme borne ouverte.
pub fn is_open_end(date: &str) -> bool {
    date.starts_with("2999") || date.starts_with("9999")
}

fn ts_to_date(secs: i64) -> String {
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// "YYYY-MM-DD" → timestamp en millisecondes (UTC, minuit).
pub fn ymd_to_ms(s: &str) -> Option<i64> {
    let p: Vec<&str> = s.get(..10)?.split('-').collect();
    if p.len() != 3 {
        return None;
    }
    let (y, m, d) = (p[0].parse::<i64>().ok()?, p[1].parse::<u32>().ok()?, p[2].parse::<u32>().ok()?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d) * 86_400_000)
}

/// Timestamp ms ou date ISO → ms (pour comparer des bornes de versions).
pub fn value_to_ms(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse::<i64>().ok().or_else(|| ymd_to_ms(s)),
        _ => None,
    }
}

pub fn today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    ts_to_date(secs)
}

// ---------------------------------------------------------------------------
// Utilitaires
// ---------------------------------------------------------------------------
pub fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

fn first_str<'a>(v: &'a Value, keys: &[&str]) -> &'a str {
    for k in keys {
        if let Some(x) = v[*k].as_str() {
            if !x.is_empty() {
                return x;
            }
        }
    }
    ""
}

fn push_kv(lines: &mut Vec<String>, label: &str, value: &str) {
    if !value.trim().is_empty() {
        lines.push(format!("**{label}** : {value}"));
    }
}

/// Texte (HTML ou brut) nettoyé et tronqué, avec mention de troncature.
fn body_text(raw: &str, max: usize) -> Option<String> {
    let t = strip_html(raw);
    if t.is_empty() {
        return None;
    }
    let (ex, cut) = truncate_chars(&t, max);
    Some(if cut {
        format!("{ex}\n\n*[Texte tronqué à {max} caractères]*")
    } else {
        ex
    })
}

/// Tableau de chaînes → "a, b, c"
fn join_strs(v: &Value) -> String {
    v.as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", "))
        .unwrap_or_default()
}

pub const ARTICLE_MAX: usize = 12_000;
pub const TEXT_MAX: usize = 40_000;

// ---------------------------------------------------------------------------
// Article (Article / ConsultArticle)
// ---------------------------------------------------------------------------
pub fn fmt_article(a: &Value) -> String {
    fmt_article_max(a, ARTICLE_MAX)
}

pub fn fmt_article_max(a: &Value, max: usize) -> String {
    if a.is_null() {
        return String::new();
    }
    let num = first_str(a, &["num", "numero", "number"]);
    let titre = first_str(a, &["titre", "title", "sectionParentTitre"]);
    let raw = first_str(a, &["texteHtml", "texte", "content"]);
    let etat = first_str(a, &["etat", "legalStatus"]);
    let debut = fmt_date(&a["dateDebut"]);
    let fin = fmt_date(&a["dateFin"]);
    let rid = s(a, "id");
    let cid = s(a, "cid");

    let mut lines = Vec::new();
    let mut header = if num.is_empty() { "### Article".to_string() } else { format!("### Article {num}") };
    if !titre.is_empty() {
        header.push_str(&format!(" — {titre}"));
    }
    lines.push(header);
    if !etat.is_empty() {
        lines.push(format!("*État : {etat}*"));
    }
    if !debut.is_empty() {
        let fin_txt = if fin.is_empty() || is_open_end(&fin) { "en vigueur".to_string() } else { fin };
        lines.push(format!("*Version : {debut} → {fin_txt}*"));
    }
    if !rid.is_empty() {
        lines.push(format!("*ID : `{rid}`*"));
    }
    if !cid.is_empty() && cid != rid {
        lines.push(format!("*CID : `{cid}`*"));
    }
    if let Some(t) = body_text(raw, max) {
        lines.push(format!("\n{t}"));
    }
    if let Some(n) = body_text(first_str(a, &["notaHtml", "nota"]), 3_000) {
        lines.push(format!("\n**Nota** : {n}"));
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Résultat de recherche (SearchResponseDTO)
// ---------------------------------------------------------------------------
pub fn fmt_search(data: &Value, query: &str) -> String {
    let results = match data["results"].as_array() {
        Some(r) if !r.is_empty() => r,
        _ => return format!("Aucun résultat pour : « {query} »"),
    };
    let total = data["totalResultNumber"].as_i64().unwrap_or(results.len() as i64);
    let mut head = format!("{total} résultat(s) — {} affiché(s)", results.len());
    if let Some(n) = data["totalArticleResultNumber"].as_i64().filter(|n| *n > 0) {
        head.push_str(&format!(" ({n} article(s) correspondants)"));
    }
    let mut lines = vec![head + "\n"];
    for (i, r) in results.iter().enumerate() {
        let first = r["titles"].as_array().and_then(|a| a.first()).unwrap_or(r);
        let title = first_str(first, &["title"]);
        let title = if title.is_empty() { first_str(r, &["title"]) } else { title };
        let title = if title.is_empty() { "Sans titre" } else { title };
        let rid = first_str(first, &["cid", "id"]);
        let rid = if rid.is_empty() { first_str(r, &["cid", "id"]) } else { rid };
        let (title, cut) = truncate_chars(title, 160);
        let title = if cut { format!("{title}…") } else { title };
        let date = [
            first_str(first, &["datePubliTexte"]),
            first_str(r, &["dateSignature", "datePublication", "date", "dateDiffusion"]),
        ]
        .into_iter()
        .find(|d| !d.is_empty())
        .unwrap_or("");
        lines.push(format!("**{}**. {title}", i + 1));
        let mut meta = Vec::new();
        for (k, label) in [("nature", ""), ("type", ""), ("etat", "état "), ("num", "n° "), ("nor", "NOR "), ("idcc", "IDCC "), ("origin", "")] {
            let v = s(r, k);
            if !v.is_empty() {
                meta.push(format!("{label}{v}"));
            }
        }
        if !meta.is_empty() {
            lines.push(format!("   {}", meta.join(" · ")));
        }
        if !date.is_empty() {
            lines.push(format!("   Date : {}", fmt_date(&Value::String(date.to_string()))));
        }
        if !rid.is_empty() {
            lines.push(format!("   ID : `{rid}`"));
        }
        // Un résultat peut regrouper plusieurs articles/sections.
        if let Some(secs) = r["sections"].as_array() {
            for sec in secs.iter().take(3) {
                let st = s(sec, "title");
                let sid = first_str(sec, &["id", "cid"]);
                if !st.is_empty() {
                    let (st, _) = truncate_chars(st, 120);
                    lines.push(format!("   ↳ {st}{}", if sid.is_empty() { String::new() } else { format!(" — `{sid}`") }));
                }
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Table des matières (ConsultTextResponse / ConsultSection / ConsultKaliContResponse)
// ---------------------------------------------------------------------------
pub fn fmt_toc(data: &Value, label: &str) -> String {
    fmt_toc_opts(data, label, 6, false, 800)
}

pub fn fmt_toc_opts(data: &Value, label: &str, max_depth: usize, with_articles: bool, max_lines: usize) -> String {
    let mut lines = vec![format!("# Table des matières — {label}\n")];
    let mut truncated = false;
    let sections = data["sections"].as_array();
    let has_sections = sections.map(|s| !s.is_empty()).unwrap_or(false);
    walk_toc(sections, &mut lines, 0, max_depth, with_articles, max_lines, &mut truncated);
    // Texte sans sections : on liste ses articles à plat.
    if !has_sections {
        walk_articles(data["articles"].as_array(), &mut lines, 0, max_lines, &mut truncated);
    }
    if truncated {
        lines.push(format!("\n*… arborescence tronquée à {max_lines} lignes — utilisez legifrance_section_par_cid pour descendre dans une section.*"));
    }
    if lines.len() > 1 {
        lines.join("\n")
    } else {
        format!("Structure vide pour {label}.")
    }
}

fn walk_articles(items: Option<&Vec<Value>>, lines: &mut Vec<String>, depth: usize, max_lines: usize, truncated: &mut bool) {
    let Some(items) = items else { return };
    let indent = "  ".repeat(depth);
    for a in items {
        if lines.len() >= max_lines {
            *truncated = true;
            return;
        }
        let num = first_str(a, &["num", "numero"]);
        let id = first_str(a, &["id", "cid"]);
        let etat = s(a, "etat");
        let mut l = format!("{indent}- Art. {}", if num.is_empty() { "?" } else { num });
        if !id.is_empty() {
            l.push_str(&format!(" (`{id}`)"));
        }
        if !etat.is_empty() {
            l.push_str(&format!(" [{etat}]"));
        }
        lines.push(l);
    }
}

fn walk_toc(items: Option<&Vec<Value>>, lines: &mut Vec<String>, depth: usize, max_depth: usize, with_articles: bool, max_lines: usize, truncated: &mut bool) {
    let Some(items) = items else { return };
    let indent = "  ".repeat(depth);
    for item in items {
        if lines.len() >= max_lines {
            *truncated = true;
            return;
        }
        let t = first_str(item, &["title", "titre"]);
        let rid = first_str(item, &["cid", "id"]);
        if !t.is_empty() {
            if depth == 0 {
                lines.push(format!("\n## {t}{}", if rid.is_empty() { String::new() } else { format!(" (`{rid}`)") }));
            } else {
                let id_str = if rid.is_empty() { String::new() } else { format!(" (`{rid}`)") };
                lines.push(format!("{indent}- {t}{id_str}"));
            }
        }
        if with_articles {
            walk_articles(item["articles"].as_array(), lines, depth + 1, max_lines, truncated);
        }
        if depth + 1 < max_depth {
            walk_toc(item["sections"].as_array().or_else(|| item["children"].as_array()), lines, depth + 1, max_depth, with_articles, max_lines, truncated);
        }
    }
}

// ---------------------------------------------------------------------------
// Texte complet structuré (ConsultTextResponse : sections imbriquées + articles)
// ---------------------------------------------------------------------------
pub fn fmt_legi_text(data: &Value, id: &str) -> String {
    let titre = first_str(data, &["title", "titre"]);
    let mut lines = vec![format!("# {}\n", if titre.is_empty() { id } else { titre })];
    push_kv(&mut lines, "Nature", s(data, "nature"));
    push_kv(&mut lines, "État", s(data, "etat"));
    push_kv(&mut lines, "NOR", s(data, "nor"));
    push_kv(&mut lines, "ELI", s(data, "eli"));
    push_kv(&mut lines, "Numéro", first_str(data, &["textNumber", "numParution"]));
    let dv = fmt_date(&data["dateDebutVersion"]);
    let df = fmt_date(&data["dateFinVersion"]);
    if !dv.is_empty() {
        let df_txt = if df.is_empty() || is_open_end(&df) { "en vigueur".to_string() } else { df };
        push_kv(&mut lines, "Version", &format!("{dv} → {df_txt}"));
    }
    push_kv(&mut lines, "Date du texte", &fmt_date(&data["dateTexte"]));
    push_kv(&mut lines, "Date de parution", &fmt_date(&data["dateParution"]));
    lines.push(format!("**ID** : `{}`", if s(data, "id").is_empty() { id } else { s(data, "id") }));
    if let Some(v) = body_text(s(data, "visa"), 2_000) {
        lines.push(format!("\n**Visas** : {v}"));
    }
    if let Some(v) = body_text(first_str(data, &["notice", "resume"]), 3_000) {
        lines.push(format!("\n**Notice** : {v}"));
    }

    let mut budget = TEXT_MAX as i64;
    let mut n_art = 0usize;
    let mut cut = false;
    // Articles à la racine, puis sections récursives.
    emit_articles(data["articles"].as_array(), &mut lines, &mut budget, &mut n_art, &mut cut);
    emit_sections(data["sections"].as_array(), &mut lines, 0, &mut budget, &mut n_art, &mut cut);
    if n_art == 0 && lines.len() < 8 {
        lines.push("\n*(Aucun article dans la réponse — utilisez legifrance_consulter_code ou legifrance_section_par_cid pour naviguer.)*".into());
    }
    if cut {
        lines.push(format!("\n*[Texte tronqué à ~{TEXT_MAX} caractères — {n_art} article(s) affichés. Utilisez legifrance_consulter_code / legifrance_section_par_cid / legifrance_obtenir_article pour la suite.]*"));
    }
    lines.join("\n")
}

fn emit_articles(items: Option<&Vec<Value>>, lines: &mut Vec<String>, budget: &mut i64, n: &mut usize, cut: &mut bool) {
    let Some(items) = items else { return };
    for a in items {
        if *budget <= 0 {
            *cut = true;
            return;
        }
        let rendered = fmt_article_max(a, 4_000);
        *budget -= rendered.chars().count() as i64;
        *n += 1;
        lines.push(rendered);
        lines.push(String::new());
    }
}

fn emit_sections(items: Option<&Vec<Value>>, lines: &mut Vec<String>, depth: usize, budget: &mut i64, n: &mut usize, cut: &mut bool) {
    let Some(items) = items else { return };
    for sec in items {
        if *budget <= 0 {
            *cut = true;
            return;
        }
        let t = first_str(sec, &["title", "titre"]);
        if !t.is_empty() {
            lines.push(format!("\n{} {t}", "#".repeat((depth + 2).min(6))));
            *budget -= t.chars().count() as i64;
        }
        emit_articles(sec["articles"].as_array(), lines, budget, n, cut);
        emit_sections(sec["sections"].as_array(), lines, depth + 1, budget, n, cut);
    }
}

// ---------------------------------------------------------------------------
// Texte « simple » (jurisprudence JURI/CETAT/CONSTIT, CNIL) — TexteSimple
// ---------------------------------------------------------------------------
pub fn fmt_texte_simple(t: &Value, id: &str) -> String {
    let titre = first_str(t, &["titre", "titreLong", "titreJo"]);
    let mut lines = vec![format!("# {}\n", if titre.is_empty() { id } else { titre })];
    push_kv(&mut lines, "Nature", first_str(t, &["nature", "natureQualifiee", "typeTexte"]));
    push_kv(&mut lines, "Juridiction", first_str(t, &["juridiction", "natureJuridiction", "juridictionJudiciaire", "autorite"]));
    push_kv(&mut lines, "Formation", s(t, "formation"));
    push_kv(&mut lines, "Siège", s(t, "siegeAppel"));
    push_kv(&mut lines, "Date", &fmt_date(&t["dateTexte"]));
    push_kv(&mut lines, "Numéro d'affaire", &join_strs(&t["numeroAffaire"]));
    push_kv(&mut lines, "Numéro", first_str(t, &["num", "numJo"]));
    push_kv(&mut lines, "ECLI", s(t, "ecli"));
    push_kv(&mut lines, "Solution", s(t, "solution"));
    push_kv(&mut lines, "Nature de la délibération", s(t, "natureDelib"));
    push_kv(&mut lines, "NOR", s(t, "nor"));
    push_kv(&mut lines, "Publication", first_str(t, &["publicationRecueil", "typePublicationBulletin"]));
    push_kv(&mut lines, "Président", s(t, "president"));
    push_kv(&mut lines, "Rapporteur", s(t, "rapporteur"));
    push_kv(&mut lines, "Avocats", s(t, "avocats"));
    push_kv(&mut lines, "Mots-clés", &join_strs(&t["motsCles"]));
    push_kv(&mut lines, "Ancien ID", s(t, "ancienId"));
    lines.push(format!("**ID** : `{}`", if s(t, "id").is_empty() { id } else { s(t, "id") }));
    if let Some(r) = body_text(first_str(t, &["resumeHtml", "resume"]), 3_000) {
        lines.push(format!("\n**Sommaire / résumé** :\n{r}"));
    }
    if let Some(v) = body_text(first_str(t, &["visasHtml", "visas"]), 3_000) {
        lines.push(format!("\n**Visas** : {v}"));
    }
    if let Some(body) = body_text(first_str(t, &["texteHtml", "texte"]), TEXT_MAX) {
        lines.push(format!("\n{body}"));
    }
    lines.join("\n")
}

/// Accord d'entreprise (Accord)
pub fn fmt_acco(a: &Value, id: &str) -> String {
    let titre = first_str(a, &["titreTexte", "raisonSociale"]);
    let mut lines = vec![format!("# {}\n", if titre.is_empty() { id } else { titre })];
    push_kv(&mut lines, "Raison sociale", s(a, "raisonSociale"));
    push_kv(&mut lines, "SIRET", s(a, "siret"));
    push_kv(&mut lines, "IDCC", s(a, "codeIdcc"));
    push_kv(&mut lines, "Code APE", s(a, "codeApe"));
    push_kv(&mut lines, "Secteur", s(a, "secteur"));
    push_kv(&mut lines, "Nature", s(a, "nature"));
    push_kv(&mut lines, "Numéro", s(a, "numero"));
    push_kv(&mut lines, "Date de signature", &fmt_date(&a["dateTexte"]));
    push_kv(&mut lines, "Date de dépôt", &fmt_date(&a["dateDepot"]));
    push_kv(&mut lines, "Date de diffusion", &fmt_date(&a["dateDiffusion"]));
    push_kv(&mut lines, "Date d'effet", &fmt_date(&a["dateEffet"]));
    push_kv(&mut lines, "Date de fin", &fmt_date(&a["dateFin"]));
    push_kv(&mut lines, "Signataires", &join_strs(&a["signataires"]));
    push_kv(&mut lines, "Fichier", first_str(a, &["attachementUrl", "url"]));
    lines.push(format!("**ID** : `{}`", if s(a, "id").is_empty() { id } else { s(a, "id") }));
    let content = first_str(&a["attachment"], &["content"]);
    let content = if content.is_empty() { s(a, "data") } else { content };
    if let Some(b) = body_text(content, TEXT_MAX) {
        lines.push(format!("\n{b}"));
    }
    lines.join("\n")
}

/// Circulaire (Circulaire)
pub fn fmt_circulaire(c: &Value, id: &str) -> String {
    let titre = s(c, "titre");
    let mut lines = vec![format!("# {}\n", if titre.is_empty() { id } else { titre })];
    push_kv(&mut lines, "NOR", s(c, "nor"));
    push_kv(&mut lines, "Numéro interne", s(c, "numeroInterne"));
    push_kv(&mut lines, "État", s(c, "etat"));
    push_kv(&mut lines, "Opposable", s(c, "opposable"));
    push_kv(&mut lines, "Date de signature", &fmt_date(&c["dateSignature"]));
    push_kv(&mut lines, "Date de mise en application", &fmt_date(&c["dateMiseApplication"]));
    push_kv(&mut lines, "Ministères déposants", &join_strs(&c["ministeresDeposants"]));
    push_kv(&mut lines, "Domaines", &join_strs(&c["domaines"]));
    push_kv(&mut lines, "Mots-clés", &join_strs(&c["motsCles"]));
    push_kv(&mut lines, "Publication JO/BO", s(c, "referencePublicationJoBo"));
    push_kv(&mut lines, "Fichier", s(c, "attachmentUrl"));
    lines.push(format!("**ID** : `{}`", if s(c, "id").is_empty() { id } else { s(c, "id") }));
    if let Some(r) = body_text(s(c, "resume"), 3_000) {
        lines.push(format!("\n**Résumé** : {r}"));
    }
    let content = first_str(&c["attachment"], &["content"]);
    let content = if content.is_empty() { s(c, "data") } else { content };
    if let Some(b) = body_text(content, TEXT_MAX) {
        lines.push(format!("\n{b}"));
    }
    lines.join("\n")
}

/// Dossier législatif (DossierLegislatif)
pub fn fmt_dossier(d: &Value, id: &str) -> String {
    let titre = s(d, "titre");
    let mut lines = vec![format!("# {}\n", if titre.is_empty() { id } else { titre })];
    push_kv(&mut lines, "Type", s(d, "type"));
    push_kv(&mut lines, "Nature", s(d, "nature"));
    push_kv(&mut lines, "Législature", s(&d["legislature"], "libelle"));
    push_kv(&mut lines, "Création", &fmt_date(&d["dateCreation"]));
    push_kv(&mut lines, "Dernière modification", &fmt_date(&d["dateDerniereModification"]));
    push_kv(&mut lines, "URL", s(d, "url"));
    lines.push(format!("**ID** : `{}`", if s(d, "id").is_empty() { id } else { s(d, "id") }));
    if let Some(e) = body_text(s(d, "exposeMotif"), 4_000) {
        lines.push(format!("\n**Exposé des motifs** : {e}"));
    }
    if let Some(items) = d["dossiers"].as_array() {
        if !items.is_empty() {
            lines.push(format!("\n**{} dossier(s) associé(s)**", items.len()));
            for it in items.iter().take(40) {
                let t = first_str(it, &["titre", "title", "libelle"]);
                let i = s(it, "id");
                let ty = s(it, "type");
                lines.push(format!("- {}{}{}", if t.is_empty() { "(sans titre)" } else { t }, if ty.is_empty() { String::new() } else { format!(" [{ty}]") }, if i.is_empty() { String::new() } else { format!(" — `{i}`") }));
            }
        }
    }
    lines.join("\n")
}

/// Débat parlementaire (Debat)
pub fn fmt_debat(d: &Value, id: &str) -> String {
    let mut lines = vec![format!("# Débat parlementaire `{}`\n", if s(d, "id").is_empty() { id } else { s(d, "id") })];
    push_kv(&mut lines, "Séance du", &fmt_date(&d["dateSeance"]));
    push_kv(&mut lines, "Parution", &fmt_date(&d["dateParution"]));
    push_kv(&mut lines, "Assemblée", s(d, "typeAssemblee"));
    if let Some(l) = d["legislature"].as_i64() {
        push_kv(&mut lines, "Législature", &l.to_string());
    }
    push_kv(&mut lines, "Session", first_str(d, &["nomSession", "session"]));
    push_kv(&mut lines, "Fichier", first_str(d, &["attachmentUrl", "pathToFile"]));
    let content = first_str(&d["attachment"], &["content"]);
    let content = if content.is_empty() { s(d, "data") } else { content };
    if let Some(b) = body_text(content, TEXT_MAX) {
        lines.push(format!("\n{b}"));
    }
    lines.join("\n")
}

/// Liste de sections (GetListSectionResponse.listSection)
pub fn fmt_list_section(data: &Value, id: &str) -> String {
    let secs = data["listSection"].as_array().cloned().unwrap_or_default();
    if secs.is_empty() {
        return format!("Aucune section trouvée pour `{id}`.");
    }
    let mut lines = Vec::new();
    for sec in &secs {
        let t = first_str(sec, &["titre", "title"]);
        lines.push(format!("## {}", if t.is_empty() { id } else { t }));
        let (d, f) = (fmt_date(&sec["dateDebut"]), fmt_date(&sec["dateFin"]));
        if !d.is_empty() {
            lines.push(format!("*Version : {d} → {}*", if f.is_empty() || is_open_end(&f) { "en vigueur".to_string() } else { f }));
        }
        lines.push(format!("*ID : `{}`*", first_str(sec, &["cid", "id"])));
        if let Some(n) = body_text(first_str(sec, &["notaHtml", "nota"]), 2_000) {
            lines.push(format!("**Nota** : {n}"));
        }
        if let Some(ls) = sec["liensSection"].as_array() {
            if !ls.is_empty() {
                lines.push("\n**Sous-sections**".into());
                for l in ls.iter().take(200) {
                    let etat = s(l, "etat");
                    lines.push(format!("- {} (`{}`){}", first_str(l, &["titre"]), first_str(l, &["cid", "id"]), if etat.is_empty() { String::new() } else { format!(" [{etat}]") }));
                }
            }
        }
        if let Some(la) = sec["liensArticle"].as_array() {
            if !la.is_empty() {
                lines.push("\n**Articles**".into());
                for l in la.iter().take(300) {
                    let etat = s(l, "etat");
                    lines.push(format!("- `{}`{}", s(l, "id"), if etat.is_empty() { String::new() } else { format!(" [{etat}]") }));
                }
                lines.push("\n*(Utilisez legifrance_obtenir_article avec ces identifiants pour lire le contenu.)*".into());
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

/// Versions d'un article (GetListArticleResponse.listArticle ou Article.articleVersions)
pub fn fmt_article_versions(versions: &[Value], id: &str) -> String {
    if versions.is_empty() {
        return format!("Aucune version trouvée pour `{id}`.");
    }
    let mut sorted: Vec<&Value> = versions.iter().collect();
    sorted.sort_by_key(|v| value_to_ms(&v["dateDebut"]).unwrap_or(0));
    let mut lines = vec![format!("**{} version(s) de `{id}`** (de la plus ancienne à la plus récente)\n", sorted.len())];
    for (i, v) in sorted.iter().enumerate() {
        let debut = fmt_date(&v["dateDebut"]);
        let fin = fmt_date(&v["dateFin"]);
        let etat = first_str(v, &["etat"]);
        let vid = s(v, "id");
        let num = first_str(v, &["num", "numero"]);
        let range = match (debut.is_empty(), fin.is_empty() || is_open_end(&fin)) {
            (false, false) => format!("{debut} → {fin}"),
            (false, true) => format!("{debut} → en vigueur"),
            _ => "(dates inconnues)".to_string(),
        };
        lines.push(format!("{}. {range}{}{} — `{vid}`", i + 1, if etat.is_empty() { String::new() } else { format!(" — *{etat}*") }, if num.is_empty() { String::new() } else { format!(" — art. {num}") }));
    }
    lines.join("\n")
}

/// Chronologie (ChronolegiResponse) : regroupements[].versions est une MAP
/// { date → VersionDTO { dateDebut, isEndVersion, articlesModificateurs } }.
pub fn fmt_chrono(data: &Value, label: &str) -> String {
    fmt_chrono_range(data, label, None, None)
}

#[allow(dead_code)]
pub fn fmt_chrono_range(data: &Value, label: &str, y0: Option<i64>, y1: Option<i64>) -> String {
    let regs = data["regroupements"].as_array().cloned().unwrap_or_default();
    if regs.is_empty() {
        return format!("Aucune version trouvée pour `{label}`.");
    }
    let mut lines = vec![format!("**Chronologie de `{label}`** — {} regroupement(s)", regs.len())];
    if let Some(dp) = data["datePublication"].as_str() {
        lines.push(format!("*Publication : {}*", fmt_date(&Value::String(dp.to_string()))));
    }
    lines.push(String::new());
    for rg in &regs {
        let title = s(rg, "title");
        if !title.is_empty() {
            lines.push(format!("### {title}"));
        }
        let mut entries: Vec<(i64, String, &Value)> = Vec::new();
        match &rg["versions"] {
            Value::Object(m) => {
                for (k, v) in m {
                    let ms = value_to_ms(&v["dateDebut"]).or_else(|| k.parse::<i64>().ok()).or_else(|| ymd_to_ms(k)).unwrap_or(0);
                    let d = if !v["dateDebut"].is_null() { fmt_date(&v["dateDebut"]) } else { fmt_date(&Value::String(k.clone())) };
                    entries.push((ms, d, v));
                }
            }
            Value::Array(a) => {
                for v in a {
                    let ms = value_to_ms(&v["dateDebut"]).unwrap_or(0);
                    entries.push((ms, fmt_date(&v["dateDebut"]), v));
                }
            }
            _ => {}
        }
        entries.sort_by_key(|e| e.0);
        entries.retain(|(_, d, _)| {
            let y = d.get(..4).and_then(|x| x.parse::<i64>().ok());
            match y { Some(y) => y0.map_or(true, |a| y >= a) && y1.map_or(true, |b| y <= b), None => true }
        });
        for (_, d, v) in entries.iter().take(400) {
            let mut l = format!("- {d}");
            if v["isEndVersion"].as_bool() == Some(true) {
                l.push_str(" *(fin de version)*");
            }
            if let Some(m) = v["articlesModificateurs"].as_object() {
                if !m.is_empty() {
                    let ids: Vec<&str> = m.keys().map(|k| k.as_str()).take(3).collect();
                    l.push_str(&format!(" — modifié par {}{}", ids.join(", "), if m.len() > 3 { format!(" (+{})", m.len() - 3) } else { String::new() }));
                }
            }
            lines.push(l);
        }
        if entries.len() > 400 {
            lines.push(format!("*… {} version(s) supplémentaire(s)*", entries.len() - 400));
        }
    }
    lines.join("\n")
}

/// Liens de concordance / articles de même numéro (SectionsRevisionArticleResponse)
pub fn fmt_revision_links(data: &Value, id: &str, title: &str) -> String {
    let mut lines = vec![format!("**{title} — `{id}`**\n")];
    let mut n = 0;
    for (key, label) in [("oldTexts", "Anciens textes / articles"), ("newTexts", "Nouveaux textes / articles")] {
        if let Some(arr) = data[key].as_array() {
            if arr.is_empty() {
                continue;
            }
            lines.push(format!("**{label}** ({})", arr.len()));
            for a in arr.iter().take(60) {
                n += 1;
                let name = first_str(a, &["name", "titre"]);
                let (d, f) = (fmt_date(&a["dateDebut"]), fmt_date(&a["dateFin"]));
                let range = if d.is_empty() { String::new() } else { format!(" ({d} → {})", if f.is_empty() || is_open_end(&f) { "en vigueur".to_string() } else { f }) };
                lines.push(format!("- {name}{range} — `{}`{}", first_str(a, &["id", "cid"]), if s(a, "nature").is_empty() { String::new() } else { format!(" [{}]", s(a, "nature")) }));
            }
            lines.push(String::new());
        }
    }
    if n == 0 {
        format!("Aucun résultat pour `{id}`.")
    } else {
        lines.join("\n")
    }
}

/// Liste clé/valeur générique d'un objet JSON (réponses simples).
pub fn kv_lines(data: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            if v.is_null() || k == "executionTime" {
                continue;
            }
            let val = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            if !val.is_empty() && val != "[]" && val != "{}" {
                let (val, _) = truncate_chars(&val, 600);
                out.push(format!("- **{k}** : {val}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strip_html_preserve_les_accents() {
        let out = strip_html("<p>Le délai est prolongé à l'échéance&nbsp;: 3&nbsp;€ &laquo;&nbsp;test&nbsp;&raquo;.</p>");
        assert!(out.contains("délai") && out.contains("prolongé") && out.contains("échéance"), "{out:?}");
        assert!(out.contains("« test »"), "{out:?}");
        assert!(!out.contains('Ã'));
    }
    #[test]
    fn strip_html_listes_et_chevron_isole() {
        let out = strip_html("<ul><li>un</li><li>deux</li></ul> a < b");
        assert!(out.contains("- un") && out.contains("- deux") && out.contains("a < b"), "{out:?}");
    }
    #[test]
    fn troncature_sans_panic_sur_utf8() {
        let s = format!("{}é{}", "a".repeat(2999), "b".repeat(50));
        let (t, cut) = truncate_chars(&s, 3000);
        assert!(cut && t.chars().count() == 3000);
        let a = json!({"num":"1","texte": s});
        let _ = fmt_article(&a);
    }
    #[test]
    fn dates() {
        assert_eq!(fmt_date(&json!(1700000000000i64)), "2023-11-14");
        assert_eq!(fmt_date(&json!(-86400000i64)), "1969-12-31");
        assert_eq!(fmt_date(&json!("2024-01-31T00:00:00.000+0000")), "2024-01-31");
        assert_eq!(fmt_date(&json!(32472144000000i64)), "2999-01-01");
        assert!(is_open_end("2999-01-01"));
        assert_eq!(ymd_to_ms("2024-01-15"), Some(1705276800000));
        assert_eq!(ymd_to_ms("2024-13-01"), None);
    }
    #[test]
    fn chrono_map_de_versions() {
        let d = json!({"regroupements":[{"title":"Art. 1","versions":{
            "2020-01-01":{"dateDebut":1577836800000i64,"isEndVersion":false,"articlesModificateurs":{"JORFTEXT1":{}}},
            "2019-01-01":{"dateDebut":1546300800000i64,"isEndVersion":true}}}]});
        let out = fmt_chrono(&d, "X");
        let (a, b) = (out.find("2019-01-01").unwrap(), out.find("2020-01-01").unwrap());
        assert!(a < b && out.contains("JORFTEXT1") && out.contains("fin de version"), "{out}");
    }
    #[test]
    fn versions_article_triees() {
        let v = vec![json!({"id":"B","dateDebut":1577836800000i64,"dateFin":32472144000000i64,"etat":"VIGUEUR"}),
                     json!({"id":"A","dateDebut":1546300800000i64,"dateFin":1577836800000i64,"etat":"MODIFIE"})];
        let out = fmt_article_versions(&v, "X");
        assert!(out.find("`A`").unwrap() < out.find("`B`").unwrap() && out.contains("en vigueur"));
    }
}
