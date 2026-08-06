// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
// Portage Rust de system_tools.py (projet Prométhée)
// Licence : AGPL-3.0
// ============================================================================

use crate::mcp::{ToolCallResult, ToolDef};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local};
use serde_json::{json, Value};
use sha2::Sha256;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

// ============================================================================
// Sécurité
// ============================================================================

const FORBIDDEN_NAMES: &[&str] = &[
    ".env", ".ssh", "id_rsa", "id_ed25519", "passwd", "shadow",
    ".aws", "credentials", ".gnupg", "private", "secret",
];

const PROTECTED_PATHS: &[&str] = &[
    "/etc", "/sys", "/proc", "/dev", "/boot", "/root",
    "/bin", "/sbin", "/usr/bin", "/usr/sbin",
];

const MAX_WRITE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

fn is_safe_path(path: &Path, operation: &str) -> Result<()> {
    let resolved = path.canonicalize()
        .or_else(|_| {
            // Pour les chemins qui n'existent pas encore (création), résoudre le parent
            if let Some(parent) = path.parent() {
                let parent_res = parent.canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf());
                Ok::<PathBuf, std::io::Error>(parent_res.join(path.file_name().unwrap_or_default()))
            } else {
                Ok(path.to_path_buf())
            }
        })?;

    let resolved_str = resolved.to_string_lossy().to_lowercase();

    // Vérifier noms interdits
    for part in resolved.components() {
        let part_str = part.as_os_str().to_string_lossy().to_lowercase();
        if FORBIDDEN_NAMES.iter().any(|f| part_str.contains(f)) {
            bail!("Accès refusé : '{}' interdit", part_str);
        }
    }

    // Vérifier chemins protégés
    for protected in PROTECTED_PATHS {
        if resolved_str.starts_with(protected) {
            bail!("Chemin système protégé : {}", protected);
        }
    }

    // Pour écriture/suppression : vérifier qu'on est sous HOME
    if operation == "write" || operation == "delete" {
        let home = dirs_home();
        if !resolved_str.starts_with(&home.to_string_lossy().to_lowercase()) {
            bail!("Écriture autorisée uniquement sous {}", home.display());
        }
    }

    Ok(())
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home"))
}

fn resolve_path(s: &str) -> PathBuf {
    let p = PathBuf::from(s);
    if p.starts_with("~") {
        let home = dirs_home();
        home.join(p.strip_prefix("~").unwrap_or(&p))
    } else {
        p
    }
}

// ============================================================================
// Formatage
// ============================================================================

fn format_size(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    for unit in &units[..units.len() - 1] {
        if size < 1024.0 {
            return format!("{:.1} {}", size, unit);
        }
        size /= 1024.0;
    }
    format!("{:.1} TB", size)
}

fn format_time(t: SystemTime) -> String {
    let dt: DateTime<Local> = t.into();
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

// ============================================================================
// Registre des outils
// ============================================================================

pub fn all_tools() -> Vec<ToolDef> {
    vec![
        // Lecture / Écriture
        tool_read_file(),
        tool_write_file(),
        tool_append_to_file(),
        tool_tail_file(),
        tool_head_file(),
        tool_find_and_replace(),
        tool_replace_regex(),
        tool_touch_file(),
        // Navigation
        tool_list_files(),
        tool_tree_view(),
        tool_search_files(),
        tool_grep_files(),
        // Gestion
        tool_copy_file(),
        tool_move_file(),
        tool_delete_file(),
        tool_create_directory(),
        tool_get_file_info(),
        tool_count_lines(),
        tool_disk_usage(),
        // Fichiers binaires
        tool_read_binary_file(),
        tool_write_binary_file(),
        // Permissions et liens
        tool_set_permissions(),
        tool_create_symlink(),
        tool_read_symlink(),
        // Archives
        tool_compress_files(),
        tool_extract_archive(),
        // Comparaison
        tool_diff_files(),
        tool_compare_directories(),
        // Batch
        tool_batch_rename(),
        tool_batch_delete(),
        // Système
        tool_which_command(),
        tool_system_info(),
        tool_checksum(),
    ]
}

// ============================================================================
// Dispatch
// ============================================================================

pub async fn dispatch(name: &str, args: &Value) -> ToolCallResult {
    let args = args.clone();
    let name = name.to_string();
    let result = tokio::task::spawn_blocking(move || {
        dispatch_sync(&name, &args)
    })
    .await
    .unwrap_or_else(|e| Err(anyhow::anyhow!("Panique interne : {e}")));

    match result {
        Ok(text) => ToolCallResult::text(text),
        Err(e)   => ToolCallResult::error(format!("❌ Erreur : {e}")),
    }
}

fn dispatch_sync(name: &str, args: &Value) -> Result<String> {
    match name {
        "read_file"         => do_read_file(args),
        "write_file"        => do_write_file(args),
        "append_to_file"    => do_append_to_file(args),
        "tail_file"         => do_tail_file(args),
        "head_file"         => do_head_file(args),
        "find_and_replace"  => do_find_and_replace(args),
        "replace_regex"     => do_replace_regex(args),
        "touch_file"        => do_touch_file(args),
        "list_files"        => do_list_files(args),
        "tree_view"         => do_tree_view(args),
        "search_files"      => do_search_files(args),
        "grep_files"        => do_grep_files(args),
        "copy_file"         => do_copy_file(args),
        "move_file"         => do_move_file(args),
        "delete_file"       => do_delete_file(args),
        "create_directory"  => do_create_directory(args),
        "get_file_info"     => do_get_file_info(args),
        "count_lines"       => do_count_lines(args),
        "disk_usage"        => do_disk_usage(args),
        "read_binary_file"  => do_read_binary_file(args),
        "write_binary_file" => do_write_binary_file(args),
        "set_permissions"   => do_set_permissions(args),
        "create_symlink"    => do_create_symlink(args),
        "read_symlink"      => do_read_symlink(args),
        "compress_files"    => do_compress_files(args),
        "extract_archive"   => do_extract_archive(args),
        "diff_files"        => do_diff_files(args),
        "compare_directories" => do_compare_directories(args),
        "batch_rename"      => do_batch_rename(args),
        "batch_delete"      => do_batch_delete(args),
        "which_command"     => do_which_command(args),
        "system_info"       => do_system_info(args),
        "checksum"          => do_checksum(args),
        other => bail!("Outil inconnu : {other}"),
    }
}

// ============================================================================
// Définitions d'outils (ToolDef)
// ============================================================================

fn tool_read_file() -> ToolDef { ToolDef {
    name: "read_file".into(),
    description: "Lit un fichier texte avec support de plages de lignes et détection d'encoding.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Chemin du fichier"},
            "max_chars": {"type": "integer", "default": 4000},
            "start_line": {"type": "integer", "description": "Ligne de départ (1-indexed)"},
            "end_line": {"type": "integer", "description": "Ligne de fin (-1 pour EOF)"},
            "encoding": {"type": "string", "default": "utf-8"}
        },
        "required": ["path"]
    }),
}}

fn tool_write_file() -> ToolDef { ToolDef {
    name: "write_file".into(),
    description: "Écrit du contenu dans un fichier (max 10MB).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "content": {"type": "string"},
            "mode": {"type": "string", "default": "w",
                     "description": "w (écraser), a (ajouter), x (créer uniquement)"},
            "encoding": {"type": "string", "default": "utf-8"}
        },
        "required": ["path", "content"]
    }),
}}

fn tool_append_to_file() -> ToolDef { ToolDef {
    name: "append_to_file".into(),
    description: "Ajoute du contenu à la fin d'un fichier (le crée si besoin).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "content": {"type": "string"},
            "newline": {"type": "boolean", "default": true,
                        "description": "Ajoute un saut de ligne avant le contenu si le fichier n'est pas vide"}
        },
        "required": ["path", "content"]
    }),
}}

fn tool_tail_file() -> ToolDef { ToolDef {
    name: "tail_file".into(),
    description: "Affiche les dernières lignes d'un fichier.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "lines": {"type": "integer", "default": 10}
        },
        "required": ["path"]
    }),
}}

fn tool_head_file() -> ToolDef { ToolDef {
    name: "head_file".into(),
    description: "Affiche les premières lignes d'un fichier.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "lines": {"type": "integer", "default": 10}
        },
        "required": ["path"]
    }),
}}

fn tool_find_and_replace() -> ToolDef { ToolDef {
    name: "find_and_replace".into(),
    description: "Recherche et remplace du texte dans des fichiers.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "find": {"type": "string"},
            "replace": {"type": "string"},
            "pattern": {"type": "string", "default": "*.txt"},
            "recursive": {"type": "boolean", "default": true},
            "preview": {"type": "boolean", "default": true}
        },
        "required": ["path", "find", "replace"]
    }),
}}

