use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::Repository;
use nerd_font_symbols::md;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, path::PathBuf};

use crate::core::{Status, Statuses};
use crate::prelude::*;

pub struct App {
    repo: Repository,
    statuses: Statuses,
    list_state: ListState,
    diff_scroll: u16,
    commit_message: String,
    commit_message_cursor: usize, // Cursor position in bytes
    recent_commits: Vec<String>,
    cached_diff: Option<(usize, Vec<String>, bool)>, // (file_index, diff_lines, is_staged)
}

impl App {
    pub fn new(repo: Repository, statuses: Statuses) -> Self {
        let mut list_state = ListState::default();
        if !statuses.files().is_empty() {
            list_state.select(Some(0));
        }

        // Get recent commit messages
        let recent_commits = Self::get_recent_commits(&repo);

        Self {
            repo,
            statuses,
            list_state,
            diff_scroll: 0,
            commit_message: String::new(),
            commit_message_cursor: 0,
            recent_commits,
            cached_diff: None,
        }
    }

    fn get_recent_commits(repo: &Repository) -> Vec<String> {
        let mut commits = Vec::new();

        if let Ok(mut revwalk) = repo.revwalk() {
            if revwalk.push_head().is_ok() {
                for oid in revwalk.take(10) {
                    if let Ok(oid) = oid {
                        if let Ok(commit) = repo.find_commit(oid) {
                            if let Some(message) = commit.message() {
                                commits.push(message.to_string());
                            }
                        }
                    }
                }
            }
        }

        commits
    }

    fn selected_file(&self) -> Option<&Status> {
        self.list_state.selected().and_then(|i| self.statuses.files().get(i))
    }

    fn get_diff(&self, width: u16) -> Result<(Vec<String>, bool)> {
        use std::process::Command;
        use std::io::Write;

        let file = match self.selected_file() {
            Some(f) => f,
            None => return Ok((vec![], false)),
        };

        let is_staged = file.staged.is_some();

        let workdir = self.repo.workdir().ok_or_else(|| {
            git2::Error::from_str("Repository has no working directory")
        })?;

        // Get the old content from HEAD
        let old_content = self.repo.head().ok()
            .and_then(|head_ref| head_ref.peel_to_commit().ok())
            .and_then(|commit| commit.tree().ok())
            .and_then(|tree| tree.get_path(&file.path).ok())
            .and_then(|entry| entry.to_object(&self.repo).ok())
            .and_then(|object| object.as_blob().map(|b| b.content().to_vec()))
            .unwrap_or_default();

        // Get the new content from working tree
        let file_path = workdir.join(&file.path);
        let new_content = std::fs::read(&file_path).unwrap_or_default();

        // Create temporary files for difftastic with proper extensions
        let extension = file.path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");

        let mut old_file = tempfile::Builder::new()
            .suffix(&format!(".{}", extension))
            .tempfile()?;
        let mut new_file = tempfile::Builder::new()
            .suffix(&format!(".{}", extension))
            .tempfile()?;

        old_file.write_all(&old_content)?;
        new_file.write_all(&new_content)?;

        // Try difftastic with color and constrained width
        // Subtract 4 for borders (2 on each side)
        let display_width = width.saturating_sub(4).max(40);
        let output = Command::new("difft")
            .arg("--color=always")
            .arg("--display=inline")
            .arg("--syntax-highlight=on")
            .arg(format!("--width={}", display_width))
            .arg(old_file.path())
            .arg(new_file.path())
            .output();  

        match output {
            Ok(out) if out.status.success() => {
                let content = String::from_utf8_lossy(&out.stdout);
                // Keep the ANSI codes, ratatui will handle them
                Ok((content.lines().map(String::from).collect(), is_staged))
            }
            _ => {
                // Difftastic not available, show error
                Ok((vec![
                    "difftastic (difft) not found in PATH".to_string(),
                    "Install it with: cargo install difftastic".to_string(),
                    "".to_string(),
                    "Showing plain diff instead:".to_string(),
                    "".to_string(),
                    format!("--- {}", file.path.display()),
                    format!("+++ {}", file.path.display()),
                ], is_staged))
            }
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.statuses.files().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.diff_scroll = 0;
        self.cached_diff = None; // Invalidate cache on file change
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.statuses.files().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.diff_scroll = 0;
        self.cached_diff = None; // Invalidate cache on file change
    }

    fn scroll_diff_down(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_add(1);
    }

    fn scroll_diff_up(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_sub(1);
    }

    fn toggle_stage(&mut self) -> Result<()> {
        if let Some(file) = self.selected_file() {
            let path = file.path.clone();
            if file.staged.is_some() {
                unstage_file(&self.repo, &path)?;
            } else {
                stage_file(&self.repo, &path)?;
            }
            // Refresh statuses
            self.statuses = Statuses::query(&self.repo)?;
            self.cached_diff = None; // Invalidate cache on staging change
        }
        Ok(())
    }

    fn stage_all(&mut self) -> Result<()> {
        // Check if there are any unstaged files
        let has_unstaged = self.statuses.files().iter().any(|f| f.unstaged.is_some());

        if has_unstaged {
            // Stage all files
            for file in self.statuses.files() {
                stage_file(&self.repo, &file.path)?;
            }
        } else {
            // Everything is staged, unstage all
            for file in self.statuses.files() {
                if file.staged.is_some() {
                    unstage_file(&self.repo, &file.path)?;
                }
            }
        }

        // Refresh statuses
        self.statuses = Statuses::query(&self.repo)?;
        self.cached_diff = None; // Invalidate cache on staging change
        Ok(())
    }
}

fn stage_file(repo: &Repository, path: &PathBuf) -> Result<()> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    index.write()?;
    Ok(())
}

