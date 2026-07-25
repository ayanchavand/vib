//! Keyboard input mapping and action dispatching.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Action representing user intent triggered by key input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Exit the application.
    Quit,
    /// Move cursor selection up.
    Up,
    /// Move cursor selection down.
    Down,
    /// Open directory or execute focused action.
    Enter,
    /// Navigate to parent directory or close active modal.
    Back,
    /// Toggle multi-select status for focused file.
    ToggleSelect,
    /// Select all entries in active pane.
    SelectAll,
    /// Open target device selection modal for LocalSend.
    OpenSendModal,
    /// Accept incoming LocalSend transfer.
    AcceptTransfer,
    /// Broadcast UDP discovery scan for LocalSend peers.
    ScanPeers,
    /// Toggle LocalSend main modal menu.
    ToggleLocalSendModal,
    /// Scroll text file preview pane upwards.
    ScrollPreviewUp,
    /// Scroll text file preview pane downwards.
    ScrollPreviewDown,
    /// Toggle bookmark state for current directory.
    ToggleBookmark,
    /// Toggle bookmarks modal overlay.
    ToggleBookmarkModal,
    /// Delete selected bookmark entry.
    DeleteBookmark,
    /// Copy tagged/selected files to internal clipboard.
    CopyTagged,
    /// Cut tagged/selected files to internal clipboard.
    CutTagged,
    /// Paste clipboard contents to current directory.
    PasteClipboard,
    /// Open new folder creation modal.
    NewFolder,
    /// Open rename modal for focused file or directory.
    Rename,
    /// Switch directly to specified pane index (0 or 1).
    SwitchPane(usize),
    /// Cycle active browsing pane focus.
    TogglePane,
    /// Close secondary browsing pane.
    ClosePane,
    /// No operation / unmapped keypress.
    None,
}

/// Maps a crossterm `KeyEvent` into an application `Action`.
pub fn map_key(key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => return Action::Quit,
            KeyCode::Char('d') => return Action::ScrollPreviewDown,
            KeyCode::Char('u') => return Action::ScrollPreviewUp,
            KeyCode::Char('b') => return Action::ToggleBookmarkModal,
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('t' | 'T') => Action::ToggleLocalSendModal,

        KeyCode::Char('W') | KeyCode::PageUp => Action::ScrollPreviewUp,
        KeyCode::Char('S') | KeyCode::PageDown => Action::ScrollPreviewDown,

        KeyCode::Char('1') => Action::SwitchPane(0),
        KeyCode::Char('2') => Action::SwitchPane(1),
        KeyCode::Tab => Action::TogglePane,
        KeyCode::Char('w') => Action::ClosePane,

        KeyCode::Char('b') => Action::ToggleBookmark,
        KeyCode::Char('B' | 'm' | 'M') => Action::ToggleBookmarkModal,
        KeyCode::Char('d') | KeyCode::Delete => Action::DeleteBookmark,

        KeyCode::Char('c' | 'C') => Action::CopyTagged,
        KeyCode::Char('x' | 'X') => Action::CutTagged,
        KeyCode::Char('p' | 'P') => Action::PasteClipboard,
        KeyCode::Char('n' | 'N') => Action::NewFolder,
        KeyCode::Char('r' | 'R') | KeyCode::F(2) => Action::Rename,
        KeyCode::F(5) => Action::ScanPeers,

        KeyCode::Char(' ') => Action::ToggleSelect,
        KeyCode::Char('v') => Action::SelectAll,
        KeyCode::Char('s') => Action::OpenSendModal,
        KeyCode::Char('y') => Action::AcceptTransfer,

        KeyCode::Up | KeyCode::Char('k') => Action::Up,
        KeyCode::Down | KeyCode::Char('j') => Action::Down,
        KeyCode::Enter | KeyCode::Right => Action::Enter,
        KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => Action::Back,

        _ => Action::None,
    }
}
