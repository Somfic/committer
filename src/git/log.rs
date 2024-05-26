use crate::cmd::execute;
use anyhow::Result;

pub fn log() -> Result<Vec<Commit>> {
    let commits = execute(
        "git",
        vec!["--no-pager", "log", "--decorate=short", "--pretty=oneline"],
    )?;

    Ok(commits
        .lines()
        .map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            Commit {
                sha: parts[0].to_string(),
                message: parts[1].to_string(),
            }
        })
        .collect())
}

pub struct Commit {
    pub sha: String,
    pub message: String,
}
