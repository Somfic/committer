use anyhow::anyhow;
use anyhow::Result;
use inquire::autocompletion::Replacement;
use inquire::Autocomplete;
use std::collections::HashSet;

pub fn prompt(scopes: HashSet<String>) -> Result<String> {
    let commit_scope_completer = CommitScopeCompleter::new(scopes);

    inquire::Text::new("Scope:")
        .with_help_message("What is the scope of the commit?")
        .with_autocomplete(commit_scope_completer)
        .with_placeholder("No scope")
        .prompt()
        .map_err(|e| anyhow!(e))
}

#[derive(Clone, Default)]
struct CommitScopeCompleter {
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
            .map(|scope| scope.to_string().to_lowercase())
            .collect();

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, inquire::CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            return Ok(Replacement::Some(suggestion));
        }

        // Fuzzy find the scope
        let highlighted_suggestion = self
            .scopes
            .iter()
            .filter(|scope| scope.contains(input))
            .map(|scope| scope.to_string())
            .next();

        if let Some(suggestion) = highlighted_suggestion {
            Ok(Replacement::Some(suggestion))
        } else {
            Ok(Replacement::None)
        }
    }
}