fn tool_replace_regex() -> ToolDef { ToolDef {
    name: "replace_regex".into(),
    description: "Recherche et remplace via expression régulière (syntaxe Rust regex). \
        Supporte les groupes de capture dans le remplacement ($1, $2, ...).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "pattern": {"type": "string", "description": "Expression régulière"},
            "replace": {"type": "string", "description": "Remplacement, peut contenir $1, $2..."},
            "file_pattern": {"type": "string", "default": "*.txt"},
            "recursive": {"type": "boolean", "default": true},
            "preview": {"type": "boolean", "default": true},
            "multiline": {"type": "boolean", "default": false,
                          "description": "Si true, '.' ne matche pas '\\n' est désactivé et ^/$ matchent chaque ligne"}
        },
        "required": ["path", "pattern", "replace"]
    }),
}}

fn tool_touch_file() -> ToolDef { ToolDef {
    name: "touch_file".into(),
    description: "Crée un fichier vide s'il n'existe pas, ou met à jour sa date de modification.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"}
        },
        "required": ["path"]
    }),
}}

fn tool_list_files() -> ToolDef { ToolDef {
    name: "list_files".into(),
    description: "Liste les fichiers d'un dossier.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "default": "."},
            "pattern": {"type": "string", "default": "*"},
            "recursive": {"type": "boolean", "default": false},
            "sort_by": {"type": "string", "default": "name",
                        "description": "name | size | date"},
            "show_hidden": {"type": "boolean", "default": false}
        },
        "required": []
    }),
}}

fn tool_tree_view() -> ToolDef { ToolDef {
    name: "tree_view".into(),
    description: "Affiche l'arborescence d'un dossier.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "default": "."},
            "max_depth": {"type": "integer", "default": 3},
            "show_hidden": {"type": "boolean", "default": false}
        },
        "required": []
    }),
}}

fn tool_search_files() -> ToolDef { ToolDef {
    name: "search_files".into(),
    description: "Recherche avancée de fichiers par nom et/ou contenu.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "default": "."},
            "name_pattern": {"type": "string"},
            "content": {"type": "string"},
            "recursive": {"type": "boolean", "default": true},
            "max_results": {"type": "integer", "default": 50}
        },
        "required": []
    }),
}}

fn tool_grep_files() -> ToolDef { ToolDef {
    name: "grep_files".into(),
    description: "Recherche du texte ou une expression régulière dans des fichiers et retourne \
        les lignes correspondantes avec numéro de ligne et contexte (façon grep -n -C).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "default": "."},
            "pattern": {"type": "string", "description": "Texte ou regex à rechercher"},
            "regex": {"type": "boolean", "default": false,
                      "description": "Si true, 'pattern' est traité comme une expression régulière"},
            "file_pattern": {"type": "string", "default": "*"},
            "ignore_case": {"type": "boolean", "default": false},
            "recursive": {"type": "boolean", "default": true},
            "context": {"type": "integer", "default": 0,
                        "description": "Nombre de lignes de contexte avant/après"},
            "max_results": {"type": "integer", "default": 100}
        },
        "required": ["path", "pattern"]
    }),
}}

fn tool_copy_file() -> ToolDef { ToolDef {
    name: "copy_file".into(),
    description: "Copie un fichier ou dossier.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "source": {"type": "string"},
            "destination": {"type": "string"},
            "overwrite": {"type": "boolean", "default": false}
        },
        "required": ["source", "destination"]
    }),
}}

fn tool_move_file() -> ToolDef { ToolDef {
    name: "move_file".into(),
    description: "Déplace ou renomme un fichier.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "source": {"type": "string"},
            "destination": {"type": "string"}
        },
        "required": ["source", "destination"]
    }),
}}

fn tool_delete_file() -> ToolDef { ToolDef {
    name: "delete_file".into(),
    description: "Supprime un fichier ou dossier (confirmation requise).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "confirm": {"type": "boolean", "description": "DOIT être true pour confirmer"}
        },
        "required": ["path", "confirm"]
    }),
}}

fn tool_create_directory() -> ToolDef { ToolDef {
    name: "create_directory".into(),
    description: "Crée un répertoire.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "parents": {"type": "boolean", "default": true}
        },
        "required": ["path"]
    }),
}}

fn tool_get_file_info() -> ToolDef { ToolDef {
    name: "get_file_info".into(),
    description: "Métadonnées complètes d'un fichier (taille, dates, permissions, hash MD5).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"}
        },
        "required": ["path"]
    }),
}}

fn tool_count_lines() -> ToolDef { ToolDef {
    name: "count_lines".into(),
    description: "Compte les lignes de fichiers (utile pour du code source).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "pattern": {"type": "string", "default": "*.py"},
            "recursive": {"type": "boolean", "default": true}
        },
        "required": ["path"]
    }),
}}

fn tool_disk_usage() -> ToolDef { ToolDef {
    name: "disk_usage".into(),
    description: "Calcule la taille totale d'un dossier (récursif) et l'espace disque \
        disponible/total du système de fichiers correspondant.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string", "default": "."},
            "top_n": {"type": "integer", "default": 10,
                      "description": "Nombre de plus gros sous-éléments à lister"}
        },
        "required": []
    }),
}}

fn tool_read_binary_file() -> ToolDef { ToolDef {
    name: "read_binary_file".into(),
    description: "Lit un fichier binaire (image, exécutable, etc.) et retourne son contenu \
        encodé en base64. Limité à 10MB.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"}
        },
        "required": ["path"]
    }),
}}

fn tool_write_binary_file() -> ToolDef { ToolDef {
    name: "write_binary_file".into(),
    description: "Écrit un fichier binaire à partir d'un contenu encodé en base64 (max 10MB).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "content_base64": {"type": "string", "description": "Contenu encodé en base64"}
        },
        "required": ["path", "content_base64"]
    }),
}}

fn tool_set_permissions() -> ToolDef { ToolDef {
    name: "set_permissions".into(),
    description: "Modifie les permissions Unix d'un fichier ou dossier (chmod, ex: '755', '644').".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "mode": {"type": "string", "description": "Permissions en octal, ex: '755' ou '644'"},
            "recursive": {"type": "boolean", "default": false}
        },
        "required": ["path", "mode"]
    }),
}}

fn tool_create_symlink() -> ToolDef { ToolDef {
    name: "create_symlink".into(),
    description: "Crée un lien symbolique vers un fichier ou dossier.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "target": {"type": "string", "description": "Chemin pointé par le lien"},
            "link_path": {"type": "string", "description": "Chemin du lien à créer"}
        },
        "required": ["target", "link_path"]
    }),
}}

fn tool_read_symlink() -> ToolDef { ToolDef {
    name: "read_symlink".into(),
    description: "Lit la cible d'un lien symbolique.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"}
        },
        "required": ["path"]
    }),
}}

fn tool_compress_files() -> ToolDef { ToolDef {
    name: "compress_files".into(),
    description: "Crée une archive ZIP ou TAR(.gz).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "files": {"type": "array", "items": {"type": "string"}},
            "output": {"type": "string"},
            "format": {"type": "string", "default": "zip",
                       "description": "zip | tar | tar.gz"}
        },
        "required": ["files", "output"]
    }),
}}

fn tool_extract_archive() -> ToolDef { ToolDef {
    name: "extract_archive".into(),
    description: "Extrait une archive ou liste son contenu sans l'extraire. \
        Formats supportés : .zip, .tar, .tar.gz, .tgz, .tar.bz2, .tar.xz. \
        Protégé contre le zip-slip (path traversal). \
        Avec liste_seulement=true, retourne le contenu sans rien écrire sur disque.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "archive": {"type": "string", "description": "Chemin de l'archive"},
            "destination": {"type": "string",
                            "description": "Dossier de destination (défaut : même dossier)"},
            "liste_seulement": {"type": "boolean", "default": false},
            "sous_dossier": {"type": "boolean", "default": true,
                             "description": "Extraire dans un sous-dossier éponyme"}
        },
        "required": ["archive"]
    }),
}}

fn tool_diff_files() -> ToolDef { ToolDef {
    name: "diff_files".into(),
    description: "Compare deux fichiers texte (ou deux chaînes) et retourne leurs différences. \
        Produit un diff unifié ou des statistiques. Idéal pour comparer des versions de code, \
        vérifier des modifications de config, valider un find_and_replace.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "source_a": {"type": "string",
                         "description": "Chemin du premier fichier, ou texte brut si texte_brut=true"},
            "source_b": {"type": "string",
                         "description": "Chemin du second fichier, ou texte brut si texte_brut=true"},
            "texte_brut": {"type": "boolean", "default": false},
            "contexte": {"type": "integer", "default": 3},
            "mode": {"type": "string", "default": "unifie",
                     "enum": ["unifie", "stats"],
                     "description": "unifie (diff patch) | stats (statistiques seules)"},
            "ignorer_casse": {"type": "boolean", "default": false},
            "ignorer_espaces": {"type": "boolean", "default": false}
        },
        "required": ["source_a", "source_b"]
    }),
}}

fn tool_compare_directories() -> ToolDef { ToolDef {
    name: "compare_directories".into(),
    description: "Compare deux dossiers récursivement : fichiers présents uniquement dans A, \
        uniquement dans B, communs avec contenu différent, et identiques.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "dir_a": {"type": "string"},
            "dir_b": {"type": "string"},
            "compare_content": {"type": "boolean", "default": true,
                                 "description": "Si true, compare le contenu (hash) des fichiers communs"}
        },
        "required": ["dir_a", "dir_b"]
    }),
}}

