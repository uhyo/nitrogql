//! Content mapper for TypeScript 7.1+.
//!
//! TypeScript spawns this as a child process and asks it to transform GraphQL files
//! into virtual TypeScript code. This replaces generation of `.d.graphql.ts` files.
//!
//! Protocol is JSON-RPC 2.0 over stdio, with messages framed by `Content-Length` headers.

use std::{collections::HashMap, path::Path};

use log::{debug, error};
use nitrogql_config_file::stop_node;
use nitrogql_error::ErrorCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use self::{
    project::{MapperOptions, Project},
    protocol::{
        CloseProjectParams, INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, InitializeParams,
        InitializeResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND,
        MethodResult, OpenProjectParams, OpenProjectResult, OptionDiagnostic, PARSE_ERROR,
        PositionEncoding, TransformParams, TransformResult,
    },
    transform::transform,
};

mod fallback;
mod framing;
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
    let mut connection = io::Connection::default();
    let mut server = Server::default();
    while let Some(message) = connection.receive().await {
        if let Some(response) = server.handle_message(&message).await {
            connection.send(&response);
        }
    }
    stop_node();
}

/// Returns given path in the form TypeScript uses, i.e. with forward slashes even on Windows.
fn ts_path(path: &Path) -> String {
    let path = path.to_string_lossy();
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.into_owned()
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
    /// Handles a JSON-RPC message and returns the response to send, if any.
    async fn handle_message(&mut self, message: &str) -> Option<String> {
        let request: JsonRpcRequest = match serde_json::from_str(message) {
            Ok(request) => request,
            Err(err) => {
                error!("Failed to parse message: {err}");
                let (id, error) = invalid_message_error(message, err);
                return Some(serialize_response(&JsonRpcResponse::<()>::new(
                    &id,
                    Err(error),
                )));
            }
        };
        debug!("Received {} request", request.method);
        let result = self.handle(&request.method, request.params).await;
        // Notifications are not responded to.
        let id = request.id?;
        Some(serialize_response(&JsonRpcResponse::new(&id, result)))
    }

    async fn handle(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<MethodResult<'_>, JsonRpcError> {
        match method {
            "initialize" => self
                .initialize(parse_params(params)?)
                .map(MethodResult::Initialize),
            "openProject" => Ok(MethodResult::OpenProject(
                self.open_project(parse_params(params)?).await,
            )),
            "transform" => self
                .transform(parse_params(params)?)
                .map(MethodResult::Transform),
            "closeProject" => {
                self.close_project(parse_params(params)?);
                Ok(MethodResult::Null)
            }
            _ => Err(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {method}"),
            }),
        }
    }

    fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult, JsonRpcError> {
        let utf8 = serde_json::to_value(PositionEncoding::Utf8).unwrap();
        let utf16 = serde_json::to_value(PositionEncoding::Utf16).unwrap();
        let encoding = if params.position_encodings.contains(&utf8) {
            PositionEncoding::Utf8
        } else if params.position_encodings.contains(&utf16) {
            PositionEncoding::Utf16
        } else {
            return Err(JsonRpcError {
                code: INVALID_PARAMS,
                message: "Neither UTF-8 nor UTF-16 position encoding is supported".to_owned(),
            });
        };
        self.encoding = Some(encoding);
        Ok(InitializeResult {
            position_encoding: encoding,
            diagnostic_source: DIAGNOSTIC_SOURCE,
        })
    }

    async fn open_project(&mut self, params: OpenProjectParams) -> OpenProjectResult {
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
                .map(|path| ts_path(path))
                .collect(),
            option_diagnostics,
        };
        self.projects.insert(params.project_handle, project);
        result
    }

    fn transform(&self, params: TransformParams) -> Result<TransformResult<'_>, JsonRpcError> {
        let encoding = self.encoding.ok_or_else(|| JsonRpcError {
            code: INTERNAL_ERROR,
            message: "Content mapper is not initialized".to_owned(),
        })?;
        let project = self
            .projects
            .get(&params.project_handle)
            .ok_or_else(|| JsonRpcError {
                code: INVALID_PARAMS,
                message: format!("Unknown project: {}", params.project_handle),
            })?;
        Ok(transform(
            project,
            Path::new(&params.file_name),
            &params.content,
            encoding,
        ))
    }

    fn close_project(&mut self, params: CloseProjectParams) {
        if let Some(project) = self.projects.remove(&params.project_handle) {
            self.closed_project = Some(project);
        }
    }
}

/// Returns the id to respond with and the error for a message that is not a valid request.
fn invalid_message_error(message: &str, err: serde_json::Error) -> (Value, JsonRpcError) {
    match serde_json::from_str::<Value>(message) {
        Err(err) => (
            Value::Null,
            JsonRpcError {
                code: PARSE_ERROR,
                message: format!("Parse error: {err}"),
            },
        ),
        // The id is used if it can be read, so that the client can tell which request failed.
        Ok(value) => (
            value.get("id").cloned().unwrap_or(Value::Null),
            JsonRpcError {
                code: INVALID_REQUEST,
                message: format!("Invalid request: {err}"),
            },
        ),
    }
}

fn serialize_response(response: &impl Serialize) -> String {
    serde_json::to_string(response).expect("Failed to serialize response")
}

fn parse_params<T: DeserializeOwned>(params: Value) -> Result<T, JsonRpcError> {
    serde_json::from_value(params).map_err(|err| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("Invalid params: {err}"),
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
