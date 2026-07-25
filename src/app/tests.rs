//! Unit tests for application state management, dual-pane operations, and transfers.

#[cfg(test)]
mod tests {
    use crate::app::AppState;
    use crate::app::modal::LocalSendModalState;
    use crate::events::AppEvent;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_bookmark_toggle_and_delete() {
        let download_dir = Arc::new(Mutex::new(PathBuf::from("/tmp")));
        let mut app = AppState::new(
            PathBuf::from("/tmp"),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );
        app.bookmark_path = None;

        let path1 = PathBuf::from("/tmp/dir1");
        let path2 = PathBuf::from("/tmp/dir2");

        app.bookmarks.clear();
        app.bookmarks.push(path1.clone());
        app.bookmarks.push(path2.clone());
        app.bookmark_selected = 0;
        app.bookmark_list_state.select(Some(0));

        assert_eq!(app.bookmarks.len(), 2);

        app.select_next_bookmark();
        assert_eq!(app.bookmark_selected, 1);

        app.delete_selected_bookmark();
        assert_eq!(app.bookmarks.len(), 1);
        assert_eq!(app.bookmarks[0], path1);
    }

    #[test]
    fn test_go_up_remembers_previous_folder() {
        let cwd = std::env::current_dir().unwrap().canonicalize().unwrap();
        if let Some(parent) = cwd.parent() {
            let download_dir = Arc::new(Mutex::new(cwd.clone()));
            let mut app = AppState::new(
                cwd.clone(),
                "test".to_string(),
                "fp".to_string(),
                8080,
                download_dir,
            );
            app.bookmark_path = None;

            let _ = app.go_up();
            assert_eq!(app.current_path, parent.to_path_buf());
            if let Some(pos) = app.entries.iter().position(|e| e.path == cwd) {
                assert_eq!(app.selected, pos);
            }
        }
    }

    #[test]
    fn test_copy_cut_tagged_files() {
        let download_dir = Arc::new(Mutex::new(PathBuf::from("/tmp")));
        let mut app = AppState::new(
            PathBuf::from("/tmp"),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );
        app.bookmark_path = None;

        app.copy_tagged();
        assert!(app.clipboard.is_empty());

        let file1 = PathBuf::from("/tmp/file1.txt");
        app.entries.push(crate::fs::FileEntry {
            name: "file1.txt".to_string(),
            path: file1.clone(),
            kind: crate::fs::EntryKind::Text,
            size: Some(100),
            modified: None,
        });
        app.selected = 0;

        app.copy_tagged();
        assert_eq!(app.clipboard.len(), 1);
        assert_eq!(app.clipboard[0], file1);
        assert!(!app.clipboard_is_cut);

        app.cut_tagged();
        assert_eq!(app.clipboard.len(), 1);
        assert_eq!(app.clipboard[0], file1);
        assert!(app.clipboard_is_cut);

        let file2 = PathBuf::from("/tmp/file2.txt");
        app.tagged_files.insert(file2.clone());

        app.copy_tagged();
        assert_eq!(app.clipboard.len(), 1);
        assert_eq!(app.clipboard[0], file2);
        assert!(!app.clipboard_is_cut);
    }

