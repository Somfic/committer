use anyhow::anyhow;
use anyhow::Result;

use crate::git::status::Status;

pub fn prompt(status: &Status) -> Result<bool> {
    inquire::Confirm::new(
        format!(
            "You are behind by {} {}. Do you want to pull from the remote?",
            status.commits_behind,
            if status.commits_behind == 1 {
                "commit"
            } else {
                "commits"
            }
        )
        .as_str(),
    )
    .with_default(true)
    .prompt()
    .map_err(|e| anyhow!(e))
}
