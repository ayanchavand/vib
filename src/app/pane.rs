//! Browsing pane state managing directory contents, selection, and text preview buffer.

use crate::error::AppError;
use crate::fs::{self, EntryKind, FileEntry};
use ratatui::widgets::ListState;
use std::path::PathBuf;

/// Represents a single file browser pane with path, listed entries, list selection state, and text preview.
#[derive(Clone, Debug)]
pub struct BrowsingPane {
    /// Currently displayed directory path.
    pub current_path: PathBuf,
    /// List of file entries in current directory.
    pub entries: Vec<FileEntry>,
    /// Index of currently highlighted entry.
    pub selected: usize,
    /// Ratatui list state for scroll/selection rendering.
    pub list_state: ListState,
    /// Path of currently loaded text preview file, if any.
    pub text_preview_path: Option<PathBuf>,
    /// Lines buffer of text preview file.
    pub text_preview_lines: Vec<String>,
    /// Scroll offset line count for text preview.
    pub text_preview_scroll: usize,
}

impl BrowsingPane {
    /// Creates a new `BrowsingPane` initialized to `path`.
    pub fn new(path: PathBuf) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            current_path: path,
            entries: Vec::new(),
            selected: 0,
            list_state,
            text_preview_path: None,
            text_preview_lines: Vec::new(),
            text_preview_scroll: 0,
        }
    }

    /// Loads directory entries from disk for `current_path` and updates selection/preview.
    pub fn load(&mut self) -> Result<(), AppError> {
        let entries = fs::list_dir(&self.current_path)?;
        self.entries = entries;

        if self.entries.is_empty() {
            self.selected = 0;
            self.list_state.select(None);
        } else {
            if self.selected >= self.entries.len() {
                self.selected = self.entries.len() - 1;
            }
            self.list_state.select(Some(self.selected));
        }

        self.update_text_preview();
        Ok(())
    }

    /// Returns a reference to the currently selected `FileEntry`, if available.
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }

    /// Updates text file preview buffer based on currently selected entry.
    pub fn update_text_preview(&mut self) {
        if let Some(entry) = self.selected_entry()
            && (entry.kind == EntryKind::Text || entry.kind == EntryKind::Unknown)
        {
            let target_path = entry.path.clone();
            if self.text_preview_path.as_ref() != Some(&target_path) {
                self.text_preview_path = Some(target_path.clone());
                self.text_preview_scroll = 0;
                self.text_preview_lines = match std::fs::read_to_string(&target_path) {
                    Ok(content) => content.lines().map(|s| s.to_string()).collect(),
                    Err(_) => vec!["[Unable to read text preview]".to_string()],
                };
            }
            return;
        }
        self.text_preview_path = None;
        self.text_preview_lines.clear();
        self.text_preview_scroll = 0;
    }
}
