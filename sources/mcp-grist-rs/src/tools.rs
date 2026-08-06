// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================
use crate::client::GristClient;
use crate::mcp::{ToolCallResult, ToolDef};
use anyhow::Result;
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Registre
// ---------------------------------------------------------------------------

pub fn all_tools() -> Vec<ToolDef> {
    vec![
        // Organisations
        tool_list_my_docs(),
        tool_list_orgs(),
        tool_list_workspaces(),
        // Documents
        tool_list_docs(),
        tool_describe_doc(),
        tool_create_doc(),
        tool_delete_doc(),
        tool_move_doc_to_trash(),
        // Tables
        tool_list_tables(),
        tool_create_table(),
        tool_delete_table(),
        // Colonnes
        tool_list_columns(),
        tool_add_columns(),
        // Enregistrements
        tool_list_records(),
        tool_add_records(),
        tool_update_records(),
        tool_delete_records(),
        // SQL
        tool_run_sql(),
    ]
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub async fn dispatch(name: &str, args: &Value, client: &GristClient) -> ToolCallResult {
    let result: Result<String> = match name {
        "grist_list_my_docs"      => do_list_my_docs(client).await,
        "grist_list_orgs"         => do_list_orgs(client).await,
        "grist_list_workspaces"   => do_list_workspaces(args, client).await,
        "grist_list_docs"         => do_list_docs(args, client).await,
        "grist_describe_doc"      => do_describe_doc(args, client).await,
        "grist_create_doc"        => do_create_doc(args, client).await,
        "grist_delete_doc"        => do_delete_doc(args, client).await,
        "grist_move_doc_to_trash" => do_move_doc_to_trash(args, client).await,
        "grist_list_tables"       => do_list_tables(args, client).await,
        "grist_create_table"      => do_create_table(args, client).await,
        "grist_delete_table"      => do_delete_table(args, client).await,
        "grist_list_columns"      => do_list_columns(args, client).await,
        "grist_add_columns"       => do_add_columns(args, client).await,
        "grist_list_records"      => do_list_records(args, client).await,
        "grist_add_records"       => do_add_records(args, client).await,
        "grist_update_records"    => do_update_records(args, client).await,
        "grist_delete_records"    => do_delete_records(args, client).await,
        "grist_run_sql"           => do_run_sql(args, client).await,
        other => Err(anyhow::anyhow!("Outil inconnu : {other}")),
    };
    match result {
        Ok(text)  => ToolCallResult::text(text),
        Err(e)    => ToolCallResult::error(format!("❌ Erreur : {e}")),
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

fn req_str<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    let v = args[key].as_str()
        .ok_or_else(|| anyhow::anyhow!("Paramètre '{key}' manquant ou non-string"))?;
    if v.trim().is_empty() {
        anyhow::bail!("Paramètre '{key}' ne peut pas être vide.");
    }
    Ok(v)
}

fn req_i64(args: &Value, key: &str) -> Result<i64> {
    args[key].as_i64()
        .ok_or_else(|| anyhow::anyhow!("Paramètre '{key}' manquant ou non-entier"))
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

// ===========================================================================
// 1. grist_list_my_docs
// ===========================================================================

fn tool_list_my_docs() -> ToolDef {
    ToolDef {
        name: "grist_list_my_docs".into(),
        description: "Affiche tous les fichiers, documents et tableurs présents dans Grist : \
            workspaces, espaces de travail, documents disponibles. \
            À appeler dès que l'utilisateur mentionne Grist, demande ses fichiers, \
            ses documents, ses tableurs, son workspace ou veut savoir ce qui est \
            disponible dans Grist. \
            Ne nécessite aucun paramètre — retourne directement la structure complète \
            avec org_id, workspace_id et doc_id prêts à l'emploi.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

async fn do_list_my_docs(client: &GristClient) -> Result<String> {
    let orgs = client.get("/orgs", vec![]).await?;
    let orgs_arr = orgs.as_array().cloned().unwrap_or_default();

    if orgs_arr.is_empty() {
        return Ok(pretty(&json!({
            "message": "Aucune organisation accessible avec cette clé API.",
            "orgs": []
        })));
    }

    let mut result = Vec::new();
    for org in &orgs_arr {
        let org_id   = &org["id"];
        let org_name = org["name"].as_str().unwrap_or("?");
        let org_id_str = org_id.to_string();

        match client.get(&format!("/orgs/{org_id_str}/workspaces"), vec![]).await {
            Err(e) => {
                result.push(json!({
                    "org_id":   org_id,
                    "org_name": org_name,
                    "error":    e.to_string()
                }));
            }
            Ok(workspaces) => {
                let ws_arr = workspaces.as_array().cloned().unwrap_or_default();
                let ws_list: Vec<Value> = ws_arr.iter().map(|ws| {
                    let docs: Vec<Value> = ws["docs"].as_array()
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(|doc| json!({
                            "doc_id":   doc["id"],
                            "doc_name": doc["name"],
                            "url_id":   doc["urlId"],
                            "pinned":   doc["isPinned"].as_bool().unwrap_or(false)
                        }))
                        .collect();
                    json!({
                        "workspace_id":   ws["id"],
                        "workspace_name": ws["name"],
                        "docs": docs
                    })
                }).collect();

                result.push(json!({
                    "org_id":     org_id,
                    "org_name":   org_name,
                    "domain":     org["domain"].as_str().unwrap_or(""),
                    "workspaces": ws_list
                }));
            }
        }
    }

    Ok(pretty(&Value::Array(result)))
}

// ===========================================================================
// 2. grist_list_orgs
// ===========================================================================

fn tool_list_orgs() -> ToolDef {
    ToolDef {
        name: "grist_list_orgs".into(),
        description: "Liste toutes les organisations Grist accessibles avec la clé API configurée. \
            Une organisation correspond à un espace d'équipe (team site) ou à l'espace \
            personnel de l'utilisateur. \
            Retourne un tableau JSON d'organisations avec leurs id, name, domain, access.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

async fn do_list_orgs(client: &GristClient) -> Result<String> {
    let data = client.get("/orgs", vec![]).await?;
    Ok(pretty(&data))
}

// ===========================================================================
// 3. grist_list_workspaces
// ===========================================================================

fn tool_list_workspaces() -> ToolDef {
    ToolDef {
        name: "grist_list_workspaces".into(),
        description: "Liste les espaces de travail (workspaces) d'une organisation Grist \
            ainsi que les documents qu'ils contiennent. \
            L'orgId peut être un entier, un sous-domaine (ex: 'gristlabs') \
            ou 'current' pour l'organisation courante.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "org_id": {
                    "type": "string",
                    "description": "Identifiant de l'organisation : entier numérique, sous-domaine ou 'current'."
                }
            },
            "required": ["org_id"]
        }),
    }
}

async fn do_list_workspaces(args: &Value, client: &GristClient) -> Result<String> {
    let org_id = req_str(args, "org_id")?;
    let data = client.get(&format!("/orgs/{org_id}/workspaces"), vec![]).await?;
    Ok(pretty(&data))
}

// ===========================================================================
// 4. grist_list_docs
// ===========================================================================

fn tool_list_docs() -> ToolDef {
    ToolDef {
        name: "grist_list_docs".into(),
        description: "Liste les documents contenus dans un espace de travail Grist. \
            Retourne un tableau JSON de documents avec id, name, isPinned, urlId, access.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "workspace_id": {
                    "type": "integer",
                    "description": "Identifiant entier de l'espace de travail."
                }
            },
            "required": ["workspace_id"]
        }),
    }
}

