use crate::error::AppError;
use crate::events::{AppEvent, IncomingTransferRequest};
use crate::fs::{self, EntryKind, FileEntry};
use crate::input::Action;
use crate::localsend::protocol::Peer;
use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const MESSAGE_TIMEOUT: Duration = Duration::from_secs(4);
const SUCCESS_BANNER_TIMEOUT: Duration = Duration::from_secs(6);
const FAIL_BANNER_TIMEOUT: Duration = Duration::from_secs(6);
const ANIM_INTERVAL: Duration = Duration::from_millis(350);

fn get_bookmark_storage_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("vib").join("bookmarks.json"))
}

fn load_bookmarks_from_disk(path: Option<&std::path::Path>) -> Vec<PathBuf> {
    if let Some(path) = path
        && path.exists()
        && let Ok(content) = std::fs::read_to_string(path)
        && let Ok(bookmarks) = serde_json::from_str::<Vec<PathBuf>>(&content)
    {
        return bookmarks;
    }
    Vec::new()
}

fn save_bookmarks_to_disk(bookmarks: &[PathBuf], path: Option<&std::path::Path>) {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(bookmarks) {
            let _ = std::fs::write(path, json);
        }
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalSendModalState {
    Closed,
    Menu,
    ReceiveMode,
    SendMode,
}

#[derive(Debug, Clone)]
pub struct TransferProgressInfo {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct BrowsingPane {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub list_state: ListState,
    pub text_preview_path: Option<PathBuf>,
    pub text_preview_lines: Vec<String>,
    pub text_preview_scroll: usize,
}

impl BrowsingPane {
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

    pub fn load(&mut self) -> Result<(), AppError> {
        let entries = crate::fs::list_dir(&self.current_path)?;
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

    pub fn update_text_preview(&mut self) {
        if let Some(entry) = self.entries.get(self.selected)
            && (entry.kind == EntryKind::Text || entry.kind == EntryKind::Unknown)
        {
            if self.text_preview_path.as_ref() != Some(&entry.path) {
                self.text_preview_path = Some(entry.path.clone());
                self.text_preview_scroll = 0;
                self.text_preview_lines = match std::fs::read_to_string(&entry.path) {
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

pub struct AppState {
    pub panes: Vec<BrowsingPane>,
    pub active_pane: usize,

    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub list_state: ListState,

    // Bookmarking Feature
    pub bookmarks: Vec<PathBuf>,
    pub bookmark_selected: usize,
    pub bookmark_list_state: ListState,
    pub show_bookmark_modal: bool,
    pub bookmark_path: Option<PathBuf>,

    // File Operations / Clipboard
    pub clipboard: Vec<PathBuf>,
    pub clipboard_is_cut: bool,

    // Text File Preview Pane
    pub text_preview_path: Option<PathBuf>,
    pub text_preview_lines: Vec<String>,
    pub text_preview_scroll: usize,

    // Multi-selection (tagged files to send)
    pub tagged_files: HashSet<PathBuf>,

    // LocalSend Overlay Modal (triggered via `t`)
    pub localsend_modal: LocalSendModalState,
    pub localsend_modal_selected: usize, // 0 for Send, 1 for Receive

    // 3-Frame Receive Mode ASCII Animation
    pub anim_frame: usize,
    pub last_anim_tick: Instant,

    // Receiving Progress & Big Fat Success/Fail Banner
    pub active_receive_progress: Option<(u64, u64, String)>, // (bytes, total, file_name)
    pub success_banner: Option<(String, Instant)>,           // (message, timestamp)
    pub fail_banner: Option<(String, Instant)>,              // (message, timestamp)

    // LocalSend Peers
    pub peers: HashMap<String, Peer>,
    pub peer_list: Vec<Peer>,
    pub peer_selected: usize,
    pub peer_list_state: ListState,

    // Incoming Transfer Requests Queue
    pub incoming_requests: Vec<IncomingTransferRequest>,

    // Active Transfers
    pub transfers: HashMap<String, TransferProgressInfo>,

    // Modals & Interactivity
    pub show_send_modal: bool,
    pub show_new_folder_modal: bool,
    pub new_folder_input: String,
    pub show_rename_modal: bool,
    pub rename_input: String,
    pub rename_target_path: Option<PathBuf>,

    // Local Device Info & Settings
    pub alias: String,
    pub fingerprint: String,
    pub port: u16,
    pub download_dir: Arc<Mutex<PathBuf>>,

    // Status & Error
    pub status_message: Option<(String, Instant)>,
    pub error: Option<(AppError, Instant)>,
}

impl AppState {
    pub fn new(
        path: PathBuf,
        alias: String,
        fingerprint: String,
        port: u16,
        download_dir: Arc<Mutex<PathBuf>>,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut peer_list_state = ListState::default();
        peer_list_state.select(Some(0));

        let bookmark_path = get_bookmark_storage_path();
        let bookmarks = load_bookmarks_from_disk(bookmark_path.as_deref());
        let mut bookmark_list_state = ListState::default();
        if !bookmarks.is_empty() {
            bookmark_list_state.select(Some(0));
        }

        *download_dir.lock().unwrap() = path.clone();

        let initial_pane = BrowsingPane::new(path.clone());

        let mut app = Self {
            panes: vec![initial_pane],
            active_pane: 0,
            current_path: path,
            entries: Vec::new(),
            selected: 0,
            list_state,
            bookmarks,
            bookmark_selected: 0,
            bookmark_list_state,
            show_bookmark_modal: false,
            bookmark_path,
            clipboard: Vec::new(),
            clipboard_is_cut: false,
            text_preview_path: None,
            text_preview_lines: Vec::new(),
            text_preview_scroll: 0,
            tagged_files: HashSet::new(),
            localsend_modal: LocalSendModalState::Closed,
            localsend_modal_selected: 1, // Default to Receive
            anim_frame: 0,
            last_anim_tick: Instant::now(),
            active_receive_progress: None,
            success_banner: None,
            fail_banner: None,
            peers: HashMap::new(),
            peer_list: Vec::new(),
            peer_selected: 0,
            peer_list_state,
            incoming_requests: Vec::new(),
            transfers: HashMap::new(),
            show_send_modal: false,
            show_new_folder_modal: false,
            new_folder_input: String::new(),
            show_rename_modal: false,
            rename_input: String::new(),
            rename_target_path: None,
            alias,
            fingerprint,
            port,
            download_dir,
            status_message: None,
            error: None,
        };
        app.sync_active_pane();
        app
    }

    pub fn sync_active_pane(&mut self) {
        if let Some(pane) = self.panes.get(self.active_pane) {
            self.current_path = pane.current_path.clone();
            self.entries = pane.entries.clone();
            self.selected = pane.selected;
            self.list_state = pane.list_state.clone();
            self.text_preview_path = pane.text_preview_path.clone();
            self.text_preview_lines = pane.text_preview_lines.clone();
            self.text_preview_scroll = pane.text_preview_scroll;
        }
    }

    pub fn switch_to_pane(&mut self, index: usize) -> Result<(), AppError> {
        if index >= 2 {
            return Ok(());
        }
        if index == 1 && self.panes.len() < 2 {
            let initial_path = self.panes[0].current_path.clone();
            let mut new_pane = BrowsingPane::new(initial_path);
            let _ = new_pane.load();
            self.panes.push(new_pane);
            self.active_pane = 1;
            self.sync_active_pane();
            self.set_status("Opened Pane 2 [Dual Pane Mode Active]".to_string());
        } else if index < self.panes.len() {
            self.active_pane = index;
            self.sync_active_pane();
            self.set_status(format!("Switched to Pane {}", index + 1));
        }
        Ok(())
    }

    pub fn toggle_pane(&mut self) -> Result<(), AppError> {
        if self.panes.len() < 2 {
            self.switch_to_pane(1)?;
        } else {
            let next = (self.active_pane + 1) % self.panes.len();
            self.switch_to_pane(next)?;
        }
        Ok(())
    }

    pub fn close_current_pane(&mut self) -> Result<(), AppError> {
        if self.panes.len() > 1 {
            self.panes.remove(self.active_pane);
            if self.active_pane >= self.panes.len() {
                self.active_pane = self.panes.len() - 1;
            }
            self.sync_active_pane();
            self.set_status("Closed Pane [Single Pane Mode]".to_string());
        }
        Ok(())
    }

    pub fn reload_all_panes(&mut self) {
        for pane in &mut self.panes {
            let _ = pane.load();
        }
        self.sync_active_pane();
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        if let Some(pane) = self.panes.get_mut(self.active_pane) {
            let entries = fs::list_dir(&pane.current_path)?;
            pane.entries = entries;
            if pane.selected >= pane.entries.len() && !pane.entries.is_empty() {
                pane.selected = pane.entries.len() - 1;
            }
            pane.list_state.select(if pane.entries.is_empty() {
                None
            } else {
                Some(pane.selected)
            });
            pane.update_text_preview();
        }
        self.sync_active_pane();
        *self.download_dir.lock().unwrap() = self.current_path.clone();
        Ok(())
    }

    pub fn update_text_preview(&mut self) {
        if let Some(entry) = self.entries.get(self.selected)
            && (entry.kind == EntryKind::Text || entry.kind == EntryKind::Unknown)
        {
            if self.text_preview_path.as_ref() != Some(&entry.path) {
                self.text_preview_path = Some(entry.path.clone());
                self.text_preview_scroll = 0;
                self.text_preview_lines = match std::fs::read_to_string(&entry.path) {
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

    pub fn handle_action(&mut self, action: Action) -> Result<Option<PathBuf>, AppError> {
        // If big fat success banner is open, any key dismisses it
        if self.success_banner.is_some()
            && matches!(action, Action::Enter | Action::Back | Action::Quit)
        {
            self.success_banner = None;
            return Ok(None);
        }

        // If big fat fail banner is open, any key dismisses it
        if self.fail_banner.is_some()
            && matches!(action, Action::Enter | Action::Back | Action::Quit)
        {
            self.fail_banner = None;
            return Ok(None);
        }

        // If LocalSend Overlay Modal is Open
        if self.localsend_modal != LocalSendModalState::Closed {
            match action {
                Action::Quit => return Err(AppError::Message("Quit".to_string())),
                Action::ToggleLocalSendModal => {
                    if self.active_receive_progress.is_some() || !self.incoming_requests.is_empty() {
                        self.cancel_active_transfer();
                    } else {
                        self.localsend_modal = LocalSendModalState::Closed;
                    }
                }
                Action::Back => {
                    if self.active_receive_progress.is_some() || !self.incoming_requests.is_empty() {
                        self.cancel_active_transfer();
                    } else if self.localsend_modal == LocalSendModalState::SendMode
                        || self.localsend_modal == LocalSendModalState::ReceiveMode
                    {
                        self.localsend_modal = LocalSendModalState::Menu;
                    } else {
                        self.localsend_modal = LocalSendModalState::Closed;
                    }
                }
                Action::Up => {
                    if self.localsend_modal == LocalSendModalState::Menu {
                        if !self.tagged_files.is_empty() {
                            self.localsend_modal_selected = if self.localsend_modal_selected == 0 {
                                1
                            } else {
                                0
                            };
                        } else {
                            self.localsend_modal_selected = 1;
                        }
                    } else if self.localsend_modal == LocalSendModalState::SendMode {
                        self.select_prev_peer();
                    }
                }
                Action::Down => {
                    if self.localsend_modal == LocalSendModalState::Menu {
                        if !self.tagged_files.is_empty() {
                            self.localsend_modal_selected = if self.localsend_modal_selected == 1 {
                                0
                            } else {
                                1
                            };
                        } else {
                            self.localsend_modal_selected = 1;
                        }
                    } else if self.localsend_modal == LocalSendModalState::SendMode {
                        self.select_next_peer();
                    }
                }
                Action::Enter => {
                    if self.localsend_modal == LocalSendModalState::Menu {
                        if self.localsend_modal_selected == 1 {
                            // Receive Mode selected
                            self.localsend_modal = LocalSendModalState::ReceiveMode;
                            self.set_status(
                                "Listening for incoming LocalSend transfers...".to_string(),
                            );
                        } else {
                            // Send Mode selected
                            if !self.tagged_files.is_empty() {
                                self.localsend_modal = LocalSendModalState::SendMode;
                            } else {
                                self.localsend_modal_selected = 1;
                                self.set_status(
                                    "No files selected! Select files using [Space] first."
                                        .to_string(),
                                );
                            }
                        }
                    } else if self.localsend_modal == LocalSendModalState::ReceiveMode {
                        self.accept_current_incoming();
                    }
                }
                Action::AcceptTransfer => {
                    self.accept_current_incoming();
                }
                Action::NewFolder => {
                    self.decline_current_incoming();
                }
                Action::ScanPeers => {
                    self.set_status("Scanning network for devices...".to_string());
                }
                _ => {}
            }
            return Ok(None);
        }

        // Bookmark Overlay Modal
        if self.show_bookmark_modal {
            match action {
                Action::Quit => return Err(AppError::Message("Quit".to_string())),
                Action::ToggleBookmarkModal | Action::Back => {
                    self.show_bookmark_modal = false;
                }
                Action::Up => self.select_prev_bookmark(),
                Action::Down => self.select_next_bookmark(),
                Action::Enter => {
                    self.jump_to_selected_bookmark()?;
                }
                Action::DeleteBookmark => {
                    self.delete_selected_bookmark();
                }
                Action::ToggleBookmark => {
                    self.toggle_bookmark_current();
                }
                _ => {}
            }
            return Ok(None);
        }

        // Main File Browser view
        match action {
            Action::Quit => return Err(AppError::Message("Quit".to_string())),

            Action::SwitchPane(idx) => {
                self.switch_to_pane(idx)?;
            }

            Action::TogglePane => {
                self.toggle_pane()?;
            }

            Action::ClosePane => {
                self.close_current_pane()?;
            }

            Action::NewFolder => {
                self.show_new_folder_modal = true;
                self.new_folder_input.clear();
            }

            Action::Rename => {
                self.open_rename_modal();
            }

            Action::CopyTagged => {
                self.copy_tagged();
            }

            Action::CutTagged => {
                self.cut_tagged();
            }

            Action::PasteClipboard => {
                self.paste_clipboard()?;
            }

            Action::ToggleBookmark => {
                self.toggle_bookmark_current();
            }

            Action::ToggleBookmarkModal => {
                self.show_bookmark_modal = !self.show_bookmark_modal;
                if self.show_bookmark_modal {
                    if !self.bookmarks.is_empty() && self.bookmark_selected >= self.bookmarks.len()
                    {
                        self.bookmark_selected = 0;
                    }
                    self.bookmark_list_state
                        .select(if self.bookmarks.is_empty() {
                            None
                        } else {
                            Some(self.bookmark_selected)
                        });
                }
            }

            Action::ToggleLocalSendModal => {
                if self.localsend_modal != LocalSendModalState::Closed {
                    self.localsend_modal = LocalSendModalState::Closed;
                } else {
                    self.localsend_modal = LocalSendModalState::Menu;
                    if !self.tagged_files.is_empty() {
                        self.localsend_modal_selected = 0;
                    } else {
                        self.localsend_modal_selected = 1;
                    }
                }
            }

            Action::ScrollPreviewDown => {
                if !self.text_preview_lines.is_empty()
                    && self.text_preview_scroll + 1 < self.text_preview_lines.len()
                {
                    self.text_preview_scroll += 1;
                }
            }

            Action::ScrollPreviewUp => {
                if self.text_preview_scroll > 0 {
                    self.text_preview_scroll -= 1;
                }
            }

            Action::Up => {
                self.select_prev();
            }

            Action::Down => {
                self.select_next();
            }

            Action::Enter => {
                self.enter_selected()?;
            }

            Action::ToggleSelect => {
                self.toggle_select_current();
            }

            Action::SelectAll => {
                self.select_all_in_current_dir();
            }

            Action::OpenSendModal => {
                if !self.tagged_files.is_empty() {
                    self.localsend_modal = LocalSendModalState::SendMode;
                    self.localsend_modal_selected = 0;
                } else {
                    self.localsend_modal = LocalSendModalState::Menu;
                    self.localsend_modal_selected = 1;
                    self.set_status(
                        "No files selected! Select files using [Space] first.".to_string(),
                    );
                }
            }

            Action::AcceptTransfer => {
                self.accept_current_incoming();
            }

            Action::Back => {
                self.go_up()?;
            }

            Action::ScanPeers => {
                self.set_status("Scanning local network for LocalSend devices... [r]".to_string());
            }

            _ => {}
        }

        Ok(None)
    }

    pub fn copy_tagged(&mut self) {
        if !self.tagged_files.is_empty() {
            self.clipboard = self.tagged_files.iter().cloned().collect();
            self.clipboard_is_cut = false;
            let count = self.clipboard.len();
            self.set_status(format!("Copied {} selected item(s) to clipboard.", count));
        } else if let Some(entry) = self.entries.get(self.selected) {
            self.clipboard = vec![entry.path.clone()];
            self.clipboard_is_cut = false;
            self.set_status(format!("Copied '{}' to clipboard.", entry.name));
        } else {
            self.set_status("No file selected to copy!".to_string());
        }
    }

    pub fn cut_tagged(&mut self) {
        if !self.tagged_files.is_empty() {
            self.clipboard = self.tagged_files.iter().cloned().collect();
            self.clipboard_is_cut = true;
            let count = self.clipboard.len();
            self.set_status(format!("Cut {} selected item(s) to clipboard.", count));
        } else if let Some(entry) = self.entries.get(self.selected) {
            self.clipboard = vec![entry.path.clone()];
            self.clipboard_is_cut = true;
            self.set_status(format!("Cut '{}' to clipboard.", entry.name));
        } else {
            self.set_status("No file selected to cut!".to_string());
        }
    }

    pub fn paste_clipboard(&mut self) -> Result<(), AppError> {
        if self.clipboard.is_empty() {
            self.set_status("Clipboard is empty! Copy [c] or cut [x] files first.".to_string());
            return Ok(());
        }

        let mut success_count = 0;
        let is_cut = self.clipboard_is_cut;
        let items_to_process = self.clipboard.clone();

        for src in items_to_process {
            if !src.exists() {
                continue;
            }
            let file_name = match src.file_name() {
                Some(name) => name,
                None => continue,
            };

            let dest = self.current_path.join(file_name);
            if src == dest {
                continue;
            }

            let res = if is_cut {
                std::fs::rename(&src, &dest).or_else(|_| {
                    if src.is_dir() {
                        copy_dir_all(&src, &dest).and_then(|_| std::fs::remove_dir_all(&src))
                    } else {
                        std::fs::copy(&src, &dest)
                            .and_then(|_| std::fs::remove_file(&src))
                            .map(|_| ())
                    }
                })
            } else {
                if src.is_dir() {
                    copy_dir_all(&src, &dest)
                } else {
                    std::fs::copy(&src, &dest).map(|_| ())
                }
            };

            if res.is_ok() {
                success_count += 1;
            }
        }

        if is_cut {
            self.clipboard.clear();
            self.tagged_files.clear();
        }

        self.reload_all_panes();

        if is_cut {
            self.set_status(format!(
                "Moved {} item(s) into current directory.",
                success_count
            ));
        } else {
            self.set_status(format!(
                "Copied {} item(s) into current directory.",
                success_count
            ));
        }

        Ok(())
    }

    pub fn create_new_folder(&mut self) -> Result<(), AppError> {
        let name = self.new_folder_input.trim().to_string();
        if name.is_empty() {
            self.set_error(AppError::Message(
                "Folder name cannot be empty.".to_string(),
            ));
            self.show_new_folder_modal = false;
            self.new_folder_input.clear();
            return Ok(());
        }

        let target_path = self.current_path.join(&name);
        if target_path.exists() {
            self.set_error(AppError::Message(format!(
                "Directory or file '{}' already exists!",
                name
            )));
            self.show_new_folder_modal = false;
            self.new_folder_input.clear();
            return Ok(());
        }

        match std::fs::create_dir_all(&target_path) {
            Ok(_) => {
                self.show_new_folder_modal = false;
                self.new_folder_input.clear();
                self.reload_all_panes();
                if let Some(pos) = self.entries.iter().position(|e| e.path == target_path) {
                    self.selected = pos;
                    self.list_state.select(Some(self.selected));
                    self.update_text_preview();
                }
                self.set_status(format!("Created new folder: {}", name));
            }
            Err(e) => {
                self.show_new_folder_modal = false;
                self.new_folder_input.clear();
                self.set_error(AppError::Message(format!(
                    "Failed to create folder '{}': {}",
                    name, e
                )));
            }
        }
        Ok(())
    }

    pub fn open_rename_modal(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            self.rename_input = entry.name.clone();
            self.rename_target_path = Some(entry.path.clone());
            self.show_rename_modal = true;
        } else {
            self.set_status("No item selected to rename!".to_string());
        }
    }

    pub fn perform_rename(&mut self) -> Result<(), AppError> {
        let old_path = match self.rename_target_path.take() {
            Some(path) => path,
            None => {
                self.show_rename_modal = false;
                self.rename_input.clear();
                return Ok(());
            }
        };

        let new_name = self.rename_input.trim().to_string();
        if new_name.is_empty() {
            self.set_error(AppError::Message("Name cannot be empty.".to_string()));
            self.show_rename_modal = false;
            self.rename_input.clear();
            return Ok(());
        }

        let parent = match old_path.parent() {
            Some(p) => p,
            None => {
                self.show_rename_modal = false;
                self.rename_input.clear();
                return Ok(());
            }
        };

        let new_path = parent.join(&new_name);

        if old_path == new_path {
            self.show_rename_modal = false;
            self.rename_input.clear();
            return Ok(());
        }

        if new_path.exists() {
            self.set_error(AppError::Message(format!(
                "An item named '{}' already exists!",
                new_name
            )));
            self.show_rename_modal = false;
            self.rename_input.clear();
            return Ok(());
        }

        let old_name = old_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        match std::fs::rename(&old_path, &new_path) {
            Ok(_) => {
                self.show_rename_modal = false;
                self.rename_input.clear();

                // Update bookmark entries if renamed item was bookmarked
                if let Some(pos) = self.bookmarks.iter().position(|b| b == &old_path) {
                    self.bookmarks[pos] = new_path.clone();
                    self.save_bookmarks();
                }

                // Update tagged files set if renamed item was selected
                if self.tagged_files.contains(&old_path) {
                    self.tagged_files.remove(&old_path);
                    self.tagged_files.insert(new_path.clone());
                }

                self.reload_all_panes();

                if let Some(pos) = self.entries.iter().position(|e| e.path == new_path) {
                    self.selected = pos;
                    self.list_state.select(Some(self.selected));
                    self.update_text_preview();
                }

                self.set_status(format!("Renamed '{}' to '{}'", old_name, new_name));
            }
            Err(e) => {
                self.show_rename_modal = false;
                self.rename_input.clear();
                self.set_error(AppError::Message(format!(
                    "Failed to rename '{}': {}",
                    old_name, e
                )));
            }
        }

        Ok(())
    }

    pub fn handle_input_key(&mut self, key: crossterm::event::KeyEvent) -> Result<(), AppError> {
        if self.show_new_folder_modal {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_new_folder_modal = false;
                    self.new_folder_input.clear();
                }
                crossterm::event::KeyCode::Enter => {
                    self.create_new_folder()?;
                }
                crossterm::event::KeyCode::Backspace => {
                    self.new_folder_input.pop();
                }
                crossterm::event::KeyCode::Char(c) if !c.is_control() => {
                    self.new_folder_input.push(c);
                }
                _ => {}
            }
        } else if self.show_rename_modal {
            match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.show_rename_modal = false;
                    self.rename_input.clear();
                    self.rename_target_path = None;
                }
                crossterm::event::KeyCode::Enter => {
                    self.perform_rename()?;
                }
                crossterm::event::KeyCode::Backspace => {
                    self.rename_input.pop();
                }
                crossterm::event::KeyCode::Char(c) if !c.is_control() => {
                    self.rename_input.push(c);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn save_bookmarks(&self) {
        save_bookmarks_to_disk(&self.bookmarks, self.bookmark_path.as_deref());
    }

    pub fn toggle_bookmark_current(&mut self) {
        let target = if let Some(entry) = self.entries.get(self.selected) {
            entry.path.clone()
        } else {
            self.current_path.clone()
        };

        if let Some(pos) = self.bookmarks.iter().position(|b| b == &target) {
            self.bookmarks.remove(pos);
            if self.bookmark_selected >= self.bookmarks.len() && !self.bookmarks.is_empty() {
                self.bookmark_selected = self.bookmarks.len() - 1;
            }
            self.bookmark_list_state
                .select(if self.bookmarks.is_empty() {
                    None
                } else {
                    Some(self.bookmark_selected)
                });
            self.set_status(format!("Removed bookmark: {}", target.display()));
        } else {
            self.bookmarks.push(target.clone());
            self.bookmark_selected = self.bookmarks.len() - 1;
            self.bookmark_list_state
                .select(Some(self.bookmark_selected));
            self.set_status(format!("Bookmarked: {}", target.display()));
        }
        self.save_bookmarks();
    }

    pub fn select_prev_bookmark(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }
        if self.bookmark_selected > 0 {
            self.bookmark_selected -= 1;
        } else {
            self.bookmark_selected = self.bookmarks.len() - 1;
        }
        self.bookmark_list_state
            .select(Some(self.bookmark_selected));
    }

    pub fn select_next_bookmark(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }
        if self.bookmark_selected + 1 < self.bookmarks.len() {
            self.bookmark_selected += 1;
        } else {
            self.bookmark_selected = 0;
        }
        self.bookmark_list_state
            .select(Some(self.bookmark_selected));
    }

    pub fn delete_selected_bookmark(&mut self) {
        if self.bookmarks.is_empty() {
            self.set_status("No bookmarks to delete.".to_string());
            return;
        }
        if self.bookmark_selected < self.bookmarks.len() {
            let removed = self.bookmarks.remove(self.bookmark_selected);
            if self.bookmark_selected >= self.bookmarks.len() && !self.bookmarks.is_empty() {
                self.bookmark_selected = self.bookmarks.len() - 1;
            }
            self.bookmark_list_state
                .select(if self.bookmarks.is_empty() {
                    None
                } else {
                    Some(self.bookmark_selected)
                });
            self.save_bookmarks();
            self.set_status(format!("Deleted bookmark: {}", removed.display()));
        }
    }

    pub fn jump_to_selected_bookmark(&mut self) -> Result<(), AppError> {
        if self.bookmarks.is_empty() {
            self.set_status("No bookmarks saved yet!".to_string());
            return Ok(());
        }

        if let Some(target) = self.bookmarks.get(self.bookmark_selected).cloned() {
            self.show_bookmark_modal = false;
            if target.is_dir() {
                self.current_path = target.clone();
                self.selected = 0;
                self.load()?;
                self.set_status(format!("Jumped to directory: {}", target.display()));
            } else if target.is_file() {
                if let Some(parent) = target.parent() {
                    self.current_path = parent.to_path_buf();
                    self.load()?;
                    if let Some(pos) = self.entries.iter().position(|e| e.path == target) {
                        self.selected = pos;
                        self.list_state.select(Some(self.selected));
                        self.update_text_preview();
                    }
                    self.set_status(format!("Jumped to file: {}", target.display()));
                }
            } else {
                if let Some(parent) = target.parent().filter(|p| p.exists()) {
                    self.current_path = parent.to_path_buf();
                    self.selected = 0;
                    self.load()?;
                    self.set_error(AppError::Message(format!(
                        "Target path no longer exists: {}",
                        target.display()
                    )));
                } else {
                    self.set_error(AppError::Message(format!(
                        "Bookmarked path not found: {}",
                        target.display()
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn select_prev(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.entries.len() - 1;
        }
        self.list_state.select(Some(self.selected));
        self.update_text_preview();
        if let Some(pane) = self.panes.get_mut(self.active_pane) {
            pane.selected = self.selected;
            pane.list_state = self.list_state.clone();
            pane.text_preview_path = self.text_preview_path.clone();
            pane.text_preview_lines = self.text_preview_lines.clone();
            pane.text_preview_scroll = self.text_preview_scroll;
        }
    }

    pub fn select_next(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
        self.list_state.select(Some(self.selected));
        self.update_text_preview();
        if let Some(pane) = self.panes.get_mut(self.active_pane) {
            pane.selected = self.selected;
            pane.list_state = self.list_state.clone();
            pane.text_preview_path = self.text_preview_path.clone();
            pane.text_preview_lines = self.text_preview_lines.clone();
            pane.text_preview_scroll = self.text_preview_scroll;
        }
    }

    pub fn select_prev_peer(&mut self) {
        if self.peer_list.is_empty() {
            return;
        }
        if self.peer_selected > 0 {
            self.peer_selected -= 1;
        } else {
            self.peer_selected = self.peer_list.len() - 1;
        }
        self.peer_list_state.select(Some(self.peer_selected));
    }

    pub fn select_next_peer(&mut self) {
        if self.peer_list.is_empty() {
            return;
        }
        if self.peer_selected + 1 < self.peer_list.len() {
            self.peer_selected += 1;
        } else {
            self.peer_selected = 0;
        }
        self.peer_list_state.select(Some(self.peer_selected));
    }

    pub fn enter_selected(&mut self) -> std::io::Result<()> {
        if let Some(entry) = self.entries.get(self.selected) {
            match entry.kind {
                EntryKind::Directory => {
                    let target_path = entry.path.clone();
                    if let Some(pane) = self.panes.get_mut(self.active_pane) {
                        pane.current_path = target_path;
                        pane.selected = 0;
                    }
                    self.load()?;
                }
                EntryKind::Unknown => {
                    self.set_error(AppError::Message(
                        "This file is of unknown format and can't be opened.".to_string(),
                    ));
                }
                _ => {
                    let path = entry.path.clone();
                    let name = entry.name.clone();
                    match open::that(&path) {
                        Ok(_) => {
                            self.set_status(format!("Opened file: {}", name));
                        }
                        Err(_) => {
                            self.set_error(AppError::Message(
                                "This file is of unknown format and can't be opened.".to_string(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn go_up(&mut self) -> std::io::Result<()> {
        if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
            let previous_path = self.current_path.clone();
            if let Some(pane) = self.panes.get_mut(self.active_pane) {
                pane.current_path = parent;
                pane.selected = 0;
            }
            self.load()?;
            if let Some(pos) = self.entries.iter().position(|e| e.path == previous_path) {
                self.selected = pos;
                self.list_state.select(Some(self.selected));
                self.update_text_preview();
                if let Some(pane) = self.panes.get_mut(self.active_pane) {
                    pane.selected = self.selected;
                    pane.list_state = self.list_state.clone();
                    pane.text_preview_path = self.text_preview_path.clone();
                    pane.text_preview_lines = self.text_preview_lines.clone();
                    pane.text_preview_scroll = self.text_preview_scroll;
                }
            }
        }
        Ok(())
    }

    pub fn toggle_select_current(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            let path = entry.path.clone();
            if self.tagged_files.contains(&path) {
                self.tagged_files.remove(&path);
                self.set_status(format!("Deselected: {}", entry.name));
            } else {
                self.tagged_files.insert(path);
                self.set_status(format!("Selected: {}", entry.name));
            }
        }
    }

    pub fn select_all_in_current_dir(&mut self) {
        let all_tagged = self
            .entries
            .iter()
            .all(|e| self.tagged_files.contains(&e.path));
        if all_tagged {
            for entry in &self.entries {
                self.tagged_files.remove(&entry.path);
            }
            self.set_status("Deselected all files in current directory.".to_string());
        } else {
            for entry in &self.entries {
                self.tagged_files.insert(entry.path.clone());
            }
            self.set_status("Selected all files in current directory.".to_string());
        }
    }

    pub fn accept_current_incoming(&mut self) {
        if !self.incoming_requests.is_empty() {
            let req = self.incoming_requests.remove(0);
            let peer_alias = req.peer.alias.clone();
            if let Some(tx) = req.response_tx.lock().unwrap().take() {
                let _ = tx.send(true);
            }
            self.set_status(format!(
                "Accepted incoming transfer from {}! Saving to {}...",
                peer_alias,
                self.current_path.display()
            ));
            self.localsend_modal = LocalSendModalState::ReceiveMode;
        } else {
            self.set_status("No incoming transfer request to accept.".to_string());
        }
    }

    pub fn decline_current_incoming(&mut self) {
        if !self.incoming_requests.is_empty() {
            let req = self.incoming_requests.remove(0);
            let peer_alias = req.peer.alias.clone();
            if let Some(tx) = req.response_tx.lock().unwrap().take() {
                let _ = tx.send(false);
            }
            let msg = format!("Transfer from {} was cancelled/declined.", peer_alias);
            self.fail_banner = Some((msg.clone(), Instant::now()));
            self.set_status(msg);
        } else {
            self.set_status("No incoming transfer request to decline.".to_string());
        }
    }

    pub fn cancel_active_transfer(&mut self) {
        let mut cancelled = false;
        if self.active_receive_progress.is_some() {
            self.active_receive_progress = None;
            cancelled = true;
        }
        if !self.incoming_requests.is_empty() {
            let req = self.incoming_requests.remove(0);
            if let Some(tx) = req.response_tx.lock().unwrap().take() {
                let _ = tx.send(false);
            }
            cancelled = true;
        }
        if cancelled || self.localsend_modal != LocalSendModalState::Closed {
            let msg = "File transfer cancelled by user.".to_string();
            self.fail_banner = Some((msg.clone(), Instant::now()));
            self.set_status(msg);
            self.localsend_modal = LocalSendModalState::Closed;
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::PeerDiscovered(peer) => {
                self.peers.insert(peer.fingerprint.clone(), peer);
                self.peer_list = self.peers.values().cloned().collect();
                if self.peer_selected >= self.peer_list.len() && !self.peer_list.is_empty() {
                    self.peer_selected = self.peer_list.len() - 1;
                }
                self.peer_list_state.select(if self.peer_list.is_empty() {
                    None
                } else {
                    Some(self.peer_selected)
                });
            }
            AppEvent::IncomingTransfer(request) => {
                let peer_alias = request.peer.alias.clone();
                self.incoming_requests.push(request);
                self.localsend_modal = LocalSendModalState::ReceiveMode;
                self.set_status(format!(
                    "INCOMING REQUEST from {}! Press [y] to Accept or [n] to Decline.",
                    peer_alias
                ));
            }
            AppEvent::TransferProgress {
                session_id,
                file_id,
                bytes_transferred,
                total_bytes,
                is_upload,
            } => {
                if let Some(entry) = self.transfers.get(&session_id) {
                    if entry.status.starts_with("Cancelled") || entry.status.starts_with("Failed") {
                        return;
                    }
                }
                if self.fail_banner.is_some() {
                    return;
                }
                let progress_title = if is_upload {
                    format!("Uploading {}", file_id)
                } else {
                    format!("Receiving {}", file_id)
                };
                self.active_receive_progress =
                    Some((bytes_transferred, total_bytes, progress_title));
                let entry =
                    self.transfers
                        .entry(session_id)
                        .or_insert_with(|| TransferProgressInfo {
                            bytes_transferred: 0,
                            total_bytes,
                            status: "In Progress".to_string(),
                        });
                entry.bytes_transferred = bytes_transferred;
                entry.total_bytes = total_bytes;
                if bytes_transferred >= total_bytes {
                    entry.status = "Completed".to_string();
                }
            }
            AppEvent::TransferCompleted {
                session_id,
                message,
            } => {
                self.active_receive_progress = None;
                self.localsend_modal = LocalSendModalState::Closed;
                if let Some(entry) = self.transfers.get_mut(&session_id) {
                    entry.status = "Completed".to_string();
                }
                self.success_banner = Some((message.clone(), Instant::now()));
                self.set_status(message);
                let _ = self.load(); // Refresh file browser so newly received file immediately appears!
            }
            AppEvent::TransferFailed { session_id, error } => {
                self.active_receive_progress = None;
                self.localsend_modal = LocalSendModalState::Closed;
                for req in self.incoming_requests.drain(..) {
                    if let Some(tx) = req.response_tx.lock().unwrap().take() {
                        let _ = tx.send(false);
                    }
                }
                let status_msg = format!("Cancelled / Failed: {}", error);
                if let Some(entry) = self.transfers.get_mut(&session_id) {
                    entry.status = status_msg.clone();
                } else {
                    self.transfers.insert(
                        session_id.clone(),
                        TransferProgressInfo {
                            bytes_transferred: 0,
                            total_bytes: 0,
                            status: status_msg.clone(),
                        },
                    );
                }
                let banner_msg = if error.to_lowercase().contains("cancel") {
                    format!("Transfer Cancelled: {}", error)
                } else {
                    format!("Transfer Failed: {}", error)
                };
                self.fail_banner = Some((banner_msg, Instant::now()));
                self.set_error(AppError::Message(error));
            }
            AppEvent::TriggerScan => {
                self.set_status("Scanning network for peers...".to_string());
            }
            AppEvent::StatusMessage(msg) => {
                self.set_status(msg);
            }
        }
    }

    pub fn update(&mut self) {
        self.clear_expired_messages();
        if self.last_anim_tick.elapsed() >= ANIM_INTERVAL {
            self.anim_frame = (self.anim_frame + 1) % 3;
            self.last_anim_tick = Instant::now();
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
        self.error = None;
    }

    pub fn set_error(&mut self, err: AppError) {
        self.error = Some((err, Instant::now()));
    }

    pub fn clear_expired_messages(&mut self) {
        if let Some((_, time)) = self.status_message
            && time.elapsed() > MESSAGE_TIMEOUT
        {
            self.status_message = None;
        }
        if let Some((_, time)) = self.error
            && time.elapsed() > MESSAGE_TIMEOUT
        {
            self.error = None;
        }
        if let Some((_, time)) = self.success_banner
            && time.elapsed() > SUCCESS_BANNER_TIMEOUT
        {
            self.success_banner = None;
        }
        if let Some((_, time)) = self.fail_banner
            && time.elapsed() > FAIL_BANNER_TIMEOUT
        {
            self.fail_banner = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
            assert_eq!(app.current_path, parent);
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
        let temp_dir = std::env::temp_dir().join(format!("vib_test_cancel_{}", uuid::Uuid::new_v4()));
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
        let temp_dir = std::env::temp_dir().join(format!("vib_test_cancel_active_{}", uuid::Uuid::new_v4()));
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
        let temp_dir = std::env::temp_dir().join(format!("vib_test_stale_progress_{}", uuid::Uuid::new_v4()));
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

        // Try sending stale TransferProgress for the same session
        app.handle_event(AppEvent::TransferProgress {
            session_id: "s1".to_string(),
            file_id: "file.bin".to_string(),
            bytes_transferred: 50,
            total_bytes: 100,
            is_upload: false,
        });

        // active_receive_progress must remain None!
        assert!(app.active_receive_progress.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
