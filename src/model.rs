use std::path::PathBuf;

pub const INDENT: &str = "\u{3000}\u{3000}";

pub struct Tab {
    pub path: PathBuf,
    pub content: String,
    pub dirty: bool,
}

impl Tab {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self { path, content, dirty: false }
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string())
    }
}

pub enum TreeAction {
    Open(PathBuf),
    Rename(PathBuf),
    Delete(PathBuf),
    NewFile(PathBuf),
    NewFolder(PathBuf),
}

#[derive(PartialEq, Clone, Copy)]
pub enum PendingAction {
    None,
    CloseTab(usize),
}