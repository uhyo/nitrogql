use std::{io, path::Path};

#[cfg(not(target_os = "wasi"))]
use std::{
    cell::RefCell,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
};

#[cfg(not(target_os = "wasi"))]
use once_cell::sync::Lazy;

#[cfg(not(target_os = "wasi"))]
thread_local! {
    // The command server runs commands in a worker thread and decides
    // how to run TypeScript files (Node.js' type stripping or
    // @nitrogql/esbuild-register), just like the wasm build.
    static NODE_COMMAND_SERVER: Lazy<RefCell<Child>> = Lazy::new(|| {
        let child = Command::new("node")
            .arg("--no-warnings")
            .arg("--input-type=module")
            .arg("--eval")
            .arg("import \"@nitrogql/core/commandStdioServer.js\";")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        RefCell::new(child)
    });
}

/// Runs given string as JavaScript code, and returns the value exported as default export.
/// Value is serialized as JSON.
pub async fn run_node(code: &str) -> io::Result<String> {
    #[cfg(not(target_os = "wasi"))]
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum RunNodeResult {
            Result { result: serde_json::Value },
            Error { error: serde_json::Value },
        }

        NODE_COMMAND_SERVER.with(|server| {
            let mut server = server.borrow_mut();
            let mut stdin = server.stdin.as_mut().expect("failed to open stdin");
            serde_json::to_writer(&mut stdin, &code)?;
            writeln!(&mut stdin)?;
            // read one line from stdout
            let mut stdout = server.stdout.as_mut().expect("failed to open stdout");
            let mut reader = BufReader::new(&mut stdout);
            let mut result = String::new();
            reader.read_line(&mut result)?;
            let result: RunNodeResult = serde_json::from_str(&result)?;
            match result {
                RunNodeResult::Result { result } => {
                    let result = serde_json::to_string(&result)?;
                    Ok(result)
                }
                RunNodeResult::Error { error } => {
                    let error = serde_json::to_string(&error)?;
                    Err(io::Error::other(error))
                }
            }
        })
    }
    #[cfg(target_os = "wasi")]
    {
        use crate::execute::execute_js;
        execute_js(code)
            .await
            .map_err(|err| io::Error::other(format!("Failed to execute config file: {}", err)))
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
