use crate::model::TreeAction;
use eframe::egui;
use std::fs;
use std::path::Path;

pub fn draw_tree(
    ui: &mut egui::Ui,
    path: &Path,
    depth: usize,
    action: &mut Option<TreeAction>,
) {
    if path.is_dir() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let header = if depth == 0 {
            egui::CollapsingHeader::new(format!("📁 {}", name))
                .id_salt(path)
                .default_open(true)
        } else {
            egui::CollapsingHeader::new(format!("📁 {}", name)).id_salt(path)
        };

        let resp = header.show(ui, |ui| {
            if let Ok(entries) = fs::read_dir(path) {
                let mut entries: Vec<_> = entries.flatten().collect();
                entries.sort_by_key(|e| e.path());
                for entry in entries {
                    draw_tree(ui, &entry.path(), depth + 1, action);
                }
            }
        });

        resp.header_response.context_menu(|ui| {
            if ui.button("在此新建文件").clicked() {
                *action = Some(TreeAction::NewFile(path.to_path_buf()));
                ui.close();
            }
            if ui.button("在此新建文件夹").clicked() {
                *action = Some(TreeAction::NewFolder(path.to_path_buf()));
                ui.close();
            }
            if depth > 0 {
                ui.separator();
                if ui.button("重命名").clicked() {
                    *action = Some(TreeAction::Rename(path.to_path_buf()));
                    ui.close();
                }
                if ui.button("删除").clicked() {
                    *action = Some(TreeAction::Delete(path.to_path_buf()));
                    ui.close();
                }
            }
        });
    } else {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if ext != "txt" && ext != "md" {
            return;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let resp = ui
            .push_id(path, |ui| ui.button(format!("📄 {}", name)))
            .inner;

        if resp.clicked() {
            *action = Some(TreeAction::Open(path.to_path_buf()));
        }

        resp.context_menu(|ui| {
            if ui.button("打开").clicked() {
                *action = Some(TreeAction::Open(path.to_path_buf()));
                ui.close();
            }
            if ui.button("重命名").clicked() {
                *action = Some(TreeAction::Rename(path.to_path_buf()));
                ui.close();
            }
            if ui.button("删除").clicked() {
                *action = Some(TreeAction::Delete(path.to_path_buf()));
                ui.close();
            }
        });
    }
}