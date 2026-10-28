//! Content mapper for TypeScript 7.1+.
//!
//! TypeScript spawns this as a child process and asks it to transform GraphQL files
//! into virtual TypeScript code. This replaces generation of `.d.graphql.ts` files.
//!
//! Protocol is JSON-RPC 2.0 over stdio. Framing of messages is handled by the JavaScript host.

use std::{collections::HashMap, path::Path};

use log::{debug, error};
use nitrogql_error::ErrorCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use self::{
    project::{MapperOptions, Project},
    protocol::{
        CloseProjectParams, INTERNAL_ERROR, INVALID_PARAMS, InitializeParams, InitializeResult,
        JsonRpcError, JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND, OpenProjectParams,
        OpenProjectResult, OptionDiagnostic, PositionEncoding, TransformParams,
    },
    transform::transform,
};

mod fallback;
mod io;
mod logger;
mod position;
mod project;
mod protocol;
#[cfg(test)]
mod tests;
mod transform;

/// Diagnostic source displayed for diagnostics reported by this mapper.
const DIAGNOSTIC_SOURCE: &str = "nitrogql";

/// Runs the content mapper until the connection is closed.
pub async fn run_content_mapper() {
    logger::init_logger();
    colored::control::set_override(false);
    let mut server = Server::default();
    while let Some(message) = io::receive().await {
        let request: JsonRpcRequest = match serde_json::from_str(&message) {
            Ok(request) => request,
            Err(err) => {
                error!("Failed to parse message: {err}");
                continue;
            }
        };
        debug!("Received {} request", request.method);
        let result = server.handle(&request.method, request.params).await;
        // Notifications are not responded to.
        let Some(id) = request.id else {
            continue;
        };
        let response = match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0",
                id: &id,
                result: Some(result),
                error: None,
            },
            Err(err) => JsonRpcResponse {
                jsonrpc: "2.0",
                id: &id,
                result: None,
                error: Some(err),
            },
        };
        io::send(&serde_json::to_string(&response).expect("Failed to serialize response"));
    }
}

#[derive(Default)]
struct Server {
    encoding: Option<PositionEncoding>,
    projects: HashMap<String, Project>,
    /// The last closed project.
    /// TypeScript closes and reopens a project when a watched file changes,
    /// and the reopened project reuses what is unchanged.
    /// As the reopen follows the close, only one project is kept,
    /// so that projects closed for good do not stay in memory.
    closed_project: Option<Project>,
}

impl Server {
    async fn handle(&mut self, method: &str, params: Value) -> Result<Value, JsonRpcError> {
        match method {
            "initialize" => {
                let params: InitializeParams = parse_params(params)?;
                let utf8 = serde_json::to_value(PositionEncoding::Utf8).unwrap();
                let utf16 = serde_json::to_value(PositionEncoding::Utf16).unwrap();
                let encoding = if params.position_encodings.contains(&utf8) {
                    PositionEncoding::Utf8
                } else if params.position_encodings.contains(&utf16) {
                    PositionEncoding::Utf16
                } else {
                    return Err(JsonRpcError {
                        code: INVALID_PARAMS,
                        message: "Neither UTF-8 nor UTF-16 position encoding is supported"
                            .to_owned(),
                    });
                };
                self.encoding = Some(encoding);
                to_result(InitializeResult {
                    position_encoding: encoding,
                    diagnostic_source: DIAGNOSTIC_SOURCE,
                })
            }
            "openProject" => {
                let params: OpenProjectParams = parse_params(params)?;
                let (options, option_diagnostics) = parse_options(params.options.as_ref());
                let ts_config_file = Path::new(&params.config_file_name);
                let previous = self
                    .closed_project
                    .take_if(|project| project.ts_config_file == ts_config_file);
                let project = Project::open(ts_config_file, &options, previous).await;
                let result = OpenProjectResult {
                    config_identity: project.config_identity.clone(),
                    watched_files: project
                        .watched_files
                        .iter()
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect(),
                    option_diagnostics,
                };
                self.projects.insert(params.project_handle, project);
                to_result(result)
            }
            "transform" => {
                let params: TransformParams = parse_params(params)?;
                let encoding = self.encoding.ok_or_else(|| JsonRpcError {
                    code: INTERNAL_ERROR,
                    message: "Content mapper is not initialized".to_owned(),
                })?;
                let project =
                    self.projects
                        .get(&params.project_handle)
                        .ok_or_else(|| JsonRpcError {
                            code: INVALID_PARAMS,
                            message: format!("Unknown project: {}", params.project_handle),
                        })?;
                to_result(transform(
                    project,
                    Path::new(&params.file_name),
                    &params.content,
                    encoding,
                ))
            }
            "closeProject" => {
                let params: CloseProjectParams = parse_params(params)?;
                if let Some(project) = self.projects.remove(&params.project_handle) {
                    self.closed_project = Some(project);
                }
                Ok(Value::Null)
            }
            _ => Err(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {method}"),
            }),
        }
    }
}

fn parse_params<T: DeserializeOwned>(params: Value) -> Result<T, JsonRpcError> {
    serde_json::from_value(params).map_err(|err| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid params: {err}"),
    })
}

fn to_result(result: impl Serialize) -> Result<Value, JsonRpcError> {
    serde_json::to_value(result).map_err(|err| JsonRpcError {
        code: INTERNAL_ERROR,
        message: format!("Failed to serialize result: {err}"),
    })
}

/// Parses options given in tsconfig.json.
fn parse_options(options: Option<&Value>) -> (MapperOptions, Vec<OptionDiagnostic>) {
    let mut result = MapperOptions::default();
    let mut diagnostics = vec![];
    let Some(options) = options else {
        return (result, diagnostics);
    };
    let Some(options) = options.as_object() else {
        diagnostics.push(OptionDiagnostic {
            path: vec![],
            message_text: "Options of nitrogql content mapper must be an object.".to_owned(),
            code: ErrorCode::InvalidMapperOption.number(),
        });
        return (result, diagnostics);
    };
    for (key, value) in options {
        match (key.as_str(), value) {
            ("configFile", Value::String(path)) => {
                result.config_file = Some(path.into());
            }
            ("configFile", _) => diagnostics.push(OptionDiagnostic {
                path: vec![json!(key)],
                message_text: "'configFile' must be a string.".to_owned(),
                code: ErrorCode::InvalidMapperOption.number(),
            }),
            _ => diagnostics.push(OptionDiagnostic {
                path: vec![json!(key)],
                message_text: format!("Unknown option '{key}'."),
                code: ErrorCode::InvalidMapperOption.number(),
            }),
        }
    }
    (result, diagnostics)
}
