use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

use anyhow::anyhow;
use inquire::Autocomplete;
use serde::Deserialize;

#[derive(Deserialize)]
struct Emoji {
    pub emoji: String,
    pub entity: String,
    pub code: String,
    pub description: String,
    pub name: String,
}

impl Display for Emoji {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.emoji, self.description)
    }
}

fn main() -> anyhow::Result<()> {
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

    let subject = inquire::Text::new("Subject:")
        .with_help_message("What is the subject of the commit?")
        .with_placeholder(&intention.description)
        .with_validator(NonEmptyValidator)
        .prompt()?;

    let scope = inquire::Text::new("Scope:")
        .with_help_message("What is the scope of the commit?")
        .with_autocomplete(commit_scope_completer)
        .with_default(scopes.clone().iter().last().unwrap_or(&"".to_string()))
        .prompt_skippable()?
        .map(|s| format!("({}): ", s))
        .unwrap_or_default();

    let message = &format!("{} {}{}", intention.emoji, scope, subject);

    let message = inquire::Editor::new(&message)
        .with_help_message("What is the body of the commit?")
        .with_predefined_text(message)
        .prompt()?;

    let result = execute_cmd(&format!("git commit -m '{}", message))?;

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
