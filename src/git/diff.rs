use crate::cmd::execute;
use anyhow::Result;
use std::fmt::{Display, Formatter};

pub fn diff(staged_only: bool) -> Result<Vec<Change>> {
    let diff = if staged_only {
        execute(
            "git",
            vec!["--no-pager", "diff", "--cached", "--name-status"],
        )?
    } else {
        execute("git", vec!["--no-pager", "diff", "--name-status"])?
    };

    let changes = diff
        .lines()
        .map(|line| {
            let mut parts = line.split_whitespace();
            let kind = match parts.next() {
                Some("A") => ChangeKind::Added,
                Some("C") => ChangeKind::Copied,
                Some("D") => ChangeKind::Deleted,
                Some("M") => ChangeKind::Modified,
                Some("R") => ChangeKind::Renamed,
                Some("T") => ChangeKind::Changed,
                Some("U") => ChangeKind::Unmerged,
                Some("X") => ChangeKind::Unknown,
                Some("B") => ChangeKind::Broken,
                _ => ChangeKind::Unknown,
            };

            let path = parts.next().unwrap_or_default().to_string();

            Change { kind, path }
        })
        .collect();

    Ok(changes)
}

#[derive(Debug)]
pub struct Change {
    pub kind: ChangeKind,
    pub path: String,
    pub content: String,
}

#[derive(Debug)]
pub enum ChangeKind {
    Added,
    Copied,
    Deleted,
    Modified,
    Renamed,
    Changed,
    Unmerged,
    Unknown,
    Broken,
}

impl Display for ChangeKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ChangeKind::Added => write!(f, "A"),
            ChangeKind::Copied => write!(f, "C"),
            ChangeKind::Deleted => write!(f, "D"),
            ChangeKind::Modified => write!(f, "M"),
            ChangeKind::Renamed => write!(f, "R"),
            ChangeKind::Changed => write!(f, "T"),
            ChangeKind::Unmerged => write!(f, "U"),
            ChangeKind::Unknown => write!(f, "X"),
            ChangeKind::Broken => write!(f, "B"),
        }
    }
}