fn tool_batch_rename() -> ToolDef { ToolDef {
    name: "batch_rename".into(),
    description: "Renomme plusieurs fichiers en remplaçant un motif dans leur nom.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "find": {"type": "string"},
            "replace": {"type": "string"},
            "pattern": {"type": "string", "default": "*"},
            "preview": {"type": "boolean", "default": true}
        },
        "required": ["path", "find", "replace"]
    }),
}}

fn tool_batch_delete() -> ToolDef { ToolDef {
    name: "batch_delete".into(),
    description: "Supprime plusieurs fichiers (confirmation requise).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "files": {"type": "array", "items": {"type": "string"}},
            "confirm": {"type": "boolean"}
        },
        "required": ["files", "confirm"]
    }),
}}

fn tool_which_command() -> ToolDef { ToolDef {
    name: "which_command".into(),
    description: "Recherche un exécutable dans le PATH système et retourne son chemin complet.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "command": {"type": "string", "description": "Nom de la commande, ex: 'python3'"}
        },
        "required": ["command"]
    }),
}}

fn tool_system_info() -> ToolDef { ToolDef {
    name: "system_info".into(),
    description: "Retourne des informations système : OS, architecture, répertoire courant, \
        répertoire HOME, nombre de coeurs CPU, et variables d'environnement (filtrées).".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "env_filter": {"type": "string",
                           "description": "Sous-chaîne pour filtrer les variables d'environnement retournées \
                               (les valeurs des variables sensibles sont masquées)"}
        },
        "required": []
    }),
}}

fn tool_checksum() -> ToolDef { ToolDef {
    name: "checksum".into(),
    description: "Calcule l'empreinte (hash) d'un fichier : md5, sha1 ou sha256.".into(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "algorithm": {"type": "string", "default": "sha256",
                          "enum": ["md5", "sha1", "sha256"]}
        },
        "required": ["path"]
    }),
}}

// ============================================================================
// Implémentations
// ============================================================================

// ---------------------------------------------------------------------------
// read_file
// ---------------------------------------------------------------------------
fn do_read_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let max_chars = args["max_chars"].as_u64().unwrap_or(4000) as usize;
    let start_line = args["start_line"].as_u64().map(|n| n as usize);
    let end_line   = args["end_line"].as_i64();

    let p = resolve_path(path_str);
    is_safe_path(&p, "read")?;
    if !p.exists() { bail!("Fichier non trouvé : {path_str}"); }

    let content = fs::read_to_string(&p)
        .with_context(|| format!("Lecture de {path_str}"))?;

    let content = if start_line.is_some() || end_line.is_some() {
        let lines: Vec<&str> = content.split('\n').collect();
        let start = start_line.map(|n| n.saturating_sub(1)).unwrap_or(0);
        let end = match end_line {
            Some(-1) | None => lines.len(),
            Some(n) => (n as usize).min(lines.len()),
        };
        lines[start..end].join("\n")
    } else {
        content
    };

    let original_len = content.len();
    let truncated = content.len() > max_chars;
    let content = if truncated { content[..max_chars].to_string() } else { content };

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "file": p.to_string_lossy(),
        "size": original_len,
        "truncated": truncated,
        "content": content
    }))?)
}

// ---------------------------------------------------------------------------
// write_file
// ---------------------------------------------------------------------------
fn do_write_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let content  = args["content"].as_str().context("'content' requis")?;
    let mode     = args["mode"].as_str().unwrap_or("w");

    let size = content.len();
    if size > MAX_WRITE_SIZE {
        bail!("Trop gros : {} > 10MB", format_size(size as u64));
    }

    let p = resolve_path(path_str);
    is_safe_path(&p, "write")?;

    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    match mode {
        "x" => {
            if p.exists() { bail!("Fichier existe déjà (mode x)"); }
            fs::write(&p, content)?;
        }
        "a" => {
            let mut f = fs::OpenOptions::new().create(true).append(true).open(&p)?;
            f.write_all(content.as_bytes())?;
        }
        _ => fs::write(&p, content)?,
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "file": p.to_string_lossy(),
        "size": format_size(size as u64)
    }))?)
}

// ---------------------------------------------------------------------------
// append_to_file
// ---------------------------------------------------------------------------
fn do_append_to_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let content  = args["content"].as_str().context("'content' requis")?;
    let newline  = args["newline"].as_bool().unwrap_or(true);

    if content.len() > MAX_WRITE_SIZE {
        bail!("Trop gros : {} > 10MB", format_size(content.len() as u64));
    }

    let p = resolve_path(path_str);
    is_safe_path(&p, "write")?;

    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    let existed_non_empty = p.exists() && p.metadata().map(|m| m.len() > 0).unwrap_or(false);

    let mut f = fs::OpenOptions::new().create(true).append(true).open(&p)?;
    if newline && existed_non_empty {
        f.write_all(b"\n")?;
    }
    f.write_all(content.as_bytes())?;

    let size = p.metadata().map(|m| m.len()).unwrap_or(0);
    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "file": p.to_string_lossy(),
        "new_size": format_size(size)
    }))?)
}

// ---------------------------------------------------------------------------
// tail_file
// ---------------------------------------------------------------------------
fn do_tail_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let n = args["lines"].as_u64().unwrap_or(10) as usize;

    let p = resolve_path(path_str);
    is_safe_path(&p, "read")?;

    let content = fs::read_to_string(&p)?;
    let all_lines: Vec<&str> = content.split('\n').collect();
    let total = all_lines.len();
    let tail = &all_lines[total.saturating_sub(n)..];

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "content": tail.join("\n"),
        "lines_shown": tail.len(),
        "total_lines": total
    }))?)
}

// ---------------------------------------------------------------------------
// head_file
// ---------------------------------------------------------------------------
fn do_head_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let n = args["lines"].as_u64().unwrap_or(10) as usize;

    let p = resolve_path(path_str);
    is_safe_path(&p, "read")?;

    let content = fs::read_to_string(&p)?;
    let all_lines: Vec<&str> = content.split('\n').collect();
    let total = all_lines.len();
    let head = &all_lines[..n.min(total)];

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "content": head.join("\n"),
        "lines_shown": head.len(),
        "total_lines": total
    }))?)
}

// ---------------------------------------------------------------------------
// find_and_replace
// ---------------------------------------------------------------------------
fn do_find_and_replace(args: &Value) -> Result<String> {
    let path_str  = args["path"].as_str().context("'path' requis")?;
    let find      = args["find"].as_str().context("'find' requis")?;
    let replace   = args["replace"].as_str().context("'replace' requis")?;
    let pattern   = args["pattern"].as_str().unwrap_or("*.txt");
    let recursive = args["recursive"].as_bool().unwrap_or(true);
    let preview   = args["preview"].as_bool().unwrap_or(true);

    let base = resolve_path(path_str);
    if !base.is_dir() { bail!("Pas un dossier"); }

    let ext = pattern.trim_start_matches('*');
    let mut results = Vec::new();

    let walker: Box<dyn Iterator<Item = walkdir::DirEntry>> = if recursive {
        Box::new(WalkDir::new(&base).into_iter().filter_map(|e| e.ok()))
    } else {
        Box::new(WalkDir::new(&base).max_depth(1).into_iter().filter_map(|e| e.ok()))
    };

    for entry in walker {
        let path = entry.path();
        if !path.is_file() { continue; }
        if !ext.is_empty() && !path.to_string_lossy().ends_with(ext) { continue; }
        if is_safe_path(path, "read").is_err() { continue; }

        let Ok(content) = fs::read_to_string(path) else { continue };
        if content.contains(find) {
            let occurrences = content.matches(find).count();
            if !preview {
                let _ = fs::write(path, content.replace(find, replace));
            }
            results.push(json!({"file": path.to_string_lossy(), "occurrences": occurrences}));
            if results.len() >= 20 { break; }
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "mode": if preview { "preview" } else { "applied" },
        "files_found": results.len(),
        "results": results
    }))?)
}

