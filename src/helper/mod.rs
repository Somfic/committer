use std::io::Write;

use crate::git::log::{Commit, SemVerBump};
use anyhow::Ok;

pub struct ReleaseInfo {
    pub version: semver::Version,
    pub changelog: String,
}

pub fn calculate_new_tag_based_on_commits() -> anyhow::Result<Option<ReleaseInfo>> {
    let latest_tag = crate::git::tag::latest()?;

    let latest_tag_clean = latest_tag.replace('v', "");
    let latest_tag_clean = if latest_tag_clean.split('.').count() == 2 {
        format!("{}.0", latest_tag_clean)
    } else {
        latest_tag_clean
    };

    let latest_version = semver::Version::parse(&latest_tag_clean)?;

    let commits = crate::git::log::commits_since(&latest_tag)?;

    if commits.is_empty() {
        return Ok(None);
    }

    // Determine the highest bump level and collect commits by type
    let mut bump: Option<SemVerBump> = None;
    let mut breaking: Vec<&Commit> = Vec::new();
    let mut features: Vec<&Commit> = Vec::new();
    let mut fixes: Vec<&Commit> = Vec::new();

    for commit in &commits {
        match &commit.semver {
            Some(SemVerBump::Major) => {
                bump = Some(SemVerBump::Major);
                breaking.push(commit);
            }
            Some(SemVerBump::Minor) => {
                if bump != Some(SemVerBump::Major) {
                    bump = Some(SemVerBump::Minor);
                }
                features.push(commit);
            }
            Some(SemVerBump::Patch) => {
                if bump.is_none() {
                    bump = Some(SemVerBump::Patch);
                }
                fixes.push(commit);
            }
            None => {}
        }
    }

    let bump = match bump {
        Some(b) => b,
        None => return Ok(None),
    };

    // Calculate new version
    let mut major = latest_version.major;
    let mut minor = latest_version.minor;
    let mut patch = latest_version.patch;

    match bump {
        SemVerBump::Major => {
            major += 1;
            minor = 0;
            patch = 0;
        }
        SemVerBump::Minor => {
            minor += 1;
            patch = 0;
        }
        SemVerBump::Patch => {
            patch += 1;
        }
    }

    let new_version = semver::Version::new(major, minor, patch);

    if new_version == latest_version {
        return Ok(None);
    }

    // Generate changelog matching workflow format
    let mut changelog = String::new();

    if !breaking.is_empty() {
        changelog.push_str("### Breaking changes\n");
        for commit in &breaking {
            changelog.push_str(&format!("- {}\n", format_commit(commit)));
        }
        changelog.push('\n');
    }

    if !features.is_empty() {
        changelog.push_str("### New features\n");
        for commit in &features {
            changelog.push_str(&format!("- {}\n", format_commit(commit)));
        }
        changelog.push('\n');
    }

    if !fixes.is_empty() {
        changelog.push_str("### Fixes\n");
        for commit in &fixes {
            changelog.push_str(&format!("- {}\n", format_commit(commit)));
        }
        changelog.push('\n');
    }

    set_github_env_var("COMMITTER_CHANGELOG", &changelog)?;

    Ok(Some(ReleaseInfo {
        version: new_version,
        changelog,
    }))
}

fn format_commit(commit: &Commit) -> String {
    let emoji = commit
        .emoji
        .as_ref()
        .map(|e| format!("{} ", e))
        .unwrap_or_default();

    let scope = commit
        .scope
        .as_ref()
        .map(|s| format!("({}): ", s))
        .unwrap_or_default();

    format!("{}{}{}", emoji, scope, commit.message)
}

pub fn set_github_env_var(name: &str, value: &str) -> anyhow::Result<()> {
    println!("Writing {} to .env/{}", value, name);

    std::fs::create_dir_all(".env")?;
    let mut file = std::fs::File::create(format!(".env/{}", name))?;
    file.write_all(value.as_bytes())?;

    Ok(())
}
