use git2::{Repository, StatusOptions};
use nerd_font_symbols::md;
use owo_colors::OwoColorize;

use crate::prelude::*;
use std::{fmt::Display, path::PathBuf};

pub fn run(cwd: &PathBuf) -> Result<()> {
    let repo = Repository::open(cwd)?;

    let status = Statuses::query(&repo)?;
    println!("{}", status);

    Ok(())
}

pub struct Statuses(Vec<Status>);
impl Statuses {
    pub fn is_clean(&self) -> bool {
        self.0.is_empty()
    }

    pub fn files(&self) -> &[Status] {
        &self.0
    }
}

pub struct Status {
    pub path: PathBuf,
    pub staged: Option<StatusChange>,
    pub unstaged: Option<StatusChange>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum StatusChange {
    Modified,
    Added,
    Deleted,
    Renamed,
    TypeChange,
}

impl Statuses {
    pub fn query(repo: &Repository) -> Result<Self> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);
        opts.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut opts))?;

        let mut result = Vec::new();

        for entry in statuses.iter() {
            let path = PathBuf::from(entry.path().unwrap_or(""));
            let status = entry.status();

            let staged = if status.is_index_new() {
                Some(StatusChange::Added)
            } else if status.is_index_modified() {
                Some(StatusChange::Modified)
            } else if status.is_index_deleted() {
                Some(StatusChange::Deleted)
            } else if status.is_index_renamed() {
                Some(StatusChange::Renamed)
            } else if status.is_index_typechange() {
                Some(StatusChange::TypeChange)
            } else {
                None
            };

            let unstaged = if status.is_wt_new() {
                Some(StatusChange::Added)
            } else if status.is_wt_modified() {
                Some(StatusChange::Modified)
            } else if status.is_wt_deleted() {
                Some(StatusChange::Deleted)
            } else if status.is_wt_renamed() {
                Some(StatusChange::Renamed)
            } else if status.is_wt_typechange() {
                Some(StatusChange::TypeChange)
            } else {
                None
            };

            result.push(Status {
                path,
                staged,
                unstaged,
            });
        }

        Ok(Statuses(result))
    }
}

impl Display for StatusChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusChange::Modified => write!(f, "{} ", md::MD_FILE_DOCUMENT_EDIT.bright_yellow()),
            StatusChange::Added => write!(f, "{} ", md::MD_FILE_PLUS.bright_green()),
            StatusChange::Deleted => write!(f, "{} ", md::MD_FILE_REMOVE.bright_red()),
            StatusChange::Renamed => write!(f, "{} ", md::MD_FILE_MOVE.bright_blue()),
            StatusChange::TypeChange => write!(f, "{} ", md::MD_FILE_SWAP.bright_magenta()),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.staged, &self.unstaged) {
            (Some(staged), Some(_)) => {
                write!(
                    f,
                    "{} {} {}",
                    md::MD_MINUS_BOX.bright_yellow(),
                    staged,
                    self.path.display()
                )
            }
            (Some(staged), None) => {
                write!(
                    f,
                    "{} {} {}",
                    md::MD_CHECKBOX_MARKED.bright_green(),
                    staged,
                    self.path.display()
                )
            }
            (None, Some(unstaged)) => {
                write!(
                    f,
                    "{} {} {}",
                    md::MD_CHECKBOX_BLANK_OUTLINE.bright_black(),
                    unstaged,
                    self.path.display()
                )
            }
            (None, None) => {
                write!(
                    f,
                    "{} {} {}",
                    md::MD_CHECKBOX_BLANK_OUTLINE.bright_black(),
                    "  ",
                    self.path.display()
                )
            }
        }
    }
}

impl Display for Statuses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_clean() {
            writeln!(f, " {}", "Working directory clean".bright_green())?;
            return Ok(());
        }

        writeln!(f)?;
        for status in self.files() {
            writeln!(f, " {}", status)?;
        }

        Ok(())
    }
}
