use std::{
    io,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use once_cell::sync::Lazy;

// @nitrogql/esbuild-register requires different usage
// depending on whether Node.js >= 20.6.0 or not.
static NODE_VERSION_IS_20_6_0_OR_LATER: Lazy<bool> = Lazy::new(|| {
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
    version >= (20, 6)
});

/// Runs given string as JavaScript code, and returns string written to stdout.
pub fn run_node(code: &str) -> io::Result<String> {
    #[cfg(not(target_os = "wasi"))]
    {
        let mut command = Command::new("node");
        command.arg("--no-warnings");
        command.arg("--import=@nitrogql/esbuild-register");
        command.arg("--input-type=module");
        if !*NODE_VERSION_IS_20_6_0_OR_LATER {
            command.arg("--experimenta-loader=@nitrogql/esbuild-register/hook");
        }
        let mut command = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = command.stdin.as_mut().unwrap();
        write!(stdin, "{code}")?;

        let result = command.wait_with_output()?;
        if !result.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Node.js process exited with non-zero status",
            ));
        }

        Ok(String::from_utf8_lossy(&result.stdout).into_owned())
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
pub fn load_default_from_js_file(path: &Path) -> io::Result<String> {
    run_node(&format!(
        r#"
import config from "{}";
import {{ stdout }} from "process";
stdout.write(JSON.stringify(config?.default ?? config));
"#,
        path.display()
    ))
}
