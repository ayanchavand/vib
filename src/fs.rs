//! Filesystem abstractions, directory listing, and file entry classification.

use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Classification category for file entries based on file extension and metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// Directory entry.
    Directory,
    /// Human-readable text or source code file.
    Text,
    /// Raster or vector image format.
    Image,
    /// Audio stream or track.
    Audio,
    /// Video or multimedia container.
    Video,
    /// Compiled binary, executable, or compressed archive.
    Binary,
    /// Unclassified file type.
    Unknown,
}

/// Information about a single file system entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    /// File or directory name.
    pub name: String,
    /// Full system path.
    pub path: PathBuf,
    /// Entry classification kind.
    pub kind: EntryKind,
    /// Optional file size in bytes (absent for directories).
    pub size: Option<u64>,
    /// Last modification timestamp.
    pub modified: Option<SystemTime>,
}

/// Classifies a file path into an `EntryKind` according to extension and file type.
fn classify_entry(path: &Path, metadata: &Metadata) -> EntryKind {
    if metadata.is_dir() {
        return EntryKind::Directory;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some(
            "rs" | "txt" | "md" | "json" | "toml" | "yaml" | "yml" | "c" | "cpp" | "h" | "py"
            | "js" | "ts" | "html" | "css" | "sh" | "bash" | "zsh" | "go" | "java" | "kt" | "xml"
            | "ini" | "log" | "csv",
        ) => EntryKind::Text,

        Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff") => {
            EntryKind::Image
        }

        Some("mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a") => EntryKind::Audio,

        Some("mp4" | "mkv" | "webm" | "avi" | "mov" | "wmv") => EntryKind::Video,

        Some(
            "bin" | "exe" | "elf" | "so" | "dll" | "dylib" | "iso" | "zip" | "tar" | "gz" | "7z",
        ) => EntryKind::Binary,

        _ => EntryKind::Unknown,
    }
}

/// Reads the contents of directory at `path` and returns sorted `FileEntry` list.
/// Directories are listed first, sorted alphabetically, followed by files sorted alphabetically.
pub fn list_dir(path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        let kind = classify_entry(&path, &metadata);

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path,
            kind,
            size: if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            },
            modified: metadata.modified().ok(),
        });
    }

    entries.sort_by(|a, b| match (a.kind, b.kind) {
        (EntryKind::Directory, EntryKind::Directory) => a.name.cmp(&b.name),
        (EntryKind::Directory, _) => std::cmp::Ordering::Less,
        (_, EntryKind::Directory) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_entry() {
        let temp_file = std::env::temp_dir().join("test.rs");
        let _ = std::fs::write(&temp_file, "fn main() {}");
        if let Ok(meta) = std::fs::metadata(&temp_file) {
            assert_eq!(classify_entry(&temp_file, &meta), EntryKind::Text);
        }
        let _ = std::fs::remove_file(&temp_file);
    }
}
