//! Bookmarking system disk persistence and path management.

use std::path::{Path, PathBuf};

/// Resolves standard configuration path for stored bookmarks (`~/.config/vib/bookmarks.json`).
pub fn get_bookmark_storage_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("vib").join("bookmarks.json"))
}

/// Reads saved bookmarks list from JSON file at `path`.
pub fn load_bookmarks_from_disk(path: Option<&Path>) -> Vec<PathBuf> {
    if let Some(path) = path
        && path.exists()
        && let Ok(content) = std::fs::read_to_string(path)
        && let Ok(bookmarks) = serde_json::from_str::<Vec<PathBuf>>(&content)
    {
        return bookmarks;
    }
    Vec::new()
}

/// Writes `bookmarks` list to JSON file at `path`.
pub fn save_bookmarks_to_disk(bookmarks: &[PathBuf], path: Option<&Path>) {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(bookmarks) {
            let _ = std::fs::write(path, json);
        }
    }
}