async fn do_list_docs(args: &Value, client: &GristClient) -> Result<String> {
    let workspace_id = req_i64(args, "workspace_id")?;
    let data = client.get(&format!("/workspaces/{workspace_id}"), vec![]).await?;
    let docs = data.get("docs").cloned().unwrap_or(Value::Array(vec![]));
    Ok(pretty(&docs))
}

// ===========================================================================
// 5. grist_describe_doc
// ===========================================================================

fn tool_describe_doc() -> ToolDef {
    ToolDef {
        name: "grist_describe_doc".into(),
        description: "Décrit les métadonnées d'un document Grist : id, name, isPinned, urlId, \
            access, et l'espace de travail parent avec son organisation.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document (UUID)."
                }
            },
            "required": ["doc_id"]
        }),
    }
}

async fn do_describe_doc(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id = req_str(args, "doc_id")?;
    let data = client.get(&format!("/docs/{doc_id}"), vec![]).await?;
    Ok(pretty(&data))
}

// ===========================================================================
// 6. grist_create_doc
// ===========================================================================

fn tool_create_doc() -> ToolDef {
    ToolDef {
        name: "grist_create_doc".into(),
        description: "Crée un nouveau document Grist vide dans un espace de travail. \
            Retourne l'identifiant (docId) du document créé.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "workspace_id": {
                    "type": "integer",
                    "description": "Identifiant de l'espace de travail destination."
                },
                "name": {
                    "type": "string",
                    "description": "Nom du nouveau document."
                }
            },
            "required": ["workspace_id", "name"]
        }),
    }
}

