use crate::cmd::execute;
use anyhow::Result;

pub fn log() -> Result<Vec<Commit>> {
    Ok(vec![])
}

/// Returns all commits since the given tag, with their subject and body.
/// If tag is "v0.0.0", returns all commits from the root.
pub fn commits_since(tag: &str) -> Result<Vec<Commit>> {
    let range = if tag == "v0.0.0" {
        // Get root commit and log from there
        let root = execute("git", vec!["rev-list", "--max-parents=0", "HEAD"])?;
        format!("{}..HEAD", root.trim())
    } else {
        format!("{}..HEAD", tag)
    };

    let output = execute(
        "git",
        vec![
            "--no-pager",
            "log",
            &range,
            "--pretty=format:%H%x00%s%x00%b%x00",
        ],
    )?;

    let commits = output
        .split("\x00\n")
        .filter(|s| !s.trim().is_empty())
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.splitn(3, '\x00').collect();
            if parts.len() >= 2 {
                let subject = parts[1].trim().to_string();
                let body = parts.get(2).unwrap_or(&"").trim().to_string();
                Some(Commit::from_log(subject, body))
            } else {
                None
            }
        })
        .collect();

    Ok(commits)
}

#[derive(Debug)]
pub struct Commit {
    pub emoji: Option<String>,
    pub scope: Option<String>,
    pub message: String,
    pub body: String,
    pub semver: Option<SemVerBump>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemVerBump {
    Major,
    Minor,
    Patch,
}

impl Commit {
    pub fn from_log(subject: String, body: String) -> Self {
        let semver = if body.to_lowercase().contains("semver: major") {
            Some(SemVerBump::Major)
        } else if body.to_lowercase().contains("semver: minor") {
            Some(SemVerBump::Minor)
        } else if body.to_lowercase().contains("semver: patch") {
            Some(SemVerBump::Patch)
        } else {
            None
        };

        let commit_regex =
            regex::Regex::new(r"^([^\w\s:()]+)?\s*(?:\(?([^\)]+)\)?\s*:)?\s*([\s\w]*)$").unwrap();

        let captures = commit_regex.captures(&subject);

        match captures {
            Some(captures) => {
                let emoji = captures.get(1).map(|m| m.as_str().trim().to_string());
                let scope = captures.get(2).map(|m| m.as_str().trim().to_string());
                let message = captures
                    .get(3)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();

                Self {
                    emoji,
                    scope,
                    message,
                    body,
                    semver,
                }
            }
            None => Self {
                emoji: None,
                scope: None,
                message: subject,
                body,
                semver,
            },
        }
    }

    pub fn new(message: impl Into<String>) -> Self {
        Self::from_log(message.into(), String::new())
    }
}
