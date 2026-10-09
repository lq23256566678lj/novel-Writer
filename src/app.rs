use crate::dialogs;
use crate::editor;
use crate::model::{PendingAction, Tab, TreeAction, INDENT};
use crate::settings::{self, Settings};
use crate::tabs;
use crate::tree;
use crate::util::{add_indent_to_all_lines, collect_text_files};
use eframe::egui;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct App {
    pub root: Option<PathBuf>,
    pub tabs: Vec<Tab>,
    pub active_tab: Option<usize>,
    pub file_cache: HashMap<PathBuf, String>,
    pub toast: Option<String>,

    pub last_edit: Option<Instant>,
    pub auto_save_delay: Duration,
    pub auto_save_enabled: bool,

    pub indent_enabled: bool,

    pub renaming: Option<(PathBuf, String)>,
    pub pending_delete: Option<PathBuf>,

    pub find_open: bool,
    pub replace_open: bool,
    pub find_text: String,
    pub replace_text: String,
    pub find_case_sensitive: bool,

    pub focus_mode: bool,
    pub typewriter_scroll: bool,
    pub pending_action: PendingAction,

    pub project_search_open: bool,
    pub project_search_text: String,
    pub project_search_results: Vec<(PathBuf, usize, String)>,

    pub daily_goal: usize,
    pub goal_date: String,
    pub goal_progress: usize,

    pub batch_rename_open: bool,
    pub batch_prefix: String,

    pub settings_open: bool,
    pub font_size: f32,
    pub line_height: f32,
    pub dark_mode: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            root: None,
            tabs: Vec::new(),
            active_tab: None,
            file_cache: HashMap::new(),
            toast: None,
            last_edit: None,
            auto_save_delay: Duration::from_secs(2),
            auto_save_enabled: true,
            indent_enabled: true,
            renaming: None,
            pending_delete: None,
            find_open: false,
            replace_open: false,
            find_text: String::new(),
            replace_text: String::new(),
            find_case_sensitive: false,
            focus_mode: false,
            typewriter_scroll: false,
            pending_action: PendingAction::None,
            project_search_open: false,
            project_search_text: String::new(),
            project_search_results: Vec::new(),
            daily_goal: 3000,
            goal_date: String::new(),
            goal_progress: 0,
            batch_rename_open: false,
            batch_prefix: "第".to_string(),
            settings_open: false,
            font_size: 16.0,
            line_height: 1.5,
            dark_mode: false,
        }
    }
}