// ---------------------------------------------------------------------------
// replace_regex
// ---------------------------------------------------------------------------
fn do_replace_regex(args: &Value) -> Result<String> {
    let path_str     = args["path"].as_str().context("'path' requis")?;
    let pattern_str  = args["pattern"].as_str().context("'pattern' requis")?;
    let replace      = args["replace"].as_str().context("'replace' requis")?;
    let file_pattern = args["file_pattern"].as_str().unwrap_or("*.txt");
    let recursive    = args["recursive"].as_bool().unwrap_or(true);
    let preview      = args["preview"].as_bool().unwrap_or(true);
    let multiline    = args["multiline"].as_bool().unwrap_or(false);

    let pattern_src = if multiline {
        format!("(?m){pattern_str}")
    } else {
        pattern_str.to_string()
    };
    let re = regex::Regex::new(&pattern_src)
        .with_context(|| format!("Expression régulière invalide : {pattern_str}"))?;

    let base = resolve_path(path_str);
    if !base.is_dir() { bail!("Pas un dossier"); }

    let ext = file_pattern.trim_start_matches('*');
    let mut results = Vec::new();

    let walker: Box<dyn Iterator<Item = walkdir::DirEntry>> = if recursive {
        Box::new(WalkDir::new(&base).into_iter().filter_map(|e| e.ok()))
    } else {
        Box::new(WalkDir::new(&base).max_depth(1).into_iter().filter_map(|e| e.ok()))
    };

    for entry in walker {
        let path = entry.path();
        if !path.is_file() { continue; }
        if !ext.is_empty() && !path.to_string_lossy().ends_with(ext) { continue; }
        if is_safe_path(path, "read").is_err() { continue; }

        let Ok(content) = fs::read_to_string(path) else { continue };
        let occurrences = re.find_iter(&content).count();
        if occurrences > 0 {
            if !preview {
                let new_content = re.replace_all(&content, replace);
                if is_safe_path(path, "write").is_ok() {
                    let _ = fs::write(path, new_content.as_ref());
                }
            }
            results.push(json!({"file": path.to_string_lossy(), "occurrences": occurrences}));
            if results.len() >= 20 { break; }
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "mode": if preview { "preview" } else { "applied" },
        "pattern": pattern_str,
        "files_found": results.len(),
        "results": results
    }))?)
}

// ---------------------------------------------------------------------------
// touch_file
// ---------------------------------------------------------------------------
fn do_touch_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let p = resolve_path(path_str);
    is_safe_path(&p, "write")?;

    let created = !p.exists();

    if created {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(&p)?;
    } else {
        // Mettre à jour la date de modification en touchant le fichier
        let now = std::time::SystemTime::now();
        let f = fs::OpenOptions::new().write(true).open(&p)?;
        f.set_modified(now)?;
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "path": p.to_string_lossy(),
        "created": created
    }))?)
}

// ---------------------------------------------------------------------------
// list_files
// ---------------------------------------------------------------------------
fn do_list_files(args: &Value) -> Result<String> {
    let path_str    = args["path"].as_str().unwrap_or(".");
    let pattern     = args["pattern"].as_str().unwrap_or("*");
    let recursive   = args["recursive"].as_bool().unwrap_or(false);
    let sort_by     = args["sort_by"].as_str().unwrap_or("name");
    let show_hidden = args["show_hidden"].as_bool().unwrap_or(false);

    let base = resolve_path(path_str);
    if !base.is_dir() { bail!("Pas un dossier"); }

    let ext = pattern.trim_start_matches('*');
    let max_depth = if recursive { usize::MAX } else { 1 };

    let mut entries: Vec<Value> = WalkDir::new(&base)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != base.as_path())
        .filter(|e| {
            if !show_hidden {
                !e.file_name().to_string_lossy().starts_with('.')
            } else { true }
        })
        .filter(|e| {
            if !ext.is_empty() && e.path().is_file() {
                e.path().to_string_lossy().ends_with(ext)
            } else { true }
        })
        .take(50)
        .map(|e| {
            let meta = e.metadata().ok();
            let size_kb = meta.as_ref()
                .filter(|m| m.is_file())
                .map(|m| (m.len() as f64 / 1024.0 * 10.0).round() / 10.0);
            let modified = meta.as_ref()
                .and_then(|m| m.modified().ok())
                .map(format_time)
                .unwrap_or_default();
            json!({
                "name": e.file_name().to_string_lossy(),
                "path": e.path().to_string_lossy(),
                "type": if e.path().is_dir() { "dir" } else { "file" },
                "size_kb": size_kb,
                "modified": modified
            })
        })
        .collect();

    match sort_by {
        "size" => entries.sort_by(|a, b| {
            b["size_kb"].as_f64().unwrap_or(0.0)
                .partial_cmp(&a["size_kb"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "date" => entries.sort_by(|a, b| {
            b["modified"].as_str().unwrap_or("").cmp(a["modified"].as_str().unwrap_or(""))
        }),
        _ => entries.sort_by(|a, b| {
            a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or(""))
        }),
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "path": base.to_string_lossy(),
        "count": entries.len(),
        "files": entries
    }))?)
}

// ---------------------------------------------------------------------------
// tree_view
// ---------------------------------------------------------------------------
fn do_tree_view(args: &Value) -> Result<String> {
    let path_str    = args["path"].as_str().unwrap_or(".");
    let max_depth   = args["max_depth"].as_u64().unwrap_or(3) as usize;
    let show_hidden = args["show_hidden"].as_bool().unwrap_or(false);

    let base = resolve_path(path_str);
    if !base.is_dir() { bail!("Pas un dossier"); }

    let mut lines = vec![base.to_string_lossy().to_string()];
    build_tree(&base, "", 0, max_depth, show_hidden, &mut lines);

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "tree": lines[..lines.len().min(100)].join("\n")
    }))?)
}

fn build_tree(dir: &Path, prefix: &str, depth: usize, max_depth: usize,
              show_hidden: bool, lines: &mut Vec<String>) {
    if depth >= max_depth { return; }
    let Ok(entries) = fs::read_dir(dir) else { return };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| show_hidden || !p.file_name()
            .map(|n| n.to_string_lossy().starts_with('.'))
            .unwrap_or(false))
        .collect();
    children.sort_by(|a, b| {
        let a_dir = a.is_dir();
        let b_dir = b.is_dir();
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });
    let count = children.len();
    for (i, child) in children.into_iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = child.file_name().unwrap_or_default().to_string_lossy();
        lines.push(format!("{prefix}{connector}{name}"));
        if child.is_dir() {
            let extension = if is_last { "    " } else { "│   " };
            build_tree(&child, &format!("{prefix}{extension}"), depth + 1, max_depth, show_hidden, lines);
        }
        if lines.len() >= 100 { return; }
    }
}

// ---------------------------------------------------------------------------
// search_files
// ---------------------------------------------------------------------------
fn do_search_files(args: &Value) -> Result<String> {
    let path_str    = args["path"].as_str().unwrap_or(".");
    let name_pat    = args["name_pattern"].as_str();
    let content_pat = args["content"].as_str();
    let recursive   = args["recursive"].as_bool().unwrap_or(true);
    let max_results = args["max_results"].as_u64().unwrap_or(50) as usize;

    let base = resolve_path(path_str);
    if !base.is_dir() { bail!("Pas un dossier"); }

    let ext = name_pat.unwrap_or("*").trim_start_matches('*');
    let max_depth = if recursive { usize::MAX } else { 1 };

    let mut results = Vec::new();
    for entry in WalkDir::new(&base).max_depth(max_depth).into_iter().filter_map(|e| e.ok()) {
        if results.len() >= max_results { break; }
        let path = entry.path();
        if !path.is_file() { continue; }
        if !ext.is_empty() && !path.to_string_lossy().ends_with(ext) { continue; }

        let mut matched = true;
        if let Some(cp) = content_pat {
            matched = fs::read_to_string(path)
                .map(|c| c.contains(cp))
                .unwrap_or(false);
        }
        if matched {
            let size = path.metadata().map(|m| format_size(m.len())).unwrap_or_default();
            results.push(json!({
                "path": path.to_string_lossy(),
                "name": path.file_name().unwrap_or_default().to_string_lossy(),
                "size": size
            }));
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "found": results.len(),
        "results": results
    }))?)
}

// ---------------------------------------------------------------------------
// grep_files
// ---------------------------------------------------------------------------
fn do_grep_files(args: &Value) -> Result<String> {
    let path_str    = args["path"].as_str().unwrap_or(".");
    let pattern_str = args["pattern"].as_str().context("'pattern' requis")?;
    let use_regex   = args["regex"].as_bool().unwrap_or(false);
    let file_pattern = args["file_pattern"].as_str().unwrap_or("*");
    let ignore_case = args["ignore_case"].as_bool().unwrap_or(false);
    let recursive   = args["recursive"].as_bool().unwrap_or(true);
    let context     = args["context"].as_u64().unwrap_or(0) as usize;
    let max_results = args["max_results"].as_u64().unwrap_or(100) as usize;

    let base = resolve_path(path_str);
    if !base.exists() { bail!("Chemin introuvable"); }

    let regex_src = if use_regex {
        pattern_str.to_string()
    } else {
        regex::escape(pattern_str)
    };
    let pattern_final = if ignore_case { format!("(?i){regex_src}") } else { regex_src };
    let re = regex::Regex::new(&pattern_final)
        .with_context(|| format!("Expression régulière invalide : {pattern_str}"))?;

    let ext = file_pattern.trim_start_matches('*');
    let max_depth = if recursive { usize::MAX } else { 1 };

    let sources: Box<dyn Iterator<Item = PathBuf>> = if base.is_file() {
        Box::new(std::iter::once(base.clone()))
    } else {
        Box::new(
            WalkDir::new(&base).max_depth(max_depth).into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .filter(move |e| ext.is_empty() || e.path().to_string_lossy().ends_with(ext))
                .map(|e| e.into_path())
        )
    };

    let mut matches: Vec<Value> = Vec::new();

    'files: for file in sources {
        if is_safe_path(&file, "read").is_err() { continue; }
        let Ok(content) = fs::read_to_string(&file) else { continue };
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                let start = i.saturating_sub(context);
                let end = (i + context + 1).min(lines.len());
                let context_lines: Vec<Value> = (start..end).map(|j| json!({
                    "line": j + 1,
                    "text": lines[j],
                    "match": j == i
                })).collect();

                matches.push(json!({
                    "file": file.to_string_lossy(),
                    "line": i + 1,
                    "text": line,
                    "context": context_lines
                }));

                if matches.len() >= max_results { break 'files; }
            }
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "pattern": pattern_str,
        "found": matches.len(),
        "matches": matches
    }))?)
}

