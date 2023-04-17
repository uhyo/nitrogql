use std::{
    io,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

/// Load config from a JS file by executing it and returning the `module.exports` value.
pub fn load_config_from_js_file(path: &Path) -> io::Result<String> {
    let mut command = Command::new("node")
        .arg("--input-type=module")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = command.stdin.as_mut().unwrap();
    write!(
        stdin,
        r#"
import config from "{}";
import {{ stdout }} from "process";
stdout.write(JSON.stringify(config));
"#,
        path.display()
    )?;

    let result = command.wait_with_output()?;
    if !result.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Node.js process exited with non-zero status",
        ));
    }

    Ok(String::from_utf8_lossy(&result.stdout).into_owned())
}
