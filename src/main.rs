use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

use anyhow::anyhow;
use inquire::Autocomplete;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum SemVer {
    Major,
    Minor,
    Patch,
}

#[derive(Deserialize)]
struct Emoji {
    pub emoji: String,
    pub entity: String,
    pub code: String,
    pub description: String,
    pub name: String,
    pub semver: Option<SemVer>,
}

#[derive(Debug)]
enum ChangeKind {
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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

struct Change {
    pub kind: ChangeKind,
    pub path: String,
}

impl Display for Emoji {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.emoji, self.description)
    }
}

fn main() -> anyhow::Result<()> {
    let branch = execute_cmd("git branch")?;
    let unstaged_diff = find_diff(false);
    let staged_diff = find_diff(true);
    let status: String = status()
        .lines()
        .map(|l| format!("# {}", l))
        .collect::<Vec<String>>()
        .join("\n");

    if staged_diff.is_empty() && unstaged_diff.is_empty() {
        println!("Working directory clean. Nothing to commit.");
        return Ok(());
    }

    if staged_diff.is_empty() {
        unstaged_diff.iter().for_each(|change| {
            println!("{} {}", change.kind, change.path);
        });

        println!("No changes added to commit.");
        return Ok(());
    }

    println!("Changes to be committed:");
    staged_diff.iter().for_each(|change| {
        println!("{} {}", change.kind, change.path);
    });

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
        .with_validator(NonEmptyValidator)
        .prompt()?;

    let mut scope = inquire::Text::new("Scope:")
        .with_help_message("What is the scope of the commit?")
        .with_autocomplete(commit_scope_completer)
        .with_placeholder("No scope")
        .with_initial_value(scopes.clone().iter().last().unwrap_or(&"".to_string()))
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

    let message = &format!("{}\n\n{}\n\n{}", subject, status, semver);

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

    println!("Executing command: {}", command);

    let result = execute_cmd(command)?;

    Ok(())
}

#[derive(Clone, Default)]
pub struct NonEmptyValidator;

impl inquire::validator::StringValidator for NonEmptyValidator {
    fn validate(
        &self,
        input: &str,
    ) -> Result<inquire::validator::Validation, inquire::CustomUserError> {
        if input.is_empty() {
            Ok(inquire::validator::Validation::Invalid(
                anyhow!("Input cannot be empty").into(),
            ))
        } else {
            Ok(inquire::validator::Validation::Valid)
        }
    }
}

#[derive(Clone, Default)]
pub struct CommitScopeCompleter {
    scopes: HashSet<String>,
}

impl CommitScopeCompleter {
    pub fn new(scopes: HashSet<String>) -> Self {
        Self { scopes }
    }
}

impl Autocomplete for CommitScopeCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let suggestions = self
            .scopes
            .iter()
            .filter(|scope| scope.contains(input))
            .map(|scope| scope.to_string())
            .collect();

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        let completion = highlighted_suggestion.unwrap_or_else(|| input.to_string());

        Ok(Some(completion))
    }
}

fn status() -> String {
    execute_cmd("git remote update").unwrap();
    execute_cmd("git --no-pager status").unwrap()
}

fn find_diff(staged_only: bool) -> Vec<Change> {
    let diff = if staged_only {
        execute_cmd("git --no-pager diff --cached --name-status").unwrap()
    } else {
        execute_cmd("git --no-pager diff --name-status").unwrap()
    };

    diff.lines()
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
        .collect()
}

fn execute_cmd(cmd: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;

    if output.status.success() {
        Ok(std::str::from_utf8(&output.stdout)?.into())
    } else {
        let stderr = std::str::from_utf8(&output.stderr)?.to_string();
        Err(anyhow!(stderr))
    }
}