impl App {
    pub fn load_settings(&mut self, ctx: &egui::Context) {
        let s = settings::load();
        self.auto_save_enabled = s.auto_save_enabled;
        self.indent_enabled = s.indent_enabled;
        self.typewriter_scroll = s.typewriter_scroll;
        self.font_size = s.font_size;
        self.line_height = s.line_height;
        self.dark_mode = s.dark_mode;
        self.daily_goal = s.daily_goal;
        self.goal_date = s.goal_date.clone();
        self.goal_progress = s.goal_progress;

        if let Some(root) = &s.last_root {
            if root.exists() {
                self.root = Some(root.clone());
            }
        }

        for p in &s.open_tabs {
            if p.exists() {
                if let Ok(content) = fs::read_to_string(p) {
                    self.tabs.push(Tab::new(p.clone(), content));
                }
            }
        }
        if let Some(idx) = s.active_tab {
            if idx < self.tabs.len() {
                self.active_tab = Some(idx);
            }
        }

        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    pub fn save_settings(&self) {
        let s = Settings {
            font_size: self.font_size,
            line_height: self.line_height,
            dark_mode: self.dark_mode,
            auto_save_enabled: self.auto_save_enabled,
            indent_enabled: self.indent_enabled,
            typewriter_scroll: self.typewriter_scroll,
            daily_goal: self.daily_goal,
            last_root: self.root.clone(),
            open_tabs: self.tabs.iter().map(|t| t.path.clone()).collect(),
            active_tab: self.active_tab,
            goal_date: self.goal_date.clone(),
            goal_progress: self.goal_progress,
        };
        settings::save(&s);
    }

    pub fn open_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.root = Some(path);
        }
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab = Some(idx);
            return;
        }
        let content = self
            .file_cache
            .remove(&path)
            .or_else(|| fs::read_to_string(&path).ok())
            .unwrap_or_default();
        self.tabs.push(Tab::new(path, content));
        self.active_tab = Some(self.tabs.len() - 1);
    }

    pub fn save_current(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                match fs::write(&tab.path, &tab.content) {
                    Ok(_) => {
                        tab.dirty = false;
                        self.toast = Some(format!("已保存: {}", tab.name()));
                    }
                    Err(e) => {
                        self.toast = Some(format!("保存失败: {}", e));
                    }
                }
            }
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        let tab = &self.tabs[idx];
        self.file_cache.insert(tab.path.clone(), tab.content.clone());
        self.tabs.remove(idx);

        self.active_tab = match self.active_tab {
            Some(active) if active == idx => {
                if self.tabs.is_empty() {
                    None
                } else {
                    Some(active.min(self.tabs.len() - 1))
                }
            }
            Some(active) if active > idx => Some(active - 1),
            other => other,
        };
    }

    pub fn request_close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() && self.tabs[idx].dirty {
            self.pending_action = PendingAction::CloseTab(idx);
        } else {
            self.close_tab(idx);
        }
    }

    pub fn new_file(&mut self, dir: &Path) {
        let mut n = 1;
        let mut path = dir.join("新建章节.txt");
        while path.exists() {
            n += 1;
            path = dir.join(format!("新建章节{}.txt", n));
        }
        let content = if self.indent_enabled {
            INDENT.to_string()
        } else {
            String::new()
        };
        if fs::write(&path, &content).is_ok() {
            self.open_file(path);
            self.toast = Some("已新建文件".to_string());
        }
    }

    pub fn new_folder(&mut self, dir: &Path) {
        let mut n = 1;
        let mut path = dir.join("新建文件夹");
        while path.exists() {
            n += 1;
            path = dir.join(format!("新建文件夹{}", n));
        }
        if fs::create_dir(&path).is_ok() {
            self.toast = Some("已新建文件夹".to_string());
        }
    }

    pub fn has_dirty(&self) -> bool {
        self.tabs.iter().any(|t| t.dirty)
    }

    pub fn save_status(&self) -> String {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                return if tab.dirty { "未保存".to_string() } else { "已保存".to_string() };
            }
        }
        "—".to_string()
    }

    pub fn word_count(&self) -> usize {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                let text = tab.content.replace(INDENT, "");
                return text.chars().count();
            }
        }
        0
    }

    pub fn export_merged(&mut self) {
        let Some(root) = self.root.clone() else {
            self.toast = Some("请先打开文件夹".to_string());
            return;
        };
        let Some(save_path) = rfd::FileDialog::new()
            .set_file_name("合并稿.txt")
            .save_file()
        else {
            return;
        };

        let mut files = Vec::new();
        collect_text_files(&root, &mut files);
        files.sort();

        let mut output = String::new();
        for f in &files {
            if let Ok(text) = fs::read_to_string(f) {
                output.push_str(&text);
                output.push_str("\n\n");
            }
        }

        match fs::write(&save_path, output) {
            Ok(_) => self.toast = Some(format!("已导出: {}", save_path.display())),
            Err(e) => self.toast = Some(format!("导出失败: {}", e)),
        }
    }

    pub fn run_project_search(&mut self) {
        self.project_search_results.clear();
        let Some(root) = self.root.clone() else { return };
        let keyword = self.project_search_text.clone();
        if keyword.is_empty() {
            return;
        }
        let mut stack = vec![root];
        while let Some(dir) = stack.pop() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().map_or(false, |e| e == "txt" || e == "md") {
                        if let Ok(text) = fs::read_to_string(&path) {
                            for (i, line) in text.lines().enumerate() {
                                if line.contains(&keyword) {
                                    self.project_search_results.push((
                                        path.clone(),
                                        i + 1,
                                        line.to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        self.toast = Some(format!("找到 {} 条结果", self.project_search_results.len()));
    }

    pub fn do_batch_rename(&mut self) {
        let Some(root) = self.root.clone() else { return };
        let mut files = Vec::new();
        collect_text_files(&root, &mut files);
        files.sort();
        let prefix = self.batch_prefix.clone();
        for (i, f) in files.iter().enumerate() {
            let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if name.starts_with(&prefix) {
                continue;
            }
            let new_name = format!("{}{}章 {}.txt", prefix, i + 1, name.trim_end_matches(".txt"));
            let new_path = f.with_file_name(new_name);
            let _ = fs::rename(f, new_path);
        }
        self.toast = Some("批量重命名完成".to_string());
        self.batch_rename_open = false;
    }

    pub fn find_next(&mut self) {
        if self.find_text.is_empty() {
            return;
        }
        let Some(idx) = self.active_tab else { return };
        let Some(tab) = self.tabs.get(idx) else { return };
        let haystack = if self.find_case_sensitive {
            tab.content.clone()
        } else {
            tab.content.to_lowercase()
        };
        let needle = if self.find_case_sensitive {
            self.find_text.clone()
        } else {
            self.find_text.to_lowercase()
        };
        if let Some(pos) = haystack.find(&needle) {
            self.toast = Some(format!("找到，位置: {}", pos));
        } else {
            self.toast = Some("未找到".to_string());
        }
    }

    pub fn replace_one(&mut self) {
        if self.find_text.is_empty() {
            return;
        }
        let find = self.find_text.clone();
        let replace = self.replace_text.clone();
        let case = self.find_case_sensitive;
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                let haystack = if case { tab.content.clone() } else { tab.content.to_lowercase() };
                let needle = if case { find.clone() } else { find.to_lowercase() };
                if let Some(pos) = haystack.find(&needle) {
                    tab.content.replace_range(pos..pos + find.len(), &replace);
                    tab.dirty = true;
                    self.toast = Some("已替换".to_string());
                }
            }
        }
    }

    pub fn replace_all(&mut self) {
        if self.find_text.is_empty() {
            return;
        }
        let find = self.find_text.clone();
        let replace = self.replace_text.clone();
        let case = self.find_case_sensitive;
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                let count = if case {
                    tab.content.matches(&find).count()
                } else {
                    tab.content.to_lowercase().matches(&find.to_lowercase()).count()
                };
                tab.content = if case {
                    tab.content.replace(&find, &replace)
                } else {
                    let mut result = String::new();
                    let lower = tab.content.to_lowercase();
                    let lower_find = find.to_lowercase();
                    let mut last = 0;
                    for (i, _) in lower.match_indices(&lower_find) {
                        result.push_str(&tab.content[last..i]);
                        result.push_str(&replace);
                        last = i + find.len();
                    }
                    result.push_str(&tab.content[last..]);
                    result
                };
                tab.dirty = true;
                self.toast = Some(format!("替换了 {} 处", count));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// eframe::App 实现 —— 0.34 用 ui()
// ---------------------------------------------------------------------------

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if self.auto_save_enabled {
            if let Some(last) = self.last_edit {
                if last.elapsed() >= self.auto_save_delay {
                    if self.has_dirty() {
                        self.save_current();
                    }
                    self.last_edit = None;
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.command) {
            self.find_open = !self.find_open;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.command) {
            self.replace_open = !self.replace_open;
            self.find_open = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            self.focus_mode = !self.focus_mode;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.command) {
            self.save_current();
        }

        if !self.focus_mode {
            egui::Panel::top("menu").show_inside(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    if ui.button("打开文件夹").clicked() {
                        self.open_folder();
                    }
                    if ui.button("保存").clicked() {
                        self.save_current();
                    }
                    ui.separator();
                    if ui.button("查找").clicked() {
                        self.find_open = !self.find_open;
                    }
                    if ui.button("替换").clicked() {
                        self.replace_open = !self.replace_open;
                        self.find_open = true;
                    }
                    if ui.button("全文搜索").clicked() {
                        self.project_search_open = !self.project_search_open;
                    }
                    ui.separator();
                    if ui.button("导出合并").clicked() {
                        self.export_merged();
                    }
                    if ui.button("批量重命名").clicked() {
                        self.batch_rename_open = !self.batch_rename_open;
                    }
                    ui.separator();
                    ui.checkbox(&mut self.auto_save_enabled, "自动保存");
                    ui.checkbox(&mut self.indent_enabled, "首行缩进");
                    ui.checkbox(&mut self.typewriter_scroll, "打字机滚动");
                    if ui.button("专注模式").clicked() {
                        self.focus_mode = !self.focus_mode;
                    }
                    if ui.button("设置").clicked() {
                        self.settings_open = !self.settings_open;
                    }
                });
            });
        }

        egui::Panel::bottom("status").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let wc = self.word_count();
                ui.label(format!("字数: {}", wc));
                ui.separator();
                ui.label(self.save_status());
                ui.separator();
                ui.label(if self.auto_save_enabled { "自动保存: 开" } else { "自动保存: 关" });
                ui.separator();
                let progress = if self.daily_goal > 0 {
                    (self.goal_progress as f32 / self.daily_goal as f32 * 100.0).min(100.0)
                } else {
                    0.0
                };
                ui.label(format!("目标: {}/{} ({:.0}%)", self.goal_progress, self.daily_goal, progress));
                if let Some(root) = &self.root {
                    ui.separator();
                    ui.label(root.display().to_string());
                }
                if let Some(msg) = &self.toast {
                    ui.separator();
                    ui.label(msg);
                }
            });
        });

        if !self.focus_mode {
            egui::Panel::left("tree")
                .resizable(true)
                .default_size(260.0)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("目录");
                        ui.add_space(8.0);
                        if ui.add_sized([70.0, 22.0], egui::Button::new("＋文件")).clicked() {
                            if let Some(root) = self.root.clone() {
                                self.new_file(&root);
                            }
                        }
                        if ui.add_sized([70.0, 22.0], egui::Button::new("＋文件夹")).clicked() {
                            if let Some(root) = self.root.clone() {
                                self.new_folder(&root);
                            }
                        }
                    });
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("tree_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let root = self.root.clone();
                            if let Some(root) = root {
                                let mut action: Option<TreeAction> = None;
                                tree::draw_tree(ui, &root, 0, &mut action);
                                if let Some(action) = action {
                                    match action {
                                        TreeAction::Open(p) => {
                                            self.open_file(p);
                                            if self.indent_enabled {
                                                if let Some(idx) = self.active_tab {
                                                    if let Some(tab) = self.tabs.get_mut(idx) {
                                                        tab.content = add_indent_to_all_lines(&tab.content);
                                                    }
                                                }
                                            }
                                        }
                                        TreeAction::Rename(p) => {
                                            let name = p
                                                .file_name()
                                                .map(|n| n.to_string_lossy().to_string())
                                                .unwrap_or_default();
                                            self.renaming = Some((p, name));
                                        }
                                        TreeAction::Delete(p) => {
                                            self.pending_delete = Some(p);
                                        }
                                        TreeAction::NewFile(dir) => self.new_file(&dir),
                                        TreeAction::NewFolder(dir) => self.new_folder(&dir),
                                    }
                                }
                            } else {
                                ui.label("点击「打开文件夹」开始");
                            }
                        });
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if !self.focus_mode {
                tabs::draw_tabs(self, ui);
                if !self.tabs.is_empty() {
                    ui.separator();
                }
            }
            editor::draw_editor(self, ui);
        });

        dialogs::draw_find_replace(self, &ctx);
        dialogs::draw_project_search(self, &ctx);
        dialogs::draw_batch_rename(self, &ctx);
        dialogs::draw_settings(self, &ctx);
        dialogs::draw_dialogs(self, &ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        for tab in &mut self.tabs {
            if tab.dirty {
                let _ = fs::write(&tab.path, &tab.content);
                tab.dirty = false;
            }
        }
        self.save_settings();
    }
}