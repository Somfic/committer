use crate::cmd::execute;

pub fn latest() -> anyhow::Result<String> {
    execute("git", vec!["fetch", "--tags"])?;

    // Find the latest stable (non-draft) semver tag, matching workflow logic
    let tags = execute("git", vec!["tag", "--sort=-v:refname"]).unwrap_or_default();

    let tag = tags
        .lines()
        .find(|line| {
            let re = regex::Regex::new(r"^v[0-9]+\.[0-9]+\.[0-9]+$").unwrap();
            re.is_match(line.trim())
        })
        .unwrap_or("v0.0.0")
        .trim()
        .to_string();

    Ok(tag)
}

pub fn tag(tag: String) -> anyhow::Result<()> {
    execute("git", vec!["tag", &tag])?;

    Ok(())
}
