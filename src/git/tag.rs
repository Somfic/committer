use std::fmt::format;

use semver::Version;

use crate::cmd::execute;

pub fn latest() -> anyhow::Result<String> {
    let result = execute("git", vec!["describe", "--tags", "--abbrev=0"])?;

    Ok(result.trim().to_string())
}

pub fn tag(tag: String) -> anyhow::Result<()> {
    execute("git", vec!["tag", &tag])?;
    execute("git", vec!["push", "origin", &tag])?;

    println!("New version tagged as {}", tag);

    Ok(())
}
