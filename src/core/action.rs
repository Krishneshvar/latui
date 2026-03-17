use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Represents all possible actions that can be executed by different modes.
/// Each variant corresponds to a specific mode's execution capability.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Action {
    /// Launch an application (Apps mode)
    /// Contains the command to execute
    Launch(String),
    
    /// Execute a shell command (Run mode)
    /// Contains the command string to run in shell
    Command(String),
    
    /// Open a file with the default application (Files mode)
    /// Contains the absolute path to the file
    OpenFile(PathBuf),
    
    /// Open a folder in the default file manager (Files mode)
    /// Contains the absolute path to the directory
    OpenFolder(PathBuf),
    
    /// Copy text to system clipboard (Clipboard mode)
    /// Contains the text content to copy
    CopyToClipboard(String),
    
    /// Paste from system clipboard (Clipboard mode)
    /// This is a signal action; actual clipboard content is retrieved at execution time
    PasteFromClipboard,
    
    /// Insert an emoji character (Emojis mode)
    /// Contains the emoji character or sequence
    InsertEmoji(String),
    
    /// Execute a custom command with arguments (Custom modes)
    /// First string is the command, Vec contains arguments
    Custom { command: String, args: Vec<String> },
}
