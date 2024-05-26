use crate::cmd::execute;
use anyhow::{Context, Result};

pub struct Status {
    pub commits_behind: u32,
    pub commits_ahead: u32,
    pub message: String,
}

pub fn status() -> Result<Status> {
    execute("git", vec!["remote", "update"])?;
    let status = execute(
        "git",
        vec!["--no-pager", "status", "-s", "-b", "--porcelain"],
    )?;

    let regex = regex::Regex::new(
        r"## (.+?)(?:\.{3})?(?:\s\[(?:ahead (\d+))?(?:, )?(?:behind (\d+))?\])?$",
    )?;

    if let Some(captures) = regex.captures(&status) {
        let commits_ahead = captures
            .get(2)
            .map(|m| m.as_str().parse().unwrap())
            .unwrap_or(0);
        let commits_behind = captures
            .get(3)
            .map(|m| m.as_str().parse().unwrap())
            .unwrap_or(0);

        return Ok(Status {
            commits_ahead,
            commits_behind,
            message: status,
        });
    }

    Ok(Status {
        commits_ahead: 0,
        commits_behind: 0,
        message: status,
    })
}
