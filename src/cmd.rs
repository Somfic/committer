use anyhow::anyhow;
use anyhow::Result;

pub fn execute(cmd: &str) -> Result<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;

    if output.status.success() {
        Ok(std::str::from_utf8(&output.stdout)?.into())
    } else {
        let stderr = std::str::from_utf8(&output.stderr)?.to_string();
        Err(anyhow!(stderr))
    }
}
