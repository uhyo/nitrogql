//! Types for the TypeScript content mapper protocol.
//! Messages are exchanged as JSON-RPC 2.0 over stdio.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct JsonRpcRequest {
    /// Absent for notifications.
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Serialize, Debug)]
pub struct JsonRpcResponse<'a, T> {
    pub jsonrpc: &'static str,
    pub id: &'a Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl<'a, T> JsonRpcResponse<'a, T> {
    pub fn new(id: &'a Value, result: Result<T, JsonRpcError>) -> Self {
        let (result, error) = match result {
            Ok(result) => (Some(result), None),
            Err(error) => (None, Some(error)),
        };
        JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result,
            error,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const INVALID_PARAMS: i32 = -32602;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INTERNAL_ERROR: i32 = -32603;

/// Result of a request.
/// Each variant is serialized as is, so that results are serialized only once.
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum MethodResult<'a> {
    Initialize(InitializeResult),
    OpenProject(OpenProjectResult),
    Transform(TransformResult<'a>),
    /// Serialized as `null`.
    Null,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum PositionEncoding {
    #[serde(rename = "utf-8")]
    Utf8,
    #[serde(rename = "utf-16")]
    Utf16,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub position_encodings: Vec<Value>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub position_encoding: PositionEncoding,
    pub diagnostic_source: &'static str,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenProjectParams {
    /// Absolute tsconfig path, or an empty string for a project without a config file.
    pub config_file_name: String,
    pub project_handle: String,
    /// Object from the contentMappers entry, when specified.
    pub options: Option<Value>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenProjectResult {
    pub config_identity: String,
    pub watched_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub option_diagnostics: Vec<OptionDiagnostic>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OptionDiagnostic {
    pub path: Vec<Value>,
    pub message_text: String,
    /// Number of an `ErrorCode`.
    pub code: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransformParams {
    pub file_name: String,
    pub content: String,
    pub project_handle: String,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransformResult<'a> {
    /// Borrowed when the text is cached, e.g. the schema module.
    pub text: Cow<'a, str>,
    pub extension: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<SpanMapping>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<MapperDiagnostic>,
}

impl<'a> TransformResult<'a> {
    pub fn new(
        text: impl Into<Cow<'a, str>>,
        mappings: Vec<SpanMapping>,
        diagnostics: Vec<MapperDiagnostic>,
    ) -> Self {
        TransformResult {
            text: text.into(),
            extension: ".ts",
            mappings,
            diagnostics,
        }
    }
}

/// `[virtualStart, virtualLength, originalStart, originalLength, kind]`
pub type SpanMapping = (usize, usize, usize, usize, SpanMapKind);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpanMapKind {
    /// Virtual text has the same content as the original text.
    Verbatim = 0,
    /// Virtual text may differ from the original text.
    Atom = 1,
}

impl Serialize for SpanMapKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MapperDiagnostic {
    pub message_text: String,
    pub start: usize,
    pub length: usize,
    /// Number of an `ErrorCode`.
    pub code: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloseProjectParams {
    pub project_handle: String,
}
