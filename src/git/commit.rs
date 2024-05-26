use crate::cmd::execute;
use anyhow::Result;

pub fn commit(message: String) -> Result<String> {
    execute("git", vec!["commit", "-m", &message])
}
