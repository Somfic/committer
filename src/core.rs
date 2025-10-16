use gix::{
    bstr::ByteSlice,
    object::tree::diff::change,
    status::plumbing::index_as_worktree::{Change, Conflict, EntryStatus},
    Repository,
};
use nerd_font_symbols::{
    cod::{COD_DIFF, COD_DIFF_IGNORED, COD_DIFF_MODIFIED, COD_DIFF_REMOVED, COD_WARNING},
    md::{MD_FILE_HIDDEN, MD_VECTOR_DIFFERENCE},
    oct::OCT_DIFF_REMOVED,
};
use owo_colors::OwoColorize;

use crate::prelude::*;
use std::{fmt::Display, path::PathBuf};

pub fn run(cwd: &PathBuf) -> Result<()> {
    let repo = gix::discover(cwd)?;

    let status = repo.status(gix::progress::Discard)?;
    // .untracked_files(gix::status::UntrackedFiles::None);

    let patterns: Vec<gix::bstr::BString> = vec![];
    let changes: Vec<_> = status.into_iter(patterns)?.flatten().collect();

    for change in &changes {
        match &change {
            gix::status::Item::IndexWorktree(item) => match &item {
                gix::status::index_worktree::Item::Modification {
                    entry,
                    entry_index,
                    rela_path,
                    status,
                } => {
                    // println!("{:?}", change);
                }
                gix::status::index_worktree::Item::DirectoryContents {
                    entry,
                    collapsed_directory_status,
                } => {
                    // entry.status == entry::Status::Untracked;
                }
                gix::status::index_worktree::Item::Rewrite {
                    source,
                    dirwalk_entry,
                    dirwalk_entry_collapsed_directory_status,
                    dirwalk_entry_id,
                    diff,
                    copy,
                } => {
                    // println!("{:?}", change);
                }
            },
            gix::status::Item::TreeIndex(change_ref) => todo!(),
        };
    }

    // print all untracked files
    let mut untracked_entries: Vec<_> = changes
        .iter()
        .filter_map(|change| match change {
            gix::status::Item::IndexWorktree(
                gix::status::index_worktree::Item::DirectoryContents { entry, .. },
            ) if entry.status == gix::dir::entry::Status::Untracked => Some(entry),
            _ => None,
        })
        .collect();

    untracked_entries.sort_by(|a, b| a.rela_path.cmp(&b.rela_path));

    let mut tracked_entries: Vec<_> = changes
        .iter()
        .filter_map(|change| match change {
            gix::status::Item::IndexWorktree(gix::status::index_worktree::Item::Modification {
                entry,
                status,
                rela_path,
                ..
            }) => Some((entry, status, rela_path)),
            gix::status::Item::IndexWorktree(gix::status::index_worktree::Item::Rewrite {
                dirwalk_entry,
                source,
                ..
            }) => {
                todo!()
            }
            _ => None,
        })
        .collect();

    tracked_entries.sort_by(|a, b| a.2.cmp(b.2));

    println!("{}", "Tracked".bold().bright_green());
    for entry in &tracked_entries {
        match entry.1 {
            EntryStatus::Conflict(conflict) => println!(
                "{} {} ({})",
                MD_VECTOR_DIFFERENCE.bright_yellow(),
                entry.2.to_str_lossy().bright_yellow(),
                conflict.as_str().bright_yellow()
            ),
            EntryStatus::Change(change) => match change {
                Change::Removed => println!(
                    "{} {}",
                    OCT_DIFF_REMOVED.bright_red(),
                    entry.2.to_str_lossy().bright_red()
                ),
                Change::Type { .. } => println!(
                    "{} {}",
                    COD_DIFF.bright_magenta(),
                    entry.2.to_str_lossy().bright_magenta()
                ),
                Change::Modification { .. } => println!(
                    "{} {}",
                    COD_DIFF_MODIFIED.bright_yellow(),
                    entry.2.to_str_lossy().bright_yellow()
                ),
                Change::SubmoduleModification(_) => {
                    println!(
                        "{} {}",
                        COD_WARNING.bright_yellow(),
                        entry.2.to_str_lossy().bright_yellow()
                    )
                }
            },
            EntryStatus::NeedsUpdate(stat) => todo!(),
            EntryStatus::IntentToAdd => todo!(),
        }
    }

    println!("{}", "Untracked".bold().bright_black());
    for entry in untracked_entries {
        println!(
            "{} {}",
            COD_DIFF_IGNORED.bright_black(),
            entry.rela_path.to_str_lossy().bright_black()
        );
    }

    Ok(())
}

fn stage(repo: &Repository) {}

trait AsStr {
    fn as_str(&self) -> &'static str;
}

impl AsStr for Conflict {
    fn as_str(&self) -> &'static str {
        match self {
            Conflict::BothModified => "both modified",
            Conflict::AddedByUs => "added by us",
            Conflict::AddedByThem => "added by them",
            Conflict::DeletedByUs => "deleted by us",
            Conflict::DeletedByThem => "deleted by them",
            Conflict::BothAdded => "both added",
            Conflict::BothDeleted => "both deleted",
        }
    }
}