fn unstage_file(repo: &Repository, path: &PathBuf) -> Result<()> {
    // Reset the file to HEAD (unstage it)
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;
    let head_tree = head_commit.tree()?;

    let mut index = repo.index()?;

    // Try to get the entry from HEAD's tree
    let tree_entry = head_tree.get_path(path);

    match tree_entry {
        Ok(entry) => {
            // File exists in HEAD, restore it to the index
            index.add(&git2::IndexEntry {
                ctime: git2::IndexTime::new(0, 0),
                mtime: git2::IndexTime::new(0, 0),
                dev: 0,
                ino: 0,
                mode: entry.filemode() as u32,
                uid: 0,
                gid: 0,
                file_size: 0,
                id: entry.id(),
                flags: 0,
                flags_extended: 0,
                path: path.as_os_str().as_encoded_bytes().to_vec(),
            })?;
        }
        Err(_) => {
            // File doesn't exist in HEAD (it's a new file), remove it from index
            index.remove_path(path)?;
        }
    }

    index.write()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    // Split screen into left (50%) and right (50%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    // Split left side into file list (50%), commit message (25%), and recent commits (25%)
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(main_chunks[0]);

    // Left pane - file list
    let items: Vec<ListItem> = app
        .statuses
        .files()
        .iter()
        .map(|status| {
            let (checkbox_str, checkbox_color) = match (&status.staged, &status.unstaged) {
                (Some(_), Some(_)) => (md::MD_MINUS_BOX, Color::Yellow),
                (Some(_), None) => (md::MD_CHECKBOX_MARKED, Color::Green),
                (None, Some(_)) => (md::MD_CHECKBOX_BLANK_OUTLINE, Color::DarkGray),
                (None, None) => (md::MD_CHECKBOX_BLANK_OUTLINE, Color::DarkGray),
            };

            let (change_str, change_color) = match (&status.staged, &status.unstaged) {
                (Some(change), _) | (None, Some(change)) => {
                    match change {
                        crate::core::StatusChange::Modified => (md::MD_FILE_DOCUMENT_EDIT, Color::Yellow),
                        crate::core::StatusChange::Added => (md::MD_FILE_PLUS, Color::Green),
                        crate::core::StatusChange::Deleted => (md::MD_FILE_REMOVE, Color::Red),
                        crate::core::StatusChange::Renamed => (md::MD_FILE_MOVE, Color::Blue),
                        crate::core::StatusChange::TypeChange => (md::MD_FILE_SWAP, Color::Magenta),
                    }
                }
                (None, None) => (" ", Color::White),
            };

            let line = Line::from(vec![
                Span::styled(checkbox_str, Style::default().fg(checkbox_color)),
                Span::raw(" "),
                Span::styled(change_str, Style::default().fg(change_color)),
                Span::raw(" "),
                Span::raw(status.path.display().to_string()),
            ]);
            ListItem::new(line)
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files (Ctrl+w/s to navigate, Ctrl+space to toggle, Ctrl+a to stage/unstage all)"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, left_chunks[0], &mut app.list_state);

    // Commit message pane
    let cursor_visible = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() / 500) % 2 == 0;

    let commit_lines: Vec<Line> = {
        let before = &app.commit_message[..app.commit_message_cursor];
        let after = &app.commit_message[app.commit_message_cursor..];
        let full_text = format!("{}{}", before, after);

        let before_lines: Vec<&str> = before.lines().collect();
        let cursor_line_idx = if before.is_empty() { 0 } else { before_lines.len().saturating_sub(1) };
        let cursor_col = before_lines.last().map(|l| l.len()).unwrap_or(0);

        full_text
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let line_owned = line.to_string();
                if cursor_visible && i == cursor_line_idx {
                    if cursor_col < line_owned.len() {
                        // Cursor in middle of line
                        Line::from(vec![
                            Span::raw(line_owned[..cursor_col].to_string()),
                            Span::styled("_", Style::default().add_modifier(Modifier::UNDERLINED)),
                            Span::raw(line_owned[cursor_col + 1..].to_string()),
                        ])
                    } else {
                        // Cursor at end of line
                        Line::from(vec![
                            Span::raw(line_owned),
                            Span::styled("_", Style::default().add_modifier(Modifier::UNDERLINED)),
                        ])
                    }
                } else {
                    Line::from(line_owned)
                }
            })
            .collect()
    };
    let commit_paragraph = Paragraph::new(commit_lines)
        .block(Block::default().borders(Borders::ALL).title("Commit Message (Ctrl+q to quit)"))
        .wrap(Wrap { trim: false });

    f.render_widget(commit_paragraph, left_chunks[1]);

    // Recent commits pane
    let recent_commit_lines: Vec<Line> = app.recent_commits
        .iter()
        .enumerate()
        .map(|(i, msg)| {
            let first_line = msg.lines().next().unwrap_or("");

            Line::from(vec![
                Span::styled(format!("#{} ", i + 1), Style::default().fg(Color::DarkGray)),
                Span::raw(first_line.to_string()),
            ])
        })
        .collect();

    let recent_commits_paragraph = Paragraph::new(recent_commit_lines)
        .block(Block::default().borders(Borders::ALL).title("Recent Commits"))
        .wrap(Wrap { trim: false });

    f.render_widget(recent_commits_paragraph, left_chunks[2]);

    // Right pane - diff
    let current_index = app.list_state.selected().unwrap_or(0);

    // Check if we need to regenerate the diff
    let (diff_lines, is_staged) = if let Some((cached_index, ref cached_lines, cached_staged)) = app.cached_diff {
        if cached_index == current_index {
            (cached_lines.clone(), cached_staged)
        } else {
            let result = app.get_diff(main_chunks[1].width).unwrap_or_default();
            app.cached_diff = Some((current_index, result.0.clone(), result.1));
            result
        }
    } else {
        let result = app.get_diff(main_chunks[1].width).unwrap_or_default();
        app.cached_diff = Some((current_index, result.0.clone(), result.1));
        result
    };
    let diff_text: Vec<Line> = diff_lines
        .iter()
        .skip(app.diff_scroll as usize)
        .map(|line| {
            // Parse ANSI codes from difftastic output
            use ansi_to_tui::IntoText;
            let mut parsed_line = line.into_text()
                .unwrap_or_else(|_| ratatui::text::Text::raw(line.as_str()))
                .lines
                .into_iter()
                .next()
                .unwrap_or_else(|| Line::from(line.as_str()));

            // Convert to grayscale if not staged
            if !is_staged {
                parsed_line.spans = parsed_line.spans.into_iter().map(|span| {
                    let style = span.style;
                    let gray_style = if let Some(fg) = style.fg {
                        // Convert any color to a gray shade based on brightness
                        let gray = match fg {
                            Color::Red | Color::LightRed => Color::Gray,
                            Color::Green | Color::LightGreen => Color::Gray,
                            Color::Yellow | Color::LightYellow => Color::White,
                            Color::Blue | Color::LightBlue => Color::DarkGray,
                            Color::Magenta | Color::LightMagenta => Color::Gray,
                            Color::Cyan | Color::LightCyan => Color::Gray,
                            _ => Color::DarkGray,
                        };
                        Style::default().fg(gray).add_modifier(style.add_modifier)
                    } else {
                        Style::default().fg(Color::DarkGray).add_modifier(style.add_modifier)
                    };
                    Span::styled(span.content, gray_style)
                }).collect();
            }

            parsed_line
        })
        .collect();

    let diff = Paragraph::new(diff_text)
        .block(Block::default().borders(Borders::ALL).title("Diff (↑↓ to scroll)"))
        .wrap(Wrap { trim: false });

    f.render_widget(diff, main_chunks[1]);
}

