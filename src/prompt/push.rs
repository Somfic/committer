use anyhow::anyhow;
use anyhow::Result;

pub fn prompt() -> Result<bool> {
    inquire::Confirm::new("Push to the remote?")
        .with_default(true)
        .prompt()
        .map_err(|e| anyhow!(e))
}
