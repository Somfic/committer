use crate::cmd::execute;

pub fn latest() -> anyhow::Result<String> {
    execute("git", vec!["fetch", "--tags"])?;

    let result = execute("git", vec!["describe", "--tags", "--abbrev=0"])?;

    Ok(result.trim().to_string())
}

pub fn tag(tag: String) -> anyhow::Result<()> {
    execute("git", vec!["tag", &tag])?;

    println!("New version tagged as {}", tag);

    Ok(())
}
