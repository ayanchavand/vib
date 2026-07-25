//! Modal overlay states for LocalSend and file manager dialogues.

/// State of the LocalSend modal interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocalSendModalState {
    /// Modal is closed.
    #[default]
    Closed,
    /// Main menu showing Send / Receive options.
    Menu,
    /// Receive mode with animation and active progress bar.
    ReceiveMode,
    /// Send mode showing discovered peer devices.
    SendMode,
}
