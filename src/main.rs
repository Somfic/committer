use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

use anyhow::anyhow;
use inquire::{validator::ValueRequiredValidator, Autocomplete};
use regex::Regex;
use serde::Deserialize;

pub mod cmd;
pub mod emoji;
pub mod git;
pub mod helper;
pub mod prompt;

fn main() -> anyhow::Result<()> {
    let unstaged_diff = find_diff(false);
    let staged_diff = find_diff(true);
    let status: String = status();

    if status.contains("Your branch is behind") {
        if wants_pull {
            execute_cmd("git pull")?;
        }
    }

    if staged_diff.is_empty() && unstaged_diff.is_empty() {
        println!("Working directory clean. Nothing to commit.");
        return Ok(());
    }

    if staged_diff.is_empty() {
        unstaged_diff.iter().for_each(|change| {});

        println!("No changes added to commit. Stage changes first.");
        return Ok(());
    }

    let emojis: Vec<Emoji> = serde_json::from_str(include_str!("emojis.json")).unwrap();

    let scope_regex = regex::Regex::new(r"\s\((\w+)\):")?;

    let scopes: HashSet<String> =
        execute_cmd("git --no-pager log --decorate=short --pretty=oneline")?
            .lines()
            .filter_map(|line| scope_regex.captures(line))
            .filter_map(|capture| capture.get(1))
            .map(|m| m.as_str().to_string())
            .collect();

    let intention = inquire::Select::new("Intention:", emojis)
        .with_help_message("What is intention behind the commit?")
        .prompt()?;

    // Scan previous commits to find out all the scopes
    let commit_scope_completer = CommitScopeCompleter::new(scopes.clone());

    let description = match intention.semver {
        Some(SemVer::Major) => "Describe the breaking change",
        Some(SemVer::Minor) => "Describe the new feature",
        Some(SemVer::Patch) => "Describe the patch",
        None => "Describe the chore",
    };

    let subject = inquire::Text::new("Subject:")
        .with_help_message("Describe the commit in one line")
        .with_placeholder(description)
        .with_validator(ValueRequiredValidator::default())
        .prompt()?;

    // If not empty, add a space before the scope
    if !scope.is_empty() {
        scope = format!("({}): ", scope);
    }

    let subject = &format!("{} {}{}", intention.emoji, scope, subject);

    let semver = match intention.semver {
        Some(SemVer::Major) => "semver: major".to_string(),
        Some(SemVer::Minor) => "semver: minor".to_string(),
        Some(SemVer::Patch) => "semver: patch".to_string(),
        None => "semver: chore".to_string(),
    };

    let commented_status = status
        .lines()
        .map(|l| format!("# {}", l))
        .collect::<Vec<String>>()
        .join("\n");

    let message = &format!("{}\n\n{}\n\n{}", subject, commented_status, semver);

    let message = inquire::Editor::new(subject)
        .with_help_message("What is the body of the commit?")
        .with_predefined_text(message)
        .prompt()?;

    let message = message
        .lines()
        .filter(|line| !line.starts_with("#"))
        .collect::<Vec<&str>>()
        .join("\n");

    let command = &format!("git commit -m \"{}\"", message);

    let result = execute_cmd(command)?;

    Ok(())
}
