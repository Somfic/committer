use anyhow::Ok;
use emoji::{Emoji, SemVer};
use genai::{
    chat::{ChatMessage, ChatRequest},
    Client,
};
use git::status::Status;
use helper::set_github_env_var;
use std::collections::HashSet;

pub mod cmd;
pub mod emoji;
pub mod git;
pub mod helper;
pub mod prompt;
pub mod updater;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    if crate::git::status::status().is_err() {
        println!("Not in a git repository.");
        return Ok(());
    }

    if args.contains(&"tag".to_string()) {
        tag()?;
    } else {
        commit().await?;
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

async fn commit() -> anyhow::Result<()> {
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

    let diff = crate::git::diff::diff_raw()?;

    let suggested_scope = suggest_scope(diff.clone(), &emojis)
        .await
        .map(Some)
        .unwrap_or(None);

    let intention_select = inquire::Select::new("Intention:", emojis)
        .with_help_message("What is intention behind the commit?");

    let intention = if let Some(suggested_scope) = suggested_scope {
        intention_select.with_starting_cursor(suggested_scope)
    } else {
        intention_select.with_starting_cursor(0)
    }
    .prompt()?;

    let suggested_message = suggest_message(diff.clone())
        .await
        .map(Some)
        .unwrap_or(None);

    // TODO: Add autocomplete with previously used commit subjects
    let subject = crate::prompt::subject::prompt(
        &intention,
        log.iter().map(|m| m.message.clone()).collect(),
        suggested_message,
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

    let wants_to_push = crate::prompt::push::prompt()?;
    if wants_to_push {
        crate::git::push::push()?;
    }

    Ok(())
}

async fn suggest_scope(diff: String, emojis: &[Emoji]) -> anyhow::Result<usize> {
    let chat_req = ChatRequest::new(vec![
        ChatMessage::system(format!("Given the following list of git scopes, suggest the most appropriate one based on the given git diff. Reply with the name of the scope only.
        The list of scopes has been provided in the format: <name>: <description> (<semver>).
        The list of scopes is as follows:

        {}
        
        Make sure to choose the most appropriate scope based on the git diff.
        ",
        emojis.iter().map(|e| format!("{}: {} ({:?})", e.name, e.description, e.semver)).collect::<Vec<String>>().join("\n"))),
        ChatMessage::user(&diff),
    ]);

    let client = Client::default();
    let chat_res = client
        .exec_chat("gpt-4o-mini", chat_req.clone(), None)
        .await?;

    let response = chat_res
        .content
        .and_then(|c| c.text_into_string())
        .ok_or(anyhow::anyhow!("No content in chat response"))?;

    Ok(emojis.iter().position(|e| e.name == response).unwrap_or(0))
}

async fn suggest_message(diff: String) -> anyhow::Result<String> {
    let chat_req = ChatRequest::new(vec![
        ChatMessage::system("Generate a commit message based on the given git diff. 
        Summarize the following git diff in one sentence, assuming the user is a developer and summarizing the changes in the diff. 
        Use the present tense and avoid using 'I' or 'we'. 
        Try to be as concise as possible and use a maximum of 50 characters.
        Use code tags to indicate functions, classes, and variables.
        Do not include any other text in the response.
        Do not include any emojis in the response.
        Do not end the response with a period.
        Do not start the response with a capital letter.
        "),
        ChatMessage::user(&diff),
    ]);

    let client = Client::default();
    let chat_res = client
        .exec_chat("gpt-4o-mini", chat_req.clone(), None)
        .await?;

    chat_res
        .content
        .and_then(|c| c.text_into_string())
        .ok_or(anyhow::anyhow!("No content in chat response"))
}
