use crate::cmd::execute;
use anyhow::Result;

pub fn push() -> Result<String> {
    execute("git", vec!["push"])
}
