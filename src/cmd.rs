use std::fmt::format;

use anyhow::anyhow;
use anyhow::Result;

pub fn execute(program: &str, args: Vec<&str>) -> Result<String> {
    let output = std::process::Command::new(program)
        .args(&args)
        .output()
        .map_err(|e| anyhow!(e))?;

    if output.status.success() {
        let stdout = std::str::from_utf8(&output.stdout)?;

        Ok(stdout.to_string())
    } else {
        let stderr = std::str::from_utf8(&output.stderr)?.to_string();
        Err(anyhow!(stderr).context(format!(
            "Failed to execute command: {} {}",
            program,
            args.join(" ")
        )))
    }
}
