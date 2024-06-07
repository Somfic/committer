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

    // Commit examples:
    // 📌 Pin anyhow crate version
    // 🔖 Version bump
    // ⚰️ (cli): Remove dead code
    // Hello world
    //
    // Only keep the message part, remove emojis and scopes
    let commit_regex = regex::Regex::new(r"^(?:\p{Emoji_Presentation}+\s)?(?:\((\w+)\):\s)?(.+)$")?;

    Ok(commits
        .lines()
        .filter(|line| !line.is_empty())
        // Remove emojis and scopes
        .flat_map(|line| {
            commit_regex
                .captures(line)
                .and_then(|c| c.get(2).map(|m| m.as_str().to_string()))
        })
        .map(|line| Commit { message: line })
        .collect())
}

pub fn majors_since(tag: &String) -> Result<Vec<Commit>> {
    since(tag, "semver: major")
}

pub fn minors_since(tag: &String) -> Result<Vec<Commit>> {
    since(tag, "semver: minor")
}

pub fn patches_since(tag: &String) -> Result<Vec<Commit>> {
    since(tag, "semver: patch")
}

fn since(tag: &String, grep: &str) -> Result<Vec<Commit>> {
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

    Ok(execute("git", args)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| Commit {
            message: line.to_string(),
        })
        .collect())
}

#[derive(Debug)]
pub struct Commit {
    pub message: String,
}