pub fn run_ui(repo: Repository, statuses: Statuses) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(repo, statuses);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Poll for events with timeout to allow cursor blinking
        if event::poll(std::time::Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                    KeyCode::Down => app.scroll_diff_down(),
                    KeyCode::Up => app.scroll_diff_up(),
                    KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => app.next(),
                    KeyCode::Char('w') if key.modifiers.contains(event::KeyModifiers::CONTROL) => app.previous(),
                    KeyCode::Char(' ') if key.modifiers.contains(event::KeyModifiers::CONTROL) => app.toggle_stage()?,
                    KeyCode::Char('a') if key.modifiers.contains(event::KeyModifiers::CONTROL) => app.stage_all()?,
                    KeyCode::Left => {
                        // Move cursor left by one character
                        if app.commit_message_cursor > 0 {
                            let mut new_pos = app.commit_message_cursor - 1;
                            // Skip backwards over continuation bytes
                            while new_pos > 0 && !app.commit_message.is_char_boundary(new_pos) {
                                new_pos -= 1;
                            }
                            app.commit_message_cursor = new_pos;
                        }
                    }
                    KeyCode::Right => {
                        // Move cursor right by one character
                        if app.commit_message_cursor < app.commit_message.len() {
                            let mut new_pos = app.commit_message_cursor + 1;
                            // Skip forward over continuation bytes
                            while new_pos < app.commit_message.len() && !app.commit_message.is_char_boundary(new_pos) {
                                new_pos += 1;
                            }
                            app.commit_message_cursor = new_pos;
                        }
                    }
                    KeyCode::Backspace => {
                        if app.commit_message_cursor > 0 {
                            let mut del_pos = app.commit_message_cursor - 1;
                            // Skip backwards over continuation bytes
                            while del_pos > 0 && !app.commit_message.is_char_boundary(del_pos) {
                                del_pos -= 1;
                            }
                            app.commit_message.drain(del_pos..app.commit_message_cursor);
                            app.commit_message_cursor = del_pos;
                        }
                    }
                    KeyCode::Enter => {
                        app.commit_message.insert(app.commit_message_cursor, '\n');
                        app.commit_message_cursor += 1;
                    }
                    KeyCode::Char(c) => {
                        app.commit_message.insert(app.commit_message_cursor, c);
                        app.commit_message_cursor += c.len_utf8();
                    }
                    _ => {}
                }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