// ---------------------------------------------------------------------------
// copy_file
// ---------------------------------------------------------------------------
fn do_copy_file(args: &Value) -> Result<String> {
    let src_str  = args["source"].as_str().context("'source' requis")?;
    let dst_str  = args["destination"].as_str().context("'destination' requis")?;
    let overwrite = args["overwrite"].as_bool().unwrap_or(false);

    let src = resolve_path(src_str);
    let dst = resolve_path(dst_str);

    if !src.exists() { bail!("Source introuvable"); }
    is_safe_path(&dst, "write")?;
    if dst.exists() && !overwrite { bail!("Destination existe (utilisez overwrite=true)"); }

    if src.is_file() {
        if let Some(p) = dst.parent() { fs::create_dir_all(p)?; }
        fs::copy(&src, &dst)?;
    } else {
        copy_dir_all(&src, &dst)?;
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "copied": src.to_string_lossy(),
        "to": dst.to_string_lossy()
    }))?)
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// move_file
// ---------------------------------------------------------------------------
fn do_move_file(args: &Value) -> Result<String> {
    let src_str = args["source"].as_str().context("'source' requis")?;
    let dst_str = args["destination"].as_str().context("'destination' requis")?;

    let src = resolve_path(src_str);
    let dst = resolve_path(dst_str);

    if !src.exists() { bail!("Source introuvable"); }
    is_safe_path(&src, "delete")?;
    is_safe_path(&dst, "write")?;

    if let Some(p) = dst.parent() { fs::create_dir_all(p)?; }
    fs::rename(&src, &dst)?;

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "moved": src.to_string_lossy(),
        "to": dst.to_string_lossy()
    }))?)
}

// ---------------------------------------------------------------------------
// delete_file
// ---------------------------------------------------------------------------
fn do_delete_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let confirm  = args["confirm"].as_bool().unwrap_or(false);

    if !confirm {
        return Ok(serde_json::to_string_pretty(&json!({
            "status": "cancelled",
            "message": "Suppression annulée (confirm=false)"
        }))?);
    }

    let p = resolve_path(path_str);
    is_safe_path(&p, "delete")?;
    if !p.exists() { bail!("Fichier introuvable"); }

    if p.is_file() { fs::remove_file(&p)?; } else { fs::remove_dir_all(&p)?; }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "deleted": p.to_string_lossy()
    }))?)
}

// ---------------------------------------------------------------------------
// create_directory
// ---------------------------------------------------------------------------
fn do_create_directory(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let parents  = args["parents"].as_bool().unwrap_or(true);

    let p = resolve_path(path_str);
    is_safe_path(&p, "write")?;

    if p.exists() { bail!("Dossier existe déjà"); }

    if parents { fs::create_dir_all(&p)?; } else { fs::create_dir(&p)?; }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "created": p.to_string_lossy()
    }))?)
}

// ---------------------------------------------------------------------------
// get_file_info
// ---------------------------------------------------------------------------
fn do_get_file_info(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let p = resolve_path(path_str);
    if !p.exists() { bail!("Fichier introuvable"); }

    let meta = p.metadata()?;
    let mut info = json!({
        "status": "success",
        "path": p.to_string_lossy(),
        "name": p.file_name().unwrap_or_default().to_string_lossy(),
        "type": if meta.is_file() { "file" } else { "directory" },
        "size": format_size(meta.len()),
        "size_bytes": meta.len(),
        "created": meta.created().ok().map(format_time),
        "modified": meta.modified().ok().map(format_time),
        "permissions": format!("{:o}", {
            #[cfg(unix)]
            { use std::os::unix::fs::PermissionsExt; meta.permissions().mode() & 0o777 }
            #[cfg(not(unix))]
            { 0u32 }
        })
    });

    // Hash MD5 pour fichiers < 100 MB
    if meta.is_file() && meta.len() < 100 * 1024 * 1024 {
        let data = fs::read(&p)?;
        let digest = md5::compute(&data);
        info["md5"] = json!(format!("{:x}", digest));
    }

    Ok(serde_json::to_string_pretty(&info)?)
}

// ---------------------------------------------------------------------------
// count_lines
// ---------------------------------------------------------------------------
fn do_count_lines(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let pattern  = args["pattern"].as_str().unwrap_or("*.py");
    let recursive = args["recursive"].as_bool().unwrap_or(true);

    let p = resolve_path(path_str);
    if !p.exists() { bail!("Chemin introuvable"); }

    let ext = pattern.trim_start_matches('*');
    let max_depth = if recursive { usize::MAX } else { 1 };

    let mut total = 0usize;
    let mut file_count = 0usize;
    let mut details: Vec<Value> = Vec::new();

    let sources: Box<dyn Iterator<Item = PathBuf>> = if p.is_file() {
        Box::new(std::iter::once(p.clone()))
    } else {
        Box::new(
            WalkDir::new(&p).max_depth(max_depth).into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .filter(move |e| ext.is_empty() || e.path().to_string_lossy().ends_with(ext))
                .map(|e| e.into_path())
        )
    };

    for file in sources {
        let Ok(content) = fs::read_to_string(&file) else { continue };
        let lines = content.split('\n').count();
        total += lines;
        file_count += 1;
        details.push(json!({
            "file": file.file_name().unwrap_or_default().to_string_lossy(),
            "lines": lines
        }));
    }

    details.sort_by(|a, b| b["lines"].as_u64().unwrap_or(0).cmp(&a["lines"].as_u64().unwrap_or(0)));

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "total_lines": total,
        "files": file_count,
        "details": &details[..details.len().min(10)]
    }))?)
}

// ---------------------------------------------------------------------------
// compress_files
// ---------------------------------------------------------------------------
fn do_compress_files(args: &Value) -> Result<String> {
    let files_val = args["files"].as_array().context("'files' requis")?;
    let output    = args["output"].as_str().context("'output' requis")?;
    let format    = args["format"].as_str().unwrap_or("zip");

    let files: Vec<PathBuf> = files_val.iter()
        .filter_map(|v| v.as_str().map(|s| resolve_path(s)))
        .collect();

    let out = resolve_path(output);
    is_safe_path(&out, "write")?;
    if let Some(p) = out.parent() { fs::create_dir_all(p)?; }

    let count = files.len();
    match format {
        "zip" => {
            let file = fs::File::create(&out)?;
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for src in &files {
                if src.is_file() {
                    let name = src.file_name().unwrap_or_default().to_string_lossy();
                    zip.start_file(name.as_ref(), options)?;
                    let data = fs::read(src)?;
                    zip.write_all(&data)?;
                } else if src.is_dir() {
                    let base = src.parent().unwrap_or(src);
                    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
                        if entry.path().is_file() {
                            let rel = entry.path().strip_prefix(base)
                                .unwrap_or(entry.path());
                            zip.start_file(rel.to_string_lossy().as_ref(), options)?;
                            let data = fs::read(entry.path())?;
                            zip.write_all(&data)?;
                        }
                    }
                }
            }
            zip.finish()?;
        }
        "tar" | "tar.gz" => {
            let file = fs::File::create(&out)?;
            if format == "tar.gz" {
                let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
                let mut tar = tar::Builder::new(enc);
                for src in &files {
                    if src.exists() {
                        let name = src.file_name().unwrap_or_default();
                        if src.is_file() {
                            tar.append_path_with_name(src, name)?;
                        } else {
                            tar.append_dir_all(name, src)?;
                        }
                    }
                }
                tar.finish()?;
            } else {
                let mut tar = tar::Builder::new(file);
                for src in &files {
                    if src.exists() {
                        let name = src.file_name().unwrap_or_default();
                        if src.is_file() {
                            tar.append_path_with_name(src, name)?;
                        } else {
                            tar.append_dir_all(name, src)?;
                        }
                    }
                }
                tar.finish()?;
            }
        }
        other => bail!("Format non supporté : {other}"),
    }

    let size = out.metadata().map(|m| format_size(m.len())).unwrap_or_default();
    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "archive": out.to_string_lossy(),
        "size": size,
        "files_added": count
    }))?)
}

