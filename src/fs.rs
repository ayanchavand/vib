use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Directory,
    Text,
    Image,
    Audio,
    Video,
    Binary,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

fn classify_entry(path: &Path, metadata: &Metadata) -> EntryKind {
    if metadata.is_dir() {
        return EntryKind::Directory;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("rs") | Some("txt") | Some("md") | Some("json") | Some("toml") | Some("yaml")
        | Some("yml") | Some("c") | Some("cpp") | Some("h") | Some("py") | Some("js")
        | Some("ts") | Some("html") | Some("css") | Some("sh") | Some("bash") | Some("zsh")
        | Some("go") | Some("java") | Some("kt") | Some("xml") | Some("ini") | Some("log")
        | Some("csv") => EntryKind::Text,

        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp") | Some("svg")
        | Some("webp") | Some("ico") | Some("tiff") => EntryKind::Image,

        Some("mp3") | Some("wav") | Some("flac") | Some("aac") | Some("ogg") | Some("m4a") => {
            EntryKind::Audio
        }

        Some("mp4") | Some("mkv") | Some("webm") | Some("avi") | Some("mov") | Some("wmv") => {
            EntryKind::Video
        }

        Some("bin") | Some("exe") | Some("elf") | Some("so") | Some("dll") | Some("dylib")
        | Some("iso") | Some("zip") | Some("tar") | Some("gz") | Some("7z") => EntryKind::Binary,

        _ => EntryKind::Unknown,
    }
}

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
