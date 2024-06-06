use anyhow::anyhow;
use anyhow::Result;

use crate::git::status::Status;

pub fn prompt(status: &Status) -> Result<bool> {
    inquire::Confirm::new(
        format!(
            "You are ahead by {} {}. Do you want to push to the remote?",
            status.commits_ahead,
            if status.commits_ahead == 1 {
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