// ---------------------------------------------------------------------------
// extract_archive
// ---------------------------------------------------------------------------
fn do_extract_archive(args: &Value) -> Result<String> {
    let archive_str   = args["archive"].as_str().context("'archive' requis")?;
    let destination   = args["destination"].as_str();
    let liste_seulement = args["liste_seulement"].as_bool().unwrap_or(false);
    let sous_dossier  = args["sous_dossier"].as_bool().unwrap_or(true);

    let arc = resolve_path(archive_str);
    if !arc.exists() { bail!("Archive introuvable"); }
    is_safe_path(&arc, "read")?;

    let name_lower = arc.to_string_lossy().to_lowercase();
    let fmt = if name_lower.ends_with(".zip") { "zip" }
        else if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") { "tar.gz" }
        else if name_lower.ends_with(".tar.bz2") { "tar.bz2" }
        else if name_lower.ends_with(".tar.xz") { "tar.xz" }
        else if name_lower.ends_with(".tar") { "tar" }
        else { bail!("Format non reconnu : {:?}", arc.extension()) };

    // ── Mode listing ──────────────────────────────────────────────────────────
    if liste_seulement {
        let entries: Vec<Value> = match fmt {
            "zip" => {
                let f = fs::File::open(&arc)?;
                let mut z = zip::ZipArchive::new(f)?;
                (0..z.len()).map(|i| {
                    let e = z.by_index(i).unwrap();
                    json!({
                        "nom": e.name(),
                        "taille": format_size(e.size()),
                        "compresse": format_size(e.compressed_size()),
                        "dossier": e.is_dir()
                    })
                }).collect()
            }
            _ => {
                let entries = tar_list(&arc, fmt)?;
                entries.into_iter().map(|(name, size, is_dir)| json!({
                    "nom": name,
                    "taille": format_size(size),
                    "compresse": null,
                    "dossier": is_dir
                })).collect()
            }
        };
        let n = entries.len();
        return Ok(serde_json::to_string_pretty(&json!({
            "status": "success",
            "archive": arc.to_string_lossy(),
            "format": fmt.to_uppercase(),
            "nombre_entrees": n,
            "entrees": &entries[..n.min(100)],
            "tronque": n > 100
        }))?);
    }

    // ── Mode extraction ───────────────────────────────────────────────────────
    let dst_base = destination
        .map(|d| resolve_path(d))
        .unwrap_or_else(|| arc.parent().unwrap_or(Path::new(".")).to_path_buf());

    let dst = if sous_dossier {
        let fname = arc.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let stem = strip_archive_ext(&fname);
        dst_base.join(stem)
    } else {
        dst_base
    };

    is_safe_path(&dst, "write")?;
    fs::create_dir_all(&dst)?;

    let nb = match fmt {
        "zip" => {
            let f = fs::File::open(&arc)?;
            let mut z = zip::ZipArchive::new(f)?;
            let n = z.len();
            // Protection zip-slip
            for i in 0..n {
                let e = z.by_index(i)?;
                let member = dst.join(e.name());
                let member_res = member.canonicalize()
                    .unwrap_or_else(|_| member.clone());
                let dst_res = dst.canonicalize().unwrap_or_else(|_| dst.clone());
                if !member_res.starts_with(&dst_res) {
                    bail!("Zip-slip détecté : '{}' sort de la destination", e.name());
                }
            }
            z.extract(&dst)?;
            n
        }
        _ => tar_extract(&arc, &dst, fmt)?,
    };

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "archive": arc.to_string_lossy(),
        "extrait_dans": dst.to_string_lossy(),
        "fichiers_extraits": nb
    }))?)
}

fn strip_archive_ext(name: &str) -> &str {
    for ext in &[".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".tar", ".zip"] {
        if name.to_lowercase().ends_with(ext) {
            return &name[..name.len() - ext.len()];
        }
    }
    name
}

fn tar_list(arc: &Path, fmt: &str) -> Result<Vec<(String, u64, bool)>> {
    let f = fs::File::open(arc)?;
    let mut entries = Vec::new();
    match fmt {
        "tar.gz" => {
            let dec = flate2::read::GzDecoder::new(f);
            let mut a = tar::Archive::new(dec);
            for e in a.entries()? {
                let e = e?;
                entries.push((e.path()?.to_string_lossy().into_owned(), e.size(), e.header().entry_type().is_dir()));
            }
        }
        "tar.bz2" => {
            let dec = bzip2::read::BzDecoder::new(f);
            let mut a = tar::Archive::new(dec);
            for e in a.entries()? {
                let e = e?;
                entries.push((e.path()?.to_string_lossy().into_owned(), e.size(), e.header().entry_type().is_dir()));
            }
        }
        "tar.xz" => {
            let dec = xz2::read::XzDecoder::new(f);
            let mut a = tar::Archive::new(dec);
            for e in a.entries()? {
                let e = e?;
                entries.push((e.path()?.to_string_lossy().into_owned(), e.size(), e.header().entry_type().is_dir()));
            }
        }
        _ => {
            let mut a = tar::Archive::new(f);
            for e in a.entries()? {
                let e = e?;
                entries.push((e.path()?.to_string_lossy().into_owned(), e.size(), e.header().entry_type().is_dir()));
            }
        }
    }
    Ok(entries)
}

fn tar_extract(arc: &Path, dst: &Path, fmt: &str) -> Result<usize> {
    let f = fs::File::open(arc)?;
    // Protection zip-slip + extraction
    macro_rules! extract_tar {
        ($archive:expr) => {{
            let mut a = tar::Archive::new($archive);
            let mut n = 0;
            for e in a.entries()? {
                let mut e = e?;
                let entry_path = dst.join(e.path()?);
                let entry_res = entry_path.canonicalize()
                    .unwrap_or_else(|_| entry_path.clone());
                let dst_res = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
                if !entry_res.starts_with(&dst_res) {
                    bail!("Path traversal détecté : '{}' sort de la destination",
                          e.path()?.display());
                }
                e.unpack_in(dst)?;
                n += 1;
            }
            n
        }};
    }
    let n = match fmt {
        "tar.gz"  => { let d = flate2::read::GzDecoder::new(f); extract_tar!(d) }
        "tar.bz2" => { let d = bzip2::read::BzDecoder::new(f);  extract_tar!(d) }
        "tar.xz"  => { let d = xz2::read::XzDecoder::new(f);    extract_tar!(d) }
        _         => { extract_tar!(f) }
    };
    Ok(n)
}

// ---------------------------------------------------------------------------
// diff_files
// ---------------------------------------------------------------------------
fn do_diff_files(args: &Value) -> Result<String> {
    let source_a       = args["source_a"].as_str().context("'source_a' requis")?;
    let source_b       = args["source_b"].as_str().context("'source_b' requis")?;
    let texte_brut     = args["texte_brut"].as_bool().unwrap_or(false);
    let contexte       = args["contexte"].as_u64().unwrap_or(3) as usize;
    let mode           = args["mode"].as_str().unwrap_or("unifie");
    let ignorer_casse  = args["ignorer_casse"].as_bool().unwrap_or(false);
    let ignorer_espaces = args["ignorer_espaces"].as_bool().unwrap_or(false);

    let (content_a, content_b, label_a, label_b) = if texte_brut {
        (source_a.to_string(), source_b.to_string(),
         "<texte_a>".to_string(), "<texte_b>".to_string())
    } else {
        let pa = resolve_path(source_a);
        let pb = resolve_path(source_b);
        for p in [&pa, &pb] {
            is_safe_path(p, "read")?;
            if !p.exists() { bail!("Fichier introuvable : {}", p.display()); }
        }
        (fs::read_to_string(&pa)?, fs::read_to_string(&pb)?,
         pa.to_string_lossy().into_owned(), pb.to_string_lossy().into_owned())
    };

    let normalize = |s: &str| -> String {
        let mut result = s.to_string();
        if ignorer_espaces { result = result.lines().map(|l| l.trim().to_string()).collect::<Vec<_>>().join("\n"); }
        if ignorer_casse   { result = result.to_lowercase(); }
        result
    };

    let ca_norm = normalize(&content_a);
    let cb_norm = normalize(&content_b);

    let lines_a: Vec<&str> = ca_norm.lines().collect();
    let lines_b: Vec<&str> = cb_norm.lines().collect();

    // Statistiques via similar
    let diff = similar::TextDiff::from_lines(&ca_norm, &cb_norm);
    let mut added = 0usize;
    let mut removed = 0usize;
    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Insert => added += 1,
            similar::ChangeTag::Delete => removed += 1,
            _ => {}
        }
    }
    let identical = added == 0 && removed == 0;

    let stats = json!({
        "lignes_a": lines_a.len(),
        "lignes_b": lines_b.len(),
        "ajoutees": added,
        "supprimees": removed,
        "identique": identical
    });

    if mode == "stats" || identical {
        return Ok(serde_json::to_string_pretty(&json!({
            "status": "success",
            "label_a": label_a,
            "label_b": label_b,
            "stats": stats,
            "diff": if identical { Value::Null } else { json!("(non calculé en mode stats)") }
        }))?);
    }

    // Mode unifié
    let mut diff_text = String::new();
    for hunk in diff.unified_diff().context_radius(contexte).iter_hunks() {
        for change in hunk.iter_changes() {
            match change.tag() {
                similar::ChangeTag::Equal  => diff_text.push_str(&format!(" {}", change.value())),
                similar::ChangeTag::Insert => diff_text.push_str(&format!("+{}", change.value())),
                similar::ChangeTag::Delete => diff_text.push_str(&format!("-{}", change.value())),
            }
        }
    }

    const MAX_DIFF: usize = 20_000;
    let truncated = diff_text.len() > MAX_DIFF;
    if truncated { diff_text.truncate(MAX_DIFF); diff_text.push_str("\n... [diff tronqué]"); }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "label_a": label_a,
        "label_b": label_b,
        "stats": stats,
        "diff": diff_text,
        "tronque": truncated
    }))?)
}

