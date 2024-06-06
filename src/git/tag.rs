use std::fmt::format;

use semver::Version;

use crate::cmd::execute;

pub fn latest() -> anyhow::Result<String> {
    // Get the latest tag from the remote repository
    let output = execute(
        "git",
        vec!["--no-pager", "ls-remote", "--tags", "--sort=-v:refname"],
    )?;
    let tag = output
        .lines()
        .next()
        .unwrap()
        .split('\t')
        .skip(1)
        .next()
        .unwrap()
        .split("refs/tags/")
        .skip(1)
        .next()
        .unwrap()
        .to_string();

    Ok(tag)
}

pub fn tag(tag: String) -> anyhow::Result<()> {
    execute("git", vec!["tag", &format!("v{}", tag)])?;
    execute("git", vec!["push", "origin", &format!("v{}", tag)])?;

    Ok(())
}