    #[test]
    fn test_create_new_folder() {
        let temp_dir = std::env::temp_dir().join(format!("vib_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let download_dir = Arc::new(Mutex::new(temp_dir.clone()));
        let mut app = AppState::new(
            temp_dir.clone(),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );
        app.bookmark_path = None;

        app.new_folder_input = "test_subfolder".to_string();
        app.create_new_folder().unwrap();

        let created_path = temp_dir.join("test_subfolder");
        assert!(created_path.exists());
        assert!(created_path.is_dir());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_perform_rename() {
        let temp_dir =
            std::env::temp_dir().join(format!("vib_test_rename_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let old_file = temp_dir.join("old_name.txt");
        std::fs::write(&old_file, "hello world").unwrap();

        let download_dir = Arc::new(Mutex::new(temp_dir.clone()));
        let mut app = AppState::new(
            temp_dir.clone(),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );
        app.bookmark_path = None;
        app.load().unwrap();

        if let Some(pos) = app.entries.iter().position(|e| e.path == old_file) {
            app.selected = pos;
            app.open_rename_modal();
            app.rename_input = "new_name.txt".to_string();
            app.perform_rename().unwrap();

            let new_file = temp_dir.join("new_name.txt");
            assert!(!old_file.exists());
            assert!(new_file.exists());
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_dual_browsing_panes() {
        let temp_dir =
            std::env::temp_dir().join(format!("vib_test_panes_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let download_dir = Arc::new(Mutex::new(temp_dir.clone()));
        let mut app = AppState::new(
            temp_dir.clone(),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );
        app.bookmark_path = None;

        assert_eq!(app.panes.len(), 1);
        assert_eq!(app.active_pane, 0);

        app.switch_to_pane(1).unwrap();
        assert_eq!(app.panes.len(), 2);
        assert_eq!(app.active_pane, 1);

        app.switch_to_pane(0).unwrap();
        assert_eq!(app.active_pane, 0);

        app.toggle_pane().unwrap();
        assert_eq!(app.active_pane, 1);

        app.switch_to_pane(2).unwrap();
        assert_eq!(app.panes.len(), 2);

        app.close_current_pane().unwrap();
        assert_eq!(app.panes.len(), 1);
        assert_eq!(app.active_pane, 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_transfer_failed_event_sets_fail_banner_and_status() {
        let temp_dir =
            std::env::temp_dir().join(format!("vib_test_cancel_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let download_dir = Arc::new(Mutex::new(temp_dir.clone()));
        let mut app = AppState::new(
            temp_dir.clone(),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );

        app.handle_event(AppEvent::TransferFailed {
            session_id: "session-123".to_string(),
            error: "Transfer cancelled by peer".to_string(),
        });

        assert!(app.fail_banner.is_some());
        let (banner_msg, _) = app.fail_banner.as_ref().unwrap();
        assert!(banner_msg.contains("Transfer Cancelled"));

        assert_eq!(
            app.transfers.get("session-123").unwrap().status,
            "Cancelled / Failed: Transfer cancelled by peer"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cancel_active_transfer() {
        let temp_dir =
            std::env::temp_dir().join(format!("vib_test_cancel_active_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let download_dir = Arc::new(Mutex::new(temp_dir.clone()));
        let mut app = AppState::new(
            temp_dir.clone(),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );

        app.active_receive_progress = Some((50, 100, "test.txt".to_string()));
        app.localsend_modal = LocalSendModalState::ReceiveMode;

        app.cancel_active_transfer();

        assert!(app.active_receive_progress.is_none());
        assert_eq!(app.localsend_modal, LocalSendModalState::Closed);
        assert!(app.fail_banner.is_some());
        let (banner_msg, _) = app.fail_banner.as_ref().unwrap();
        assert!(banner_msg.contains("cancelled by user"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_stale_transfer_progress_ignored_after_failure() {
        let temp_dir =
            std::env::temp_dir().join(format!("vib_test_stale_progress_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let download_dir = Arc::new(Mutex::new(temp_dir.clone()));
        let mut app = AppState::new(
            temp_dir.clone(),
            "test".to_string(),
            "fp".to_string(),
            8080,
            download_dir,
        );

        app.handle_event(AppEvent::TransferFailed {
            session_id: "s1".to_string(),
            error: "Cancelled".to_string(),
        });

        app.handle_event(AppEvent::TransferProgress {
            session_id: "s1".to_string(),
            file_id: "file.bin".to_string(),
            bytes_transferred: 50,
            total_bytes: 100,
            is_upload: false,
        });

        assert!(app.active_receive_progress.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