async fn do_create_doc(args: &Value, client: &GristClient) -> Result<String> {
    let workspace_id = req_i64(args, "workspace_id")?;
    let name = req_str(args, "name")?;
    let data = client.post(
        &format!("/workspaces/{workspace_id}/docs"),
        json!({"name": name}),
    ).await?;
    Ok(pretty(&json!({"doc_id": data})))
}

// ===========================================================================
// 7. grist_delete_doc
// ===========================================================================

fn tool_delete_doc() -> ToolDef {
    ToolDef {
        name: "grist_delete_doc".into(),
        description: "Supprime définitivement un document Grist. \
            ⚠️ Cette action est irréversible. \
            Pour un déplacement récupérable, utiliser grist_move_doc_to_trash.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document à supprimer."
                }
            },
            "required": ["doc_id"]
        }),
    }
}

async fn do_delete_doc(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id = req_str(args, "doc_id")?;
    client.delete(&format!("/docs/{doc_id}")).await?;
    Ok(format!("Document '{doc_id}' supprimé avec succès."))
}

// ===========================================================================
// 8. grist_move_doc_to_trash
// ===========================================================================

fn tool_move_doc_to_trash() -> ToolDef {
    ToolDef {
        name: "grist_move_doc_to_trash".into(),
        description: "Déplace un document Grist dans la corbeille (soft delete). \
            Le document peut être restauré via l'interface Grist. \
            Préférer cet outil à grist_delete_doc pour éviter une perte irréversible.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document à mettre à la corbeille."
                }
            },
            "required": ["doc_id"]
        }),
    }
}

async fn do_move_doc_to_trash(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id = req_str(args, "doc_id")?;
    client.post(&format!("/docs/{doc_id}/remove"), json!({})).await?;
    Ok(format!("Document '{doc_id}' déplacé dans la corbeille."))
}

// ===========================================================================
// 9. grist_list_tables
// ===========================================================================

fn tool_list_tables() -> ToolDef {
    ToolDef {
        name: "grist_list_tables".into(),
        description: "Liste toutes les tables d'un document Grist. \
            Retourne un tableau JSON d'objets avec id (identifiant technique) \
            et fields (tableRef, onDemand). \
            À utiliser avant grist_list_records ou grist_list_columns.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                }
            },
            "required": ["doc_id"]
        }),
    }
}

async fn do_list_tables(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id = req_str(args, "doc_id")?;
    let data = client.get(&format!("/docs/{doc_id}/tables"), vec![]).await?;
    let tables = data.get("tables").cloned().unwrap_or(data);
    Ok(pretty(&tables))
}

// ===========================================================================
// 10. grist_create_table
// ===========================================================================

fn tool_create_table() -> ToolDef {
    ToolDef {
        name: "grist_create_table".into(),
        description: "Crée une nouvelle table dans un document Grist avec des colonnes définies. \
            Retourne l'identifiant de la table créée. \
            Types de colonnes : Text, Numeric, Int, Bool, Date, DateTime, \
            Choice, ChoiceList, Ref:<tableId>, RefList:<tableId>, Attachments, Any.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table (PascalCase, sans espaces). Optionnel."
                },
                "columns": {
                    "type": "array",
                    "description": "Liste des colonnes. Chaque élément : {\"id\": \"nom\", \"fields\": {\"label\": \"Nom\", \"type\": \"Text\"}}",
                    "items": {"type": "object"}
                }
            },
            "required": ["doc_id", "columns"]
        }),
    }
}

