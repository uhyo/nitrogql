use std::{
    io,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

/// Runs given string as JavaScript code, and returns string written to stdout.
pub fn run_node(code: &str) -> io::Result<String> {
    #[cfg(not(target_os = "wasi"))]
    {
        let mut command = Command::new("node")
            .arg("--no-warnings")
            .arg("--loader=esbuild-register/loader")
            .arg("--require=esbuild-register")
            .arg("--input-type=module")
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
        use crate::execute::execute_node;
        execute_node(code).map_err(|err| {
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
