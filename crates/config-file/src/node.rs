use std::{
    cell::RefCell,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command, Stdio},
};

use once_cell::sync::Lazy;

thread_local! {
    // @nitrogql/esbuild-register requires different usage
    // depending on Node.js version.
    static NODE_COMMAND_SERVER: Lazy<RefCell<Child>> = Lazy::new(|| {
        let node_supports_module_register_api = check_if_node_supports_module_register_api();
        let mut command = Command::new("node");
        command.arg("--no-warnings");
        command.arg("--input-type=module");
        if node_supports_module_register_api {
            command.arg("--import=@nitrogql/esbuild-register");
        } else {
            command.arg("--require=@nitrogql/esbuild-register");
            command.arg("--experimental-loader=@nitrogql/esbuild-register/hook");
        }
        command.arg("--eval");
        command.arg("import \"@nitrogql/core/commandServer.js\";");
        command.env("DATA_URL_RESOLUTION_BASE", {
            let mut base = std::env::current_dir().expect("failed to get current dir");
            base.push("__entrypoint__");
            base
        });
        let child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        RefCell::new(child)
    });
}

/// @nitrogql/esbuild-register requires different usage
/// depending on Node.js version.
fn check_if_node_supports_module_register_api() -> bool {
    let command = Command::new("node")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let result = command.wait_with_output().unwrap();
    if !result.status.success() {
        panic!("Node.js process exited with non-zero status");
    }

    let version = String::from_utf8_lossy(&result.stdout);
    let version = version.trim();
    let version = version.strip_prefix('v').unwrap_or(version);
    let version = version.split('.').collect::<Vec<_>>();
    let version = (
        version[0].parse::<u32>().unwrap(),
        version[1].parse::<u32>().unwrap(),
    );
    version >= (20, 6) || (version.0 == 18 && version.1 >= 19)
}

/// Runs given string as JavaScript code, and returns the value exported as default export.
/// Value is serialized as JSON.
pub async fn run_node(code: &str) -> io::Result<String> {
    #[cfg(not(target_os = "wasi"))]
    {
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
            Ok(result)
        })
    }
    #[cfg(target_os = "wasi")]
    {
        use crate::execute::execute_js;
        execute_js(code).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to execute config file: {}", err),
            )
        })
    }
}

/// Load the default export of a JS file by executing it and returning the `module.exports` value.
pub async fn load_default_from_js_file(path: &Path) -> io::Result<String> {
    run_node(&format!(
        r#"
import config from "{}";
import {{ stdout }} from "process";
export default config?.default ?? config;
"#,
        path.display()
    ))
    .await
}
