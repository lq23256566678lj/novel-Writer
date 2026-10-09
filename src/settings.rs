use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub font_size: f32,
    pub line_height: f32,
    pub dark_mode: bool,
    pub auto_save_enabled: bool,
    pub indent_enabled: bool,
    pub typewriter_scroll: bool,
    pub daily_goal: usize,
    pub last_root: Option<PathBuf>,
    pub open_tabs: Vec<PathBuf>,
    pub active_tab: Option<usize>,
    pub goal_date: String,
    pub goal_progress: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_height: 1.5,
            dark_mode: false,
            auto_save_enabled: true,
            indent_enabled: true,
            typewriter_scroll: false,
            daily_goal: 3000,
            last_root: None,
            open_tabs: Vec::new(),
            active_tab: None,
            goal_date: String::new(),
            goal_progress: 0,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "novelwriter", "NovelWriter")
        .map(|d| d.config_dir().join("settings.json"))
}

pub fn load() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };
    if let Ok(text) = fs::read_to_string(&path) {
        if let Ok(s) = serde_json::from_str::<Settings>(&text) {
            return s;
        }
    }
    Settings::default()
}

pub fn save(s: &Settings) {
    let Some(path) = settings_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(s) {
        let _ = fs::write(path, text);
    }
}