// ---------------------------------------------------------------------------
// batch_rename
// ---------------------------------------------------------------------------
fn do_batch_rename(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let find     = args["find"].as_str().context("'find' requis")?;
    let replace  = args["replace"].as_str().context("'replace' requis")?;
    let pattern  = args["pattern"].as_str().unwrap_or("*");
    let preview  = args["preview"].as_bool().unwrap_or(true);

    let base = resolve_path(path_str);
    if !base.is_dir() { bail!("Pas un dossier"); }

    let ext = pattern.trim_start_matches('*');
    let mut renames: Vec<Value> = Vec::new();

    for entry in fs::read_dir(&base)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() { continue; }
        if !ext.is_empty() && !path.to_string_lossy().ends_with(ext) { continue; }

        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        if name.contains(find) {
            let new_name = name.replace(find, replace);
            let new_path = path.parent().unwrap_or(&base).join(&new_name);

            if !preview {
                match fs::rename(&path, &new_path) {
                    Ok(_)  => renames.push(json!({"from": name, "to": new_name, "done": true})),
                    Err(_) => renames.push(json!({"from": name, "to": new_name, "done": false})),
                }
            } else {
                renames.push(json!({"from": name, "to": new_name}));
            }
            if renames.len() >= 20 { break; }
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "mode": if preview { "preview" } else { "applied" },
        "renamed": renames.len(),
        "results": renames
    }))?)
}

// ---------------------------------------------------------------------------
// batch_delete
// ---------------------------------------------------------------------------
fn do_batch_delete(args: &Value) -> Result<String> {
    let files_val = args["files"].as_array().context("'files' requis")?;
    let confirm   = args["confirm"].as_bool().unwrap_or(false);

    if !confirm {
        return Ok(serde_json::to_string_pretty(&json!({
            "status": "cancelled",
            "message": "Suppression annulée (confirm=false)"
        }))?);
    }

    let mut deleted: Vec<String> = Vec::new();
    let mut errors: Vec<Value>   = Vec::new();

    for file_val in files_val.iter().take(20) {
        let Some(path_str) = file_val.as_str() else { continue };
        let p = resolve_path(path_str);

        if let Err(e) = is_safe_path(&p, "delete") {
            errors.push(json!({"file": path_str, "error": e.to_string()}));
            continue;
        }

        let res = if p.is_file() { fs::remove_file(&p) } else { fs::remove_dir_all(&p) };
        match res {
            Ok(_)  => deleted.push(p.to_string_lossy().into_owned()),
            Err(e) => errors.push(json!({"file": path_str, "error": e.to_string()})),
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "deleted": deleted.len(),
        "errors": errors.len(),
        "deleted_files": deleted,
        "error_files": errors
    }))?)
}

// ---------------------------------------------------------------------------
// disk_usage
// ---------------------------------------------------------------------------
fn do_disk_usage(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().unwrap_or(".");
    let top_n    = args["top_n"].as_u64().unwrap_or(10) as usize;

    let base = resolve_path(path_str);
    if !base.exists() { bail!("Chemin introuvable : {path_str}"); }
    is_safe_path(&base, "read")?;

    // Taille récursive totale
    let total_bytes: u64 = WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.metadata().ok().map(|m| m.len()))
        .sum();

    // Top-N sous-éléments directs par taille
    let mut children: Vec<(String, u64)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&base) {
        for entry in rd.filter_map(|e| e.ok()) {
            let p = entry.path();
            let size: u64 = if p.is_file() {
                p.metadata().map(|m| m.len()).unwrap_or(0)
            } else {
                WalkDir::new(&p)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .filter_map(|e| e.metadata().ok().map(|m| m.len()))
                    .sum()
            };
            children.push((p.to_string_lossy().into_owned(), size));
        }
    }
    children.sort_by(|a, b| b.1.cmp(&a.1));
    let top: Vec<Value> = children.iter().take(top_n)
        .map(|(name, size)| json!({"path": name, "size": format_size(*size), "bytes": size}))
        .collect();

    // Espace disque via statvfs (Unix) — utilisation de std::fs::metadata sur le FS
    #[cfg(unix)]
    let fs_info: Option<Value> = {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        let cpath = CString::new(base.to_string_lossy().as_bytes()).ok();
        cpath.and_then(|cp| {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            let ret = unsafe { libc::statvfs(cp.as_ptr(), stat.as_mut_ptr()) };
            if ret == 0 {
                let s = unsafe { stat.assume_init() };
                let block = s.f_frsize as u64;
                // f_blocks/f_bavail sont u64 sous Linux mais u32 sous macOS
                // (libc::statvfs) — cast explicite nécessaire pour compiler
                // sur les deux plateformes (sans effet sur Linux, où le cast
                // u64→u64 est un no-op).
                let total  = s.f_blocks as u64 * block;
                let free   = s.f_bavail as u64 * block;
                let used   = total - free;
                Some(json!({
                    "total":     format_size(total),
                    "used":      format_size(used),
                    "available": format_size(free),
                    "use_percent": if total > 0 { format!("{:.1}%", used as f64 / total as f64 * 100.0) } else { "N/A".to_string() }
                }))
            } else {
                None
            }
        })
    };
    #[cfg(not(unix))]
    let fs_info: Option<Value> = None;

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "path": base.to_string_lossy(),
        "total_size": format_size(total_bytes),
        "total_bytes": total_bytes,
        "top_entries": top,
        "filesystem": fs_info
    }))?)
}

// ---------------------------------------------------------------------------
// read_binary_file
// ---------------------------------------------------------------------------
fn do_read_binary_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let p = resolve_path(path_str);
    is_safe_path(&p, "read")?;
    if !p.exists() { bail!("Fichier non trouvé : {path_str}"); }

    let meta = p.metadata()?;
    if meta.len() > MAX_WRITE_SIZE as u64 {
        bail!("Fichier trop volumineux : {} > 10MB", format_size(meta.len()));
    }

    let data = fs::read(&p)?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "file": p.to_string_lossy(),
        "size": format_size(meta.len()),
        "size_bytes": meta.len(),
        "content_base64": encoded,
        "mime_hint": guess_mime(&p)
    }))?)
}

fn guess_mime(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
        "png"  => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif"  => "image/gif",
        "webp" => "image/webp",
        "svg"  => "image/svg+xml",
        "pdf"  => "application/pdf",
        "zip"  => "application/zip",
        "tar"  => "application/x-tar",
        "gz"   => "application/gzip",
        "mp3"  => "audio/mpeg",
        "mp4"  => "video/mp4",
        "json" => "application/json",
        "xml"  => "application/xml",
        _      => "application/octet-stream",
    }
}

// ---------------------------------------------------------------------------
// write_binary_file
// ---------------------------------------------------------------------------
fn do_write_binary_file(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let b64      = args["content_base64"].as_str().context("'content_base64' requis")?;

    let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .context("Décodage base64 invalide")?;

    if data.len() > MAX_WRITE_SIZE {
        bail!("Données trop volumineuses : {} > 10MB", format_size(data.len() as u64));
    }

    let p = resolve_path(path_str);
    is_safe_path(&p, "write")?;
    if let Some(parent) = p.parent() { fs::create_dir_all(parent)?; }
    fs::write(&p, &data)?;

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "file": p.to_string_lossy(),
        "size": format_size(data.len() as u64)
    }))?)
}

// ---------------------------------------------------------------------------
// set_permissions
// ---------------------------------------------------------------------------
fn do_set_permissions(args: &Value) -> Result<String> {
    let path_str  = args["path"].as_str().context("'path' requis")?;
    let mode_str  = args["mode"].as_str().context("'mode' requis")?;
    let recursive = args["recursive"].as_bool().unwrap_or(false);

    let mode = u32::from_str_radix(mode_str, 8)
        .with_context(|| format!("Mode octal invalide : '{mode_str}'"))?;

    let p = resolve_path(path_str);
    is_safe_path(&p, "write")?;
    if !p.exists() { bail!("Chemin introuvable : {path_str}"); }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let chmod_one = |path: &Path| -> Result<()> {
            let mut perms = path.metadata()?.permissions();
            perms.set_mode(mode);
            fs::set_permissions(path, perms)
                .with_context(|| format!("chmod {} sur {}", mode_str, path.display()))?;
            Ok(())
        };

        let mut changed = 0usize;
        if recursive && p.is_dir() {
            for entry in WalkDir::new(&p).into_iter().filter_map(|e| e.ok()) {
                if chmod_one(entry.path()).is_ok() { changed += 1; }
            }
        } else {
            chmod_one(&p)?;
            changed = 1;
        }

        Ok(serde_json::to_string_pretty(&json!({
            "status": "success",
            "path": p.to_string_lossy(),
            "mode": mode_str,
            "entries_changed": changed
        }))?)
    }

    #[cfg(not(unix))]
    bail!("set_permissions n'est supporté que sur Unix")
}

