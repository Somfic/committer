use std::collections::HashMap;

use crate::cmd::execute;
use anyhow::Result;

pub fn log() -> Result<Vec<Commit>> {
    let commits = execute(
        "git",
        vec![
            "--no-pager",
            "log",
            "--all",
            "--decorate=short",
            "--pretty=format:%s",
        ],
    )?;

    Ok(commits
        .lines()
        .filter(|line| !line.is_empty())
        .map(Commit::new)
        .collect::<Vec<Commit>>())
}

pub fn majors_since(tag: &String) -> Result<HashMap<String, Vec<Commit>>> {
    since(tag, "semver: major")
}

pub fn minors_since(tag: &String) -> Result<HashMap<String, Vec<Commit>>> {
    since(tag, "semver: minor")
}

pub fn patches_since(tag: &String) -> Result<HashMap<String, Vec<Commit>>> {
    since(tag, "semver: patch")
}

fn since(tag: &String, grep: &str) -> Result<HashMap<String, Vec<Commit>>> {
    let grep = format!("--grep={}", grep);
    let since = format!("{}..HEAD", tag);

    let mut args = vec![
        "--no-pager",
        "log",
        &grep,
        "--all",
        "--decorate=short",
        "--pretty=format:%s",
    ];

    if tag != "0.0.0" {
        args.push(&since);
    };

    let commits = execute("git", args)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(Commit::new)
        .collect::<Vec<Commit>>();

    // Group commits by scope
    let mut grouped_commits = HashMap::new();
    for commit in commits {
        let scope = commit.scope.clone().unwrap_or("".to_string());
        grouped_commits
            .entry(scope)
            .or_insert_with(Vec::new)
            .push(commit);
    }

    Ok(grouped_commits)
}

#[derive(Debug)]
pub struct Commit {
    pub emoji: Option<String>,
    pub scope: Option<String>,
    pub message: String,
}

impl Commit {
    pub fn new(message: impl Into<String>) -> Self {
        // Commit examples:
        // Initial commit
        // 📌 Pin anyhow crate version
        // 🔖 (other): Version bump
        // ⚰️ (cli): Remove dead code

        let message = message.into();
        // Use \p{Emoji_Presentation} to match emojis
        let commit_regex =
            regex::Regex::new(r"^([^\w\s:()]+)?\s*(?:\(?([^\)]+)\)?\s*:)?\s*([\s\w]*)$").unwrap();

        let captures = commit_regex.captures(&message).unwrap();

        let emoji = captures.get(1).map(|m| m.as_str().trim().to_string());
        let scope = captures.get(2).map(|m| m.as_str().trim().to_string());
        let message = captures
            .get(3)
            .map(|m| m.as_str().trim().to_string())
            .unwrap();

        Self {
            emoji,
            scope,
            message,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn commit_new_nothing() {
//         let commit = crate::git::log::Commit::new("Pin anyhow crate version");
//         assert_eq!(commit.emoji, None);
//         assert_eq!(commit.scope, None);
//         assert_eq!(commit.message, "Pin anyhow crate version");
//     }

//     #[test]
//     fn commit_new_emoji() {
//         let commit = crate::git::log::Commit::new("📌 Pin anyhow crate version");
//         assert_eq!(commit.emoji, Some("📌".to_string()));
//         assert_eq!(commit.scope, None);
//         assert_eq!(commit.message, "📌 Pin anyhow crate version");
//     }

//     #[test]
//     fn commit_new_scope() {
//         let commit = crate::git::log::Commit::new("(other): Version bump");
//         assert_eq!(commit.emoji, None);
//         assert_eq!(commit.scope, Some("other".to_string()));
//         assert_eq!(commit.message, "🔖 (other): Version bump");
//     }

//     #[test]
//     fn commit_new_emoji_scope() {
//         let commit = crate::git::log::Commit::new("⚰️ (cli): Remove dead code");
//         assert_eq!(commit.emoji, Some("⚰️".to_string()));
//         assert_eq!(commit.scope, Some("cli".to_string()));
//         assert_eq!(commit.message, "⚰️ (cli): Remove dead code");
//     }

//     #[test]
//     fn commit_new_emoji_scope_no_space() {
//         let commit = crate::git::log::Commit::new("⚰️(cli): Remove dead code");
//         assert_eq!(commit.emoji, Some("⚰️".to_string()));
//         assert_eq!(commit.scope, Some("cli".to_string()));
//         assert_eq!(commit.message, "⚰️(cli): Remove dead code");
//     }

//     #[test]
//     fn commit_new_emoji_scope_no_colon() {
//         let commit = crate::git::log::Commit::new("⚰️(cli) Remove dead code");
//         assert_eq!(commit.emoji, Some("⚰️".to_string()));
//         assert_eq!(commit.scope, Some("cli".to_string()));
//         assert_eq!(commit.message, "⚰️(cli) Remove dead code");
//     }

//     #[test]
//     fn commit_new_emoji_scope_no_parenthesis() {
//         let commit = crate::git::log::Commit::new("⚰️ cli: Remove dead code");
//         assert_eq!(commit.emoji, Some("⚰️".to_string()));
//         assert_eq!(commit.scope, None);
//         assert_eq!(commit.message, "⚰️ cli: Remove dead code");
//     }

//     #[test]
//     fn commit_new_emoji_scope_no_emoji() {
//         let commit = crate::git::log::Commit::new("(cli): Remove dead code");
//         assert_eq!(commit.emoji, None);
//         assert_eq!(commit.scope, Some("cli".to_string()));
//         assert_eq!(commit.message, "(cli): Remove dead code");
//     }
// }