async fn do_create_table(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id = req_str(args, "doc_id")?;
    let columns = args["columns"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Paramètre 'columns' manquant ou non-tableau"))?;
    if columns.is_empty() {
        anyhow::bail!("columns doit être une liste non vide.");
    }

    let mut table_def = json!({"columns": columns});
    if let Some(tid) = args["table_id"].as_str() {
        if !tid.trim().is_empty() {
            table_def["id"] = json!(tid);
        }
    }

    let payload = json!({"tables": [table_def]});
    let data = client.post(&format!("/docs/{doc_id}/tables"), payload).await?;
    let tables = data.get("tables").cloned().unwrap_or(data);
    Ok(pretty(&tables))
}

// ===========================================================================
// 11. grist_delete_table
// ===========================================================================

fn tool_delete_table() -> ToolDef {
    ToolDef {
        name: "grist_delete_table".into(),
        description: "Supprime une table d'un document Grist via l'API user actions. \
            ⚠️ Toutes les données de la table seront perdues. Action irréversible.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table à supprimer."
                }
            },
            "required": ["doc_id", "table_id"]
        }),
    }
}

async fn do_delete_table(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id   = req_str(args, "doc_id")?;
    let table_id = req_str(args, "table_id")?;
    let payload  = json!([["RemoveTable", table_id]]);
    client.post(&format!("/docs/{doc_id}/apply"), payload).await?;
    Ok(format!("Table '{table_id}' supprimée avec succès."))
}

// ===========================================================================
// 12. grist_list_columns
// ===========================================================================

fn tool_list_columns() -> ToolDef {
    ToolDef {
        name: "grist_list_columns".into(),
        description: "Liste les colonnes d'une table Grist. \
            Retourne un tableau JSON avec id (identifiant technique) \
            et fields (label, type, formula, isFormula, colRef…). \
            Utiliser avant de lire ou modifier des enregistrements.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table."
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "Si true, inclut les colonnes cachées (ex: manualSort). Défaut: false."
                }
            },
            "required": ["doc_id", "table_id"]
        }),
    }
}

async fn do_list_columns(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id   = req_str(args, "doc_id")?;
    let table_id = req_str(args, "table_id")?;
    let mut params = vec![];
    if args["include_hidden"].as_bool().unwrap_or(false) {
        params.push(("hidden".to_string(), "true".to_string()));
    }
    let data = client.get(&format!("/docs/{doc_id}/tables/{table_id}/columns"), params).await?;
    let cols = data.get("columns").cloned().unwrap_or(data);
    Ok(pretty(&cols))
}

// ===========================================================================
// 13. grist_add_columns
// ===========================================================================

fn tool_add_columns() -> ToolDef {
    ToolDef {
        name: "grist_add_columns".into(),
        description: "Ajoute de nouvelles colonnes à une table Grist existante. \
            Retourne la liste des colonnes créées. \
            Types : Text, Numeric, Int, Bool, Date, DateTime:<tz>, \
            Choice, ChoiceList, Ref:<tableId>, RefList:<tableId>, Attachments, Any.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table cible."
                },
                "columns": {
                    "type": "array",
                    "description": "Colonnes à ajouter. Ex: [{\"id\": \"statut\", \"fields\": {\"label\": \"Statut\", \"type\": \"Choice\"}}]",
                    "items": {"type": "object"}
                }
            },
            "required": ["doc_id", "table_id", "columns"]
        }),
    }
}

