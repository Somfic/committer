use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

use anyhow::anyhow;
use emoji::{Emoji, SemVer};
use git::status::Status;
use inquire::{validator::ValueRequiredValidator, Autocomplete};
use regex::Regex;
use serde::Deserialize;

pub mod cmd;
pub mod emoji;
pub mod git;
pub mod helper;
pub mod prompt;

fn main() -> anyhow::Result<()> {
    let emojis = Emoji::all();

    let unstaged_diff = crate::git::diff::diff(false)?;
    let staged_diff = crate::git::diff::diff(true)?;

    if staged_diff.is_empty() && unstaged_diff.is_empty() {
        println!("Working directory is clean. Nothing to commit.");
        return Ok(());
    }

    if staged_diff.is_empty() {
        // TODO: Add support to stage files
        println!("No changes added to commit. Stage changes first.");
        return Ok(());
    }

    let status: Status = crate::git::status::status()?;
    let log = crate::git::log::log()?;
    let scope_regex = regex::Regex::new(r"\s\((\w+)\):")?;
    let scopes: HashSet<String> = log
        .iter()
        .map(|c| c.message.clone())
        .flat_map(|line| {
            scope_regex
                .captures(&line)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        })
        .collect();

    let intention = inquire::Select::new("Intention:", emojis)
        .with_help_message("What is intention behind the commit?")
        .prompt()?;

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

    let mut scope = crate::prompt::scope::prompt(scopes)?;

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
        .message
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

    crate::git::commit::commit(message)?;

    Ok(())
}
