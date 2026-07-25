//! Helper functions for formatting bytes, file sizes, and time durations.

use std::time::SystemTime;

/// Formats a byte count into a human-readable string (B, KB, MB, GB).
pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{size} B")
    }
}

/// Formats a `SystemTime` instance into a human-readable relative time string.
pub fn format_modified_time(modified: SystemTime) -> Option<String> {
    let duration = modified.elapsed().ok()?;
    let secs = duration.as_secs();

    let s = if secs < 60 {
        format!("{secs} seconds ago")
    } else if secs < 3600 {
        format!("{} minutes ago", secs / 60)
    } else if secs < 86400 {
        format!("{} hours ago", secs / 3600)
    } else {
        format!("{} days ago", secs / 86400)
    };

    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1572864), "1.50 MB");
        assert_eq!(format_size(2147483648), "2.00 GB");
    }
}
