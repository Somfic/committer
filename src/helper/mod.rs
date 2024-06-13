use std::io::Write;

use anyhow::Ok;

pub fn calculate_new_tag_based_on_commits() -> anyhow::Result<Option<semver::Version>> {
    let latest_tag = crate::git::tag::latest()?;

    // Clean up tag by adding patch version if it's missing
    let latest_tag_clean = if latest_tag.split('.').count() == 2 {
        format!("{}.0", latest_tag)
    } else {
        latest_tag.to_string()
    }
    .replace('v', "");

    let latest_version = semver::Version::parse(&latest_tag_clean)?;

    let patches_delta = crate::git::log::patches_since(&latest_tag.to_string())?;
    let minors_delta = crate::git::log::minors_since(&latest_tag.to_string())?;
    let majors_delta = crate::git::log::majors_since(&latest_tag.to_string())?;

    // String builder
    let mut changelog = String::new();

    if !majors_delta.is_empty() {
        changelog.push_str("## 🚨 Breaking changes");
        for scope in majors_delta
            .keys()
            .filter(|s| !majors_delta.get(*s).unwrap().is_empty())
        {
            let scope = scope
                .chars()
                .enumerate()
                .fold(String::new(), |mut acc, (i, c)| {
                    if i == 0 {
                        acc.push(c.to_uppercase().next().unwrap());
                    } else {
                        acc.push(c);
                    }
                    acc
                });
            changelog.push_str(&format!("\n### {}", scope));
            for commit in majors_delta.get(&scope).unwrap() {
                changelog.push_str(&format!(
                    "\n- {} {}",
                    commit.emoji.as_ref().unwrap_or(&"".to_string()),
                    commit.message
                ));
            }
        }
        changelog.push_str("\n\n");
    }

    if !minors_delta.is_empty() {
        changelog.push_str("## 🚀 New features");
        for scope in minors_delta
            .keys()
            .filter(|s| !minors_delta.get(*s).unwrap().is_empty())
        {
            // Push scope, make it start with an uppercase letter
            let scope = scope
                .chars()
                .enumerate()
                .fold(String::new(), |mut acc, (i, c)| {
                    if i == 0 {
                        acc.push(c.to_uppercase().next().unwrap());
                    } else {
                        acc.push(c);
                    }
                    acc
                });
            changelog.push_str(&format!("\n### {}", scope));
            for commit in minors_delta.get(&scope).unwrap() {
                changelog.push_str(&format!(
                    "\n- {} {}",
                    commit.emoji.as_ref().unwrap_or(&"".to_string()),
                    commit.message
                ));
            }
        }
        changelog.push_str("\n\n");
    }

    if !patches_delta.is_empty() {
        changelog.push_str("## 🐛 Bug fixes");
        for scope in patches_delta
            .keys()
            .filter(|s| !patches_delta.get(*s).unwrap().is_empty())
        {
            let scope = scope
                .chars()
                .enumerate()
                .fold(String::new(), |mut acc, (i, c)| {
                    if i == 0 {
                        acc.push(c.to_uppercase().next().unwrap());
                    } else {
                        acc.push(c);
                    }
                    acc
                });
            changelog.push_str(&format!("\n### {}", scope));
            for commit in patches_delta.get(&scope).unwrap() {
                changelog.push_str(&format!(
                    "\n- {} {}",
                    commit.emoji.as_ref().unwrap_or(&"".to_string()),
                    commit.message
                ));
            }
        }
        changelog.push_str("\n\n");
    }

    set_github_env_var("COMMITTER_CHANGELOG", &changelog)?;

    let mut patches = latest_version.patch;
    let mut minors = latest_version.minor;
    let mut majors = latest_version.major;

    if !majors_delta.is_empty() {
        majors += 1;
        minors = 0;
        patches = 0;
    } else if !minors_delta.is_empty() {
        minors += 1;
        patches = 0;
    } else if !patches_delta.is_empty() {
        patches += 1;
    }

    let new_version = semver::Version::new(majors, minors, patches);

    if new_version == latest_version {
        return Ok(None);
    }

    Ok(Some(new_version))
}

pub fn set_github_env_var(name: &str, value: &str) -> anyhow::Result<()> {
    println!("Writing {} to .env/{}", value, name);

    std::fs::create_dir_all(".env")?;
    let mut file = std::fs::File::create(format!(".env/{}", name))?;
    file.write_all(value.as_bytes())?;

    Ok(())
}
