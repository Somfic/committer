use anyhow::anyhow;
use anyhow::Result;

pub fn execute(program: &str, args: Vec<&str>) -> Result<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| anyhow!(e))?;

    if output.status.success() {
        Ok(std::str::from_utf8(&output.stdout)?.into())
    } else {
        let stderr = std::str::from_utf8(&output.stderr)?.to_string();
        Err(anyhow!(stderr))
    }
}