async fn do_add_columns(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id   = req_str(args, "doc_id")?;
    let table_id = req_str(args, "table_id")?;
    let columns = args["columns"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Paramètre 'columns' manquant"))?;
    if columns.is_empty() {
        anyhow::bail!("columns doit être une liste non vide.");
    }
    let payload = json!({"columns": columns});
    let data = client.post(&format!("/docs/{doc_id}/tables/{table_id}/columns"), payload).await?;
    let cols = data.get("columns").cloned().unwrap_or(data);
    Ok(pretty(&cols))
}

// ===========================================================================
// 14. grist_list_records
// ===========================================================================

fn tool_list_records() -> ToolDef {
    ToolDef {
        name: "grist_list_records".into(),
        description: "Récupère les enregistrements (lignes) d'une table Grist. \
            Retourne un tableau JSON d'enregistrements, chacun avec id et fields. \
            Supporte le filtrage (filter JSON), le tri (sort) et la limite (limit). \
            Pour des requêtes complexes, préférer grist_run_sql.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table."
                },
                "filter": {
                    "type": "string",
                    "description": "Filtre JSON : objet mappant colonnes à tableaux de valeurs. Ex: '{\"statut\": [\"Actif\"]}'"
                },
                "sort": {
                    "type": "string",
                    "description": "Tri : nom de colonne (ascendant) ou '-nom' (descendant). Ex: 'nom,-date'"
                },
                "limit": {
                    "type": "integer",
                    "description": "Nombre maximum d'enregistrements. 0 = pas de limite."
                }
            },
            "required": ["doc_id", "table_id"]
        }),
    }
}

async fn do_list_records(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id   = req_str(args, "doc_id")?;
    let table_id = req_str(args, "table_id")?;
    let mut params: Vec<(String, String)> = vec![];
    if let Some(f) = args["filter"].as_str() { params.push(("filter".into(), f.into())); }
    if let Some(s) = args["sort"].as_str()   { params.push(("sort".into(), s.into())); }
    if let Some(l) = args["limit"].as_i64()  { params.push(("limit".into(), l.to_string())); }

    let data = client.get(&format!("/docs/{doc_id}/tables/{table_id}/records"), params).await?;
    let records = data.get("records").cloned().unwrap_or(data);
    Ok(pretty(&records))
}

// ===========================================================================
// 15. grist_add_records
// ===========================================================================

fn tool_add_records() -> ToolDef {
    ToolDef {
        name: "grist_add_records".into(),
        description: "Ajoute un ou plusieurs enregistrements dans une table Grist. \
            Retourne la liste des IDs créés. \
            Chaque enregistrement est un objet {\"fields\": {\"colonne\": valeur}}.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table cible."
                },
                "records": {
                    "type": "array",
                    "description": "Enregistrements à créer. Ex: [{\"fields\": {\"nom\": \"Alice\", \"age\": 30}}]",
                    "items": {"type": "object"}
                }
            },
            "required": ["doc_id", "table_id", "records"]
        }),
    }
}

async fn do_add_records(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id   = req_str(args, "doc_id")?;
    let table_id = req_str(args, "table_id")?;
    let records = args["records"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Paramètre 'records' manquant"))?;
    if records.is_empty() {
        anyhow::bail!("records doit être une liste non vide.");
    }
    let payload = json!({"records": records});
    let data = client.post(&format!("/docs/{doc_id}/tables/{table_id}/records"), payload).await?;
    let created = data.get("records").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    let ids: Vec<Value> = created.iter()
        .filter_map(|r| r.get("id").cloned())
        .collect();
    Ok(pretty(&json!({"message": format!("{} enregistrement(s) créé(s).", ids.len()), "ids": ids})))
}

// ===========================================================================
// 16. grist_update_records
// ===========================================================================

fn tool_update_records() -> ToolDef {
    ToolDef {
        name: "grist_update_records".into(),
        description: "Modifie des enregistrements existants dans une table Grist. \
            Chaque enregistrement doit avoir 'id' (entier) et 'fields' avec les colonnes à modifier. \
            Seules les colonnes fournies sont modifiées.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table."
                },
                "records": {
                    "type": "array",
                    "description": "Enregistrements à modifier. Ex: [{\"id\": 1, \"fields\": {\"statut\": \"Clôturé\"}}]",
                    "items": {"type": "object"}
                }
            },
            "required": ["doc_id", "table_id", "records"]
        }),
    }
}

