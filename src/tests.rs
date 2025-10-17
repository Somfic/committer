use crate::core::{Statuses, StatusChange};

use git2::Repository;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn init_repo(path: &Path) -> Repository {
    Repository::init(path).unwrap()
}

fn create_change(repo: &Repository, file: &str, content: &str) {
    let repo_path = repo.workdir().unwrap();
    fs::write(repo_path.join(file), content).unwrap();
}

fn stage_file(repo: &Repository, file: &str) {
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(file)).unwrap();
    index.write().unwrap();
}

fn create_commit(repo: &Repository, files: &[(&str, &str)], message: &str) {
    // Write files
    for (file, content) in files {
        create_change(repo, file, content);
    }

    // Stage files
    for (file, _) in files {
        stage_file(repo, file);
    }

    // Create commit
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();

    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent_commit.as_ref().map(|c| vec![c]).unwrap_or_default();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .unwrap();
}

#[test]
fn test_status_with_unstaged_modified_file() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    let repo = init_repo(repo_path);
    create_commit(&repo, &[("file.txt", "hello")], "Initial commit");

    // Make a change (unstaged)
    create_change(&repo, "file.txt", "modified");

    let statuses = Statuses::query(&repo).unwrap();
    assert!(!statuses.is_clean());
    assert_eq!(statuses.files().len(), 1);
    assert_eq!(statuses.files()[0].staged, None);
    assert_eq!(statuses.files()[0].unstaged, Some(StatusChange::Modified));
    assert_eq!(statuses.files()[0].path.to_str().unwrap(), "file.txt");
}

#[test]
fn test_status_with_untracked_file() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    let repo = init_repo(repo_path);
    create_commit(&repo, &[("initial.txt", "initial")], "Initial commit");

    // Create untracked file
    create_change(&repo, "untracked.txt", "untracked");

    let statuses = Statuses::query(&repo).unwrap();
    assert!(!statuses.is_clean());
    assert_eq!(statuses.files().len(), 1);
    assert_eq!(statuses.files()[0].staged, None);
    assert_eq!(statuses.files()[0].unstaged, Some(StatusChange::Added));
    assert_eq!(statuses.files()[0].path.to_str().unwrap(), "untracked.txt");
}

#[test]
fn test_status_with_staged_new_file() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    let repo = init_repo(repo_path);
    create_commit(&repo, &[("initial.txt", "initial")], "Initial commit");

    // Create and stage a new file
    create_change(&repo, "staged.txt", "staged");
    stage_file(&repo, "staged.txt");

    let statuses = Statuses::query(&repo).unwrap();
    assert!(!statuses.is_clean());
    assert_eq!(statuses.files().len(), 1);
    assert_eq!(statuses.files()[0].staged, Some(StatusChange::Added));
    assert_eq!(statuses.files()[0].unstaged, None);
    assert_eq!(statuses.files()[0].path.to_str().unwrap(), "staged.txt");
}

#[test]
fn test_status_with_staged_modified_file() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    let repo = init_repo(repo_path);
    create_commit(&repo, &[("file.txt", "initial")], "Initial commit");

    // Modify and stage
    create_change(&repo, "file.txt", "modified");
    stage_file(&repo, "file.txt");

    let statuses = Statuses::query(&repo).unwrap();
    assert!(!statuses.is_clean());
    assert_eq!(statuses.files().len(), 1);
    assert_eq!(statuses.files()[0].staged, Some(StatusChange::Modified));
    assert_eq!(statuses.files()[0].unstaged, None);
    assert_eq!(statuses.files()[0].path.to_str().unwrap(), "file.txt");
}

#[test]
fn test_status_with_staged_and_unstaged_changes() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    let repo = init_repo(repo_path);
    create_commit(&repo, &[("file.txt", "initial")], "Initial commit");

    // Modify and stage
    create_change(&repo, "file.txt", "staged change");
    stage_file(&repo, "file.txt");

    // Modify again (unstaged)
    create_change(&repo, "file.txt", "unstaged change");

    let statuses = Statuses::query(&repo).unwrap();
    assert!(!statuses.is_clean());
    assert_eq!(statuses.files().len(), 1);
    assert_eq!(statuses.files()[0].staged, Some(StatusChange::Modified));
    assert_eq!(statuses.files()[0].unstaged, Some(StatusChange::Modified));
    assert_eq!(statuses.files()[0].path.to_str().unwrap(), "file.txt");
}

#[test]
fn test_status_clean_repo() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    let repo = init_repo(repo_path);
    create_commit(&repo, &[("initial.txt", "initial")], "Initial commit");

    let statuses = Statuses::query(&repo).unwrap();
    assert!(statuses.is_clean());
    assert_eq!(statuses.files().len(), 0);
}

#[test]
fn test_status_with_deleted_file() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    let repo = init_repo(repo_path);
    create_commit(&repo, &[("file.txt", "content")], "Initial commit");

    // Delete the file
    fs::remove_file(repo_path.join("file.txt")).unwrap();

    let statuses = Statuses::query(&repo).unwrap();
    assert!(!statuses.is_clean());
    assert_eq!(statuses.files().len(), 1);
    assert_eq!(statuses.files()[0].staged, None);
    assert_eq!(statuses.files()[0].unstaged, Some(StatusChange::Deleted));
    assert_eq!(statuses.files()[0].path.to_str().unwrap(), "file.txt");
}
