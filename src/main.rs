use anyhow::Ok;
use emoji::{Emoji, SemVer};
use git::status::Status;
use helper::set_github_env_var;
use std::collections::HashSet;

pub mod cmd;
pub mod emoji;
pub mod git;
pub mod helper;
pub mod prompt;
pub mod updater;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    if args.contains(&"tag".to_string()) {
        tag()?;
    } else {
        commit()?;
    }

    Ok(())
}

fn tag() -> anyhow::Result<()> {
    if let Some(tag) = crate::helper::calculate_new_tag_based_on_commits()? {
        crate::updater::cargo::set_version(&tag)?;
        set_github_env_var("COMMITTER_TAG", &tag.to_string())?;
        crate::git::tag::tag(tag.to_string())?;
        set_github_env_var("COMMITTER_IS_NEW", "true")?;
        println!("New version tagged as {}.", tag);
    } else {
        set_github_env_var("COMMITTER_TAG", "")?;
        set_github_env_var("COMMITTER_IS_NEW", "false")?;
        println!("No new version to tag.");
    }

    Ok(())
}

fn commit() -> anyhow::Result<()> {
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

    // TODO: Add autocomplete with previously used commit subjects
    let subject = crate::prompt::subject::prompt(
        &intention,
        log.iter().map(|m| m.message.clone()).collect(),
    )?;

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
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n");

    crate::git::commit::commit(message)?;

    Ok(())
}
