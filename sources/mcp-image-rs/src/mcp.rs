// ============================================================================
// Auteur  : Pierre COUGET  ktulu.analog@gmail.com
// Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
// Année   : 2026
// ============================================================================

// ============================================================================
// mcp.rs — Types JSON-RPC 2.0 / MCP HTTP Streamable (spec 2025-03-26)
// ============================================================================
// Identique à mcp-system-rs, avec un ContentBlock étendu pour supporter
// le type "image" (données base64) en plus du type "text".
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Enveloppes JSON-RPC
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id:      Option<Value>,
    pub method:  String,
    pub params:  Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id:     Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error:  Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code:    i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }

    pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError { code, message: message.into() }),
        }
    }
}

// ---------------------------------------------------------------------------
// Capacités du serveur (initialize)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ServerInfo {
    pub name:    &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct Capabilities {
    pub tools: ToolsCapability,
}

#[derive(Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: &'static str,
    pub capabilities:     Capabilities,
    #[serde(rename = "serverInfo")]
    pub server_info:      ServerInfo,
    pub instructions:     &'static str,
}

// ---------------------------------------------------------------------------
// Liste d'outils (tools/list)
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct ToolDef {
    pub name:         String,
    pub description:  String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolDef>,
}

// ---------------------------------------------------------------------------
// Résultat d'appel d'outil (tools/call)
//
// Le protocole MCP supporte plusieurs types de ContentBlock.
// Pour la génération d'image, on utilise le type "image" avec data base64.
//
// Spec MCP :
//   { "type": "image", "data": "<base64>", "mimeType": "image/png" }
//   { "type": "text",  "text": "<message>" }
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentBlock {
    /// Bloc texte — utilisé pour les messages d'erreur ou de statut
    Text {
        text: String,
    },
    /// Bloc image — données PNG/JPEG encodées en base64
    Image {
        data:     String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

#[derive(Serialize)]
pub struct ToolCallResult {
    pub content:  Vec<ContentBlock>,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

impl ToolCallResult {
    /// Retourne un bloc texte simple (succès)
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content:  vec![ContentBlock::Text { text: s.into() }],
            is_error: false,
        }
    }

    /// Retourne une image PNG encodée base64
    #[allow(dead_code)]
    pub fn image(base64_data: String, mime_type: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Image {
                data:      base64_data,
                mime_type: mime_type.into(),
            }],
            is_error: false,
        }
    }

    /// Retourne une image avec un texte de description en plus
    pub fn image_with_caption(
        base64_data: String,
        mime_type: impl Into<String>,
        caption: impl Into<String>,
    ) -> Self {
        Self {
            content: vec![
                ContentBlock::Image {
                    data:      base64_data,
                    mime_type: mime_type.into(),
                },
                ContentBlock::Text { text: caption.into() },
            ],
            is_error: false,
        }
    }

    /// Retourne une erreur textuelle
    pub fn error(s: impl Into<String>) -> Self {
        Self {
            content:  vec![ContentBlock::Text { text: s.into() }],
            is_error: true,
        }
    }
}
