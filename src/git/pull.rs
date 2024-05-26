use crate::cmd::execute;
use anyhow::Result;

pub fn pull() -> Result<String> {
    execute("git pull --ff-only")
}
