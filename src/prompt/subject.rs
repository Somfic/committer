use crate::emoji::{Emoji, SemVer};
use anyhow::Result;
use inquire::{autocompletion::Replacement, validator::ValueRequiredValidator, Autocomplete};
use std::collections::HashSet;

pub fn prompt(
    intention: &Emoji,
    previous_subjects: Vec<String>,
    default: Option<String>,
) -> anyhow::Result<String> {
    let description = match intention.semver {
        Some(SemVer::Major) => "Describe the breaking change",
        Some(SemVer::Minor) => "Describe the new feature",
        Some(SemVer::Patch) => "Describe the patch",
        None => "Describe the chore",
    };

    let autocomplete = CommitSubjectCompleter::new(previous_subjects);

    let text = inquire::Text::new("Subject:")
        .with_help_message("Describe the commit in one line")
        .with_placeholder(description)
        .with_autocomplete(autocomplete)
        .with_validator(ValueRequiredValidator::default());

    let text = if let Some(ref default_value) = default {
        text.with_default(default_value)
    } else {
        text
    };

    let result = text.prompt();

    result.map_err(|e| anyhow::Error::new(e))
}

#[derive(Clone)]
struct CommitSubjectCompleter {
    previous_subjects: Vec<String>,
}

impl CommitSubjectCompleter {
    pub fn new(previous_subjects: Vec<String>) -> Self {
        let unique_subjects: HashSet<String> = previous_subjects.into_iter().collect();

        Self {
            previous_subjects: unique_subjects.into_iter().collect(),
        }
    }
}

impl Autocomplete for CommitSubjectCompleter {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::prelude::v1::Result<Vec<String>, inquire::CustomUserError> {
        let suggestions = self
            .previous_subjects
            .iter()
            .filter(|subject| subject.contains(input))
            .map(|subject| subject.to_string())
            .collect();

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> std::prelude::v1::Result<Replacement, inquire::CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            return Ok(Replacement::Some(suggestion));
        }

        Ok(Replacement::None)
    }
}