// ---------------------------------------------------------------------------
// create_symlink
// ---------------------------------------------------------------------------
fn do_create_symlink(args: &Value) -> Result<String> {
    let target_str    = args["target"].as_str().context("'target' requis")?;
    let link_path_str = args["link_path"].as_str().context("'link_path' requis")?;

    let link_path = resolve_path(link_path_str);
    is_safe_path(&link_path, "write")?;
    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        bail!("Le chemin du lien existe déjà : {link_path_str}");
    }

    if let Some(parent) = link_path.parent() { fs::create_dir_all(parent)?; }

    let target = resolve_path(target_str);

    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link_path)
        .with_context(|| format!("Création du lien symbolique {} → {}", link_path.display(), target.display()))?;

    #[cfg(windows)]
    {
        if target.is_dir() {
            std::os::windows::fs::symlink_dir(&target, &link_path)?;
        } else {
            std::os::windows::fs::symlink_file(&target, &link_path)?;
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "link": link_path.to_string_lossy(),
        "target": target.to_string_lossy()
    }))?)
}

// ---------------------------------------------------------------------------
// read_symlink
// ---------------------------------------------------------------------------
fn do_read_symlink(args: &Value) -> Result<String> {
    let path_str = args["path"].as_str().context("'path' requis")?;
    let p = resolve_path(path_str);
    is_safe_path(&p, "read")?;

    let target = fs::read_link(&p)
        .with_context(|| format!("Impossible de lire le lien symbolique : {path_str}"))?;

    let target_exists = target.exists();
    let resolved = target.canonicalize().ok();

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "link": p.to_string_lossy(),
        "target": target.to_string_lossy(),
        "resolved": resolved.map(|r| r.to_string_lossy().into_owned()),
        "target_exists": target_exists
    }))?)
}

// ---------------------------------------------------------------------------
// compare_directories
// ---------------------------------------------------------------------------
fn do_compare_directories(args: &Value) -> Result<String> {
    let dir_a_str       = args["dir_a"].as_str().context("'dir_a' requis")?;
    let dir_b_str       = args["dir_b"].as_str().context("'dir_b' requis")?;
    let compare_content = args["compare_content"].as_bool().unwrap_or(true);

    let dir_a = resolve_path(dir_a_str);
    let dir_b = resolve_path(dir_b_str);

    for d in [&dir_a, &dir_b] {
        is_safe_path(d, "read")?;
        if !d.is_dir() { bail!("Pas un dossier : {}", d.display()); }
    }

    // Construire l'index (chemin relatif → taille) pour chaque dossier
    let index = |base: &Path| -> Result<std::collections::HashMap<String, u64>> {
        let mut map = std::collections::HashMap::new();
        for entry in WalkDir::new(base).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let rel = entry.path().strip_prefix(base)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .into_owned();
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                map.insert(rel, size);
            }
        }
        Ok(map)
    };

    let map_a = index(&dir_a)?;
    let map_b = index(&dir_b)?;

    let mut only_a:    Vec<String> = Vec::new();
    let mut only_b:    Vec<String> = Vec::new();
    let mut identical: Vec<String> = Vec::new();
    let mut different: Vec<Value>  = Vec::new();

    for (rel, size_a) in &map_a {
        if let Some(size_b) = map_b.get(rel) {
            // Fichier commun — comparer le contenu si demandé
            let same = if compare_content {
                let pa = dir_a.join(rel);
                let pb = dir_b.join(rel);
                if size_a != size_b {
                    false
                } else {
                    let ha = md5_of(&pa);
                    let hb = md5_of(&pb);
                    ha == hb
                }
            } else {
                size_a == size_b
            };
            if same {
                identical.push(rel.clone());
            } else {
                different.push(json!({
                    "file": rel,
                    "size_a": format_size(*size_a),
                    "size_b": format_size(*size_b),
                    "size_changed": size_a != size_b
                }));
            }
        } else {
            only_a.push(rel.clone());
        }
    }

    for rel in map_b.keys() {
        if !map_a.contains_key(rel) {
            only_b.push(rel.clone());
        }
    }

    only_a.sort(); only_b.sort(); identical.sort();

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "dir_a": dir_a.to_string_lossy(),
        "dir_b": dir_b.to_string_lossy(),
        "summary": {
            "only_in_a":  only_a.len(),
            "only_in_b":  only_b.len(),
            "identical":  identical.len(),
            "different":  different.len()
        },
        "only_in_a":  &only_a[..only_a.len().min(50)],
        "only_in_b":  &only_b[..only_b.len().min(50)],
        "different":  &different[..different.len().min(50)],
        "identical":  &identical[..identical.len().min(50)],
        "truncated": only_a.len() > 50 || only_b.len() > 50 || different.len() > 50 || identical.len() > 50
    }))?)
}

fn md5_of(p: &Path) -> String {
    fs::read(p)
        .map(|data| format!("{:x}", md5::compute(&data)))
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// which_command
// ---------------------------------------------------------------------------
fn do_which_command(args: &Value) -> Result<String> {
    let command = args["command"].as_str().context("'command' requis")?;

    // Sécurité : refuser les noms contenant des séparateurs ou caractères spéciaux
    if command.contains('/') || command.contains('\\') || command.contains(' ') {
        bail!("Nom de commande invalide : '{command}'");
    }

    let path_var = std::env::var("PATH").unwrap_or_default();
    let mut found: Vec<String> = Vec::new();

    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join(command);
        if candidate.is_file() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = candidate.metadata() {
                    if meta.permissions().mode() & 0o111 != 0 {
                        found.push(candidate.to_string_lossy().into_owned());
                    }
                }
            }
            #[cfg(not(unix))]
            found.push(candidate.to_string_lossy().into_owned());
        }
    }

    if found.is_empty() {
        Ok(serde_json::to_string_pretty(&json!({
            "status": "not_found",
            "command": command,
            "message": format!("'{}' n'a pas été trouvé dans le PATH", command)
        }))?)
    } else {
        Ok(serde_json::to_string_pretty(&json!({
            "status": "found",
            "command": command,
            "path": found[0],
            "all_matches": found
        }))?)
    }
}

// ---------------------------------------------------------------------------
// system_info
// ---------------------------------------------------------------------------
fn do_system_info(args: &Value) -> Result<String> {
    let env_filter = args["env_filter"].as_str();

    // Variables sensibles à masquer
    const SENSITIVE: &[&str] = &[
        "password", "passwd", "secret", "token", "api_key", "apikey",
        "auth", "private", "credential", "aws_", "ssh_", "gpg",
    ];

    let env_vars: Vec<Value> = if let Some(filter) = env_filter {
        let filter_lower = filter.to_lowercase();
        std::env::vars()
            .filter(|(k, _)| k.to_lowercase().contains(&filter_lower))
            .map(|(k, v)| {
                let k_lower = k.to_lowercase();
                let masked = SENSITIVE.iter().any(|s| k_lower.contains(s));
                json!({
                    "key": k,
                    "value": if masked { "***MASQUÉ***".to_string() } else { v }
                })
            })
            .take(50)
            .collect()
    } else {
        Vec::new()
    };

    let home    = dirs_home();
    let cwd     = std::env::current_dir().unwrap_or_default();
    let path_var = std::env::var("PATH").unwrap_or_default();
    let path_dirs: Vec<&str> = path_var.split(':').collect();

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "family": std::env::consts::FAMILY,
        "home": home.to_string_lossy(),
        "cwd": cwd.to_string_lossy(),
        "cpu_count": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        "path_dirs": path_dirs,
        "env_vars": env_vars
    }))?)
}

// ---------------------------------------------------------------------------
// checksum
// ---------------------------------------------------------------------------
fn do_checksum(args: &Value) -> Result<String> {
    let path_str  = args["path"].as_str().context("'path' requis")?;
    let algorithm = args["algorithm"].as_str().unwrap_or("sha256");

    let p = resolve_path(path_str);
    is_safe_path(&p, "read")?;
    if !p.exists() { bail!("Fichier non trouvé : {path_str}"); }

    let meta = p.metadata()?;
    if meta.len() > 512 * 1024 * 1024 {
        bail!("Fichier trop volumineux pour le calcul de hash (> 512 MB)");
    }

    let data = fs::read(&p)?;
    let hash = match algorithm {
        "md5" => {
            format!("{:x}", md5::compute(&data))
        }
        "sha1" => {
            use sha1::Digest as Sha1Digest;
            let mut h = sha1::Sha1::new();
            Sha1Digest::update(&mut h, &data);
            format!("{:x}", Sha1Digest::finalize(h))
        }
        "sha256" | _ => {
            use sha2::Digest as Sha256Digest;
            let mut h = Sha256::new();
            Sha256Digest::update(&mut h, &data);
            format!("{:x}", Sha256Digest::finalize(h))
        }
    };

    Ok(serde_json::to_string_pretty(&json!({
        "status": "success",
        "file": p.to_string_lossy(),
        "size": format_size(meta.len()),
        "algorithm": algorithm,
        "hash": hash
    }))?)
}
