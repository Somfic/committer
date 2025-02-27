use std::fmt::{Display, Formatter};

use serde::Deserialize;

const EMOJI_JSON: &str = include_str!("emojis.json");

#[derive(Deserialize)]
pub struct Emoji {
    pub emoji: String,
    pub entity: String,
    pub code: String,
    pub description: String,
    pub name: String,
    pub semver: Option<SemVer>,
}

impl Emoji {
    pub fn all() -> Vec<Emoji> {
        serde_json::from_str(EMOJI_JSON).unwrap()
    }
}

impl Display for Emoji {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji, self.description)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SemVer {
    Major,
    Minor,
    Patch,
}
