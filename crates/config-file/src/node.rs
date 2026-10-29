use std::{io, path::Path};

/// While a session is alive, code run with `run_node` shares one Node.js worker.
///
/// Otherwise, each code runs in a new worker, so that changed files are loaded again.
/// The content mapper loads a project in a session, and the CLI runs in one session.
/// Sessions can be nested; the session lasts until the outermost one ends.
pub struct NodeSession(());

impl NodeSession {
    pub fn begin() -> Self {
        #[cfg(target_os = "wasi")]
        crate::execute::begin_node_session();
        #[cfg(not(target_os = "wasi"))]
        native::begin_session();
        NodeSession(())
    }
}

impl Drop for NodeSession {
    fn drop(&mut self) {
        #[cfg(target_os = "wasi")]
        crate::execute::end_node_session();
        #[cfg(not(target_os = "wasi"))]
        native::end_session();
    }
}

/// Runs given string as JavaScript code, and returns the value exported as default export.
/// Value is serialized as JSON.
pub async fn run_node(code: &str) -> io::Result<String> {
    #[cfg(not(target_os = "wasi"))]
    {
        native::run_node(code)
    }
    #[cfg(target_os = "wasi")]
    {
        use crate::execute::execute_js;
        execute_js(code)
            .await
            .map_err(|err| io::Error::other(format!("Failed to execute config file: {}", err)))
    }
}

/// Node.js command server for the native build.
#[cfg(not(target_os = "wasi"))]
mod native {
    use std::{
        cell::{Cell, RefCell},
        io::{self, BufRead, BufReader, Write},
        process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    };

    use serde_json::json;

    /// Running command server.
    struct Server {
        _child: Child,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    }

    thread_local! {
        /// Started by the first code to run.
        static SERVER: RefCell<Option<Server>> = const { RefCell::new(None) };
        /// Depth of nested sessions. The server only sees the outermost one.
        static SESSION_DEPTH: Cell<usize> = const { Cell::new(0) };
    }

    impl Server {
        /// Starts the command server.
        /// It runs code in a worker thread and decides how to run TypeScript files
        /// (Node.js' type stripping or @nitrogql/esbuild-register), just like the wasm build.
        fn start() -> io::Result<Self> {
            let mut child = Command::new("node")
                .arg("--no-warnings")
                .arg("--input-type=module")
                .arg("--eval")
                .arg("import \"@nitrogql/core/commandStdioServer.js\";")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?;
            let stdin = child.stdin.take().expect("stdin is piped");
            let stdout = BufReader::new(child.stdout.take().expect("stdout is piped"));
            let mut server = Server {
                _child: child,
                stdin,
                stdout,
            };
            if SESSION_DEPTH.get() > 0 {
                server.send(&json!({ "session": "begin" }))?;
            }
            Ok(server)
        }

        /// Sends one line of JSON.
        fn send(&mut self, message: &serde_json::Value) -> io::Result<()> {
            serde_json::to_writer(&mut self.stdin, message)?;
            writeln!(&mut self.stdin)?;
            self.stdin.flush()
        }
    }

    pub fn begin_session() {
        let depth = SESSION_DEPTH.get();
        SESSION_DEPTH.set(depth + 1);
        if depth == 0 {
            notify_session("begin");
        }
    }

    pub fn end_session() {
        let depth = SESSION_DEPTH.get();
        SESSION_DEPTH.set(depth.saturating_sub(1));
        if depth == 1 {
            notify_session("end");
        }
    }

    /// Tells the server that a session has begun or ended.
    /// A server that is not started yet learns it when started.
    fn notify_session(session: &str) {
        SERVER.with_borrow_mut(|server| {
            if let Some(server) = server
                && let Err(err) = server.send(&json!({ "session": session }))
            {
                log::error!("Failed to send session to Node.js: {err}");
            }
        });
    }

    pub fn run_node(code: &str) -> io::Result<String> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum RunNodeResult {
            Result { result: serde_json::Value },
            Error { error: serde_json::Value },
        }

        SERVER.with_borrow_mut(|server| {
            let server = match server {
                Some(server) => server,
                None => server.insert(Server::start()?),
            };
            server.send(&json!(code))?;
            let mut result = String::new();
            server.stdout.read_line(&mut result)?;
            let result: RunNodeResult = serde_json::from_str(&result)?;
            match result {
                RunNodeResult::Result { result } => Ok(serde_json::to_string(&result)?),
                RunNodeResult::Error { error } => {
                    Err(io::Error::other(serde_json::to_string(&error)?))
                }
            }
        })
    }
}

/// Load the default export of a JS file by executing it and returning the `module.exports` value.
pub async fn load_default_from_js_file(path: &Path) -> io::Result<String> {
    run_node(&format!(
        r#"
import {{ pathToFileURL }} from "url";
const config = (await import(pathToFileURL("{}").toString())).default;
export default config?.default ?? config;
"#,
        path.to_string_lossy().escape_default()
    ))
    .await
}
