use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum Action {
    Quit,
    Up,
    Down,
    Enter,
    Back,
    ToggleSelect,
    SelectAll,
    OpenSendModal,
    AcceptTransfer,
    ScanPeers,
    ToggleLocalSendModal,
    ScrollPreviewUp,
    ScrollPreviewDown,
    ToggleBookmark,
    ToggleBookmarkModal,
    DeleteBookmark,
    CopyTagged,
    CutTagged,
    PasteClipboard,
    NewFolder,
    Rename,
    SwitchPane(usize),
    TogglePane,
    ClosePane,
    None,
}

pub fn map_key(key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::Quit;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('d') {
        return Action::ScrollPreviewDown;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('u') {
        return Action::ScrollPreviewUp;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b') {
        return Action::ToggleBookmarkModal;
    }

    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('t') | KeyCode::Char('T') => Action::ToggleLocalSendModal,

        // Shift+W (Up) and Shift+S (Down) for preview scroll
        KeyCode::Char('W') | KeyCode::PageUp => Action::ScrollPreviewUp,
        KeyCode::Char('S') | KeyCode::PageDown => Action::ScrollPreviewDown,

        KeyCode::Char('1') => Action::SwitchPane(0),
        KeyCode::Char('2') => Action::SwitchPane(1),
        KeyCode::Tab => Action::TogglePane,
        KeyCode::Char('w') => Action::ClosePane,

        KeyCode::Char('b') => Action::ToggleBookmark,
        KeyCode::Char('B') | KeyCode::Char('m') | KeyCode::Char('M') => Action::ToggleBookmarkModal,
        KeyCode::Char('d') | KeyCode::Delete => Action::DeleteBookmark,

        KeyCode::Char('c') | KeyCode::Char('C') => Action::CopyTagged,
        KeyCode::Char('x') | KeyCode::Char('X') => Action::CutTagged,
        KeyCode::Char('p') | KeyCode::Char('P') => Action::PasteClipboard,
        KeyCode::Char('n') | KeyCode::Char('N') => Action::NewFolder,
        KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::F(2) => Action::Rename,
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
