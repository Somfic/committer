use crate::cmd::execute;

pub fn latest() -> anyhow::Result<String> {
    execute("git", vec!["fetch", "--tags"])?;

    let result =
        execute("git", vec!["describe", "--tags", "--abbrev=0"]).unwrap_or("0.0.0".to_owned());

    Ok(result.trim().to_string())
}

pub fn tag(tag: String) -> anyhow::Result<()> {
    execute("git", vec!["tag", &tag])?;

    Ok(())
}