async fn do_update_records(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id   = req_str(args, "doc_id")?;
    let table_id = req_str(args, "table_id")?;
    let records = args["records"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Paramètre 'records' manquant"))?;
    if records.is_empty() {
        anyhow::bail!("records doit être une liste non vide.");
    }
    for r in records {
        if r.get("id").is_none() {
            anyhow::bail!("Chaque enregistrement doit avoir un champ 'id'.");
        }
    }
    let payload = json!({"records": records});
    client.patch(&format!("/docs/{doc_id}/tables/{table_id}/records"), payload).await?;
    Ok(format!("{} enregistrement(s) mis à jour avec succès.", records.len()))
}

// ===========================================================================
// 17. grist_delete_records
// ===========================================================================

fn tool_delete_records() -> ToolDef {
    ToolDef {
        name: "grist_delete_records".into(),
        description: "Supprime des enregistrements d'une table Grist par leurs IDs. \
            ⚠️ Suppression irréversible. \
            Utiliser grist_list_records pour obtenir les IDs avant suppression.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "table_id": {
                    "type": "string",
                    "description": "Identifiant technique de la table."
                },
                "record_ids": {
                    "type": "array",
                    "description": "Liste des IDs (entiers) à supprimer. Ex: [1, 5, 12]",
                    "items": {"type": "integer"}
                }
            },
            "required": ["doc_id", "table_id", "record_ids"]
        }),
    }
}

async fn do_delete_records(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id   = req_str(args, "doc_id")?;
    let table_id = req_str(args, "table_id")?;
    let ids = args["record_ids"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Paramètre 'record_ids' manquant"))?;
    if ids.is_empty() {
        anyhow::bail!("record_ids doit être une liste non vide.");
    }
    client.post(&format!("/docs/{doc_id}/tables/{table_id}/records/delete"), json!(ids)).await?;
    Ok(format!("{} enregistrement(s) supprimé(s) avec succès.", ids.len()))
}

// ===========================================================================
// 18. grist_run_sql
// ===========================================================================

fn tool_run_sql() -> ToolDef {
    ToolDef {
        name: "grist_run_sql".into(),
        description: "Exécute une requête SQL SELECT en lecture seule sur un document Grist. \
            Tous les documents Grist sont des bases SQLite : les tables sont accessibles \
            directement par leur nom (ex: SELECT * FROM Projets). \
            Supporte les requêtes paramétrées (args) pour éviter les injections. \
            ⚠️ Uniquement des requêtes SELECT (pas d'INSERT, UPDATE, DELETE). \
            Timeout par défaut : 1000ms.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "doc_id": {
                    "type": "string",
                    "description": "Identifiant unique du document Grist."
                },
                "sql": {
                    "type": "string",
                    "description": "Requête SQL SELECT. Ex: 'SELECT * FROM Projets WHERE statut = ?'"
                },
                "args": {
                    "type": "array",
                    "description": "Paramètres pour les '?' dans la requête. Ex: [\"Actif\", 2024]",
                    "items": {}
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout en millisecondes (max 1000, défaut 1000)."
                }
            },
            "required": ["doc_id", "sql"]
        }),
    }
}

async fn do_run_sql(args: &Value, client: &GristClient) -> Result<String> {
    let doc_id = req_str(args, "doc_id")?;
    let sql    = req_str(args, "sql")?;

    let sql_upper = sql.trim().to_uppercase();
    if !sql_upper.starts_with("SELECT") && !sql_upper.starts_with("WITH") {
        anyhow::bail!("Seules les requêtes SELECT (ou WITH … SELECT) sont autorisées.");
    }

    let mut payload = json!({"sql": sql.trim()});
    if let Some(sql_args) = args["args"].as_array() {
        if !sql_args.is_empty() {
            payload["args"] = json!(sql_args);
        }
    }
    if let Some(t) = args["timeout_ms"].as_i64() {
        payload["timeout"] = json!(t.min(1000));
    }

    let data = client.post(&format!("/docs/{doc_id}/sql"), payload).await?;
    let records = data.get("records").cloned().unwrap_or(data);
    Ok(pretty(&records))
}
