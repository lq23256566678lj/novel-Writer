use crate::app::App;
use crate::model::PendingAction;
use eframe::egui;
use std::fs;

pub fn draw_find_replace(app: &mut App, ctx: &egui::Context) {
    if !app.find_open && !app.replace_open {
        return;
    }
    egui::Window::new("查找替换")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("查找:");
                ui.text_edit_singleline(&mut app.find_text);
            });
            if app.replace_open {
                ui.horizontal(|ui| {
                    ui.label("替换:");
                    ui.text_edit_singleline(&mut app.replace_text);
                });
            }
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.find_case_sensitive, "区分大小写");
                if ui.button("查找下一个").clicked() {
                    app.find_next();
                }
                if app.replace_open {
                    if ui.button("替换").clicked() {
                        app.replace_one();
                    }
                    if ui.button("全部替换").clicked() {
                        app.replace_all();
                    }
                }
                if ui.button("关闭").clicked() {
                    app.find_open = false;
                    app.replace_open = false;
                }
            });
        });
}

pub fn draw_project_search(app: &mut App, ctx: &egui::Context) {
    if !app.project_search_open {
        return;
    }
    egui::Window::new("全文搜索")
        .collapsible(false)
        .resizable(true)
        .default_size([500.0, 400.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("关键词:");
                ui.text_edit_singleline(&mut app.project_search_text);
                if ui.button("搜索").clicked() {
                    app.run_project_search();
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let results = app.project_search_results.clone();
                for (path, line, text) in &results {
                    ui.horizontal(|ui| {
                        if ui.link(format!("{}:{}", path.display(), line)).clicked() {
                            app.open_file(path.clone());
                        }
                        ui.label(text);
                    });
                }
            });
        });
}

pub fn draw_batch_rename(app: &mut App, ctx: &egui::Context) {
    if !app.batch_rename_open {
        return;
    }
    egui::Window::new("批量重命名")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("给当前文件夹下所有 .txt 文件加序号前缀");
            ui.horizontal(|ui| {
                ui.label("前缀:");
                ui.text_edit_singleline(&mut app.batch_prefix);
            });
            if ui.button("执行").clicked() {
                app.do_batch_rename();
            }
            if ui.button("取消").clicked() {
                app.batch_rename_open = false;
            }
        });
}

pub fn draw_settings(app: &mut App, ctx: &egui::Context) {
    if !app.settings_open {
        return;
    }
    let mut open = true;
    egui::Window::new("设置")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut app.font_size, 12.0..=28.0).text("字号"));
            ui.add(egui::Slider::new(&mut app.line_height, 1.0..=3.0).text("行距"));
            let mut dark = app.dark_mode;
            if ui.checkbox(&mut dark, "深色主题").changed() {
                app.dark_mode = dark;
                if dark {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
            }
            ui.add(egui::Slider::new(&mut app.daily_goal, 0..=10000).text("每日字数目标"));
        });
    if !open {
        app.settings_open = false;
    }
}

pub fn draw_dialogs(app: &mut App, ctx: &egui::Context) {
    if let Some((old_path, mut new_name)) = app.renaming.take() {
        let mut open = true;
        let mut confirmed = false;
        egui::Window::new("重命名")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let resp = ui.text_edit_singleline(&mut new_name);
                if !resp.has_focus() {
                    resp.request_focus();
                }
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    confirmed = true;
                }
                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        confirmed = true;
                    }
                    if ui.button("取消").clicked() {}
                });
            });

        if confirmed {
            let new_path = old_path.with_file_name(&new_name);
            if fs::rename(&old_path, &new_path).is_ok() {
                for tab in &mut app.tabs {
                    if tab.path == old_path {
                        tab.path = new_path.clone();
                    }
                }
                app.toast = Some("已重命名".to_string());
            } else {
                app.toast = Some("重命名失败".to_string());
            }
        } else if open {
            app.renaming = Some((old_path, new_name));
        }
    }

    if let Some(path) = app.pending_delete.clone() {
        let mut open = true;
        let mut do_delete = false;
        let mut cancel = false;
        egui::Window::new("确认删除")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("确定删除 {} 吗？", path.display()));
                ui.horizontal(|ui| {
                    if ui.button("删除").clicked() {
                        do_delete = true;
                    }
                    if ui.button("取消").clicked() {
                        cancel = true;
                    }
                });
            });

        if do_delete {
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else {
                let _ = fs::remove_file(&path);
            }
            if let Some(idx) = app.tabs.iter().position(|t| t.path == path) {
                app.close_tab(idx);
            }
            app.pending_delete = None;
        } else if cancel || !open {
            app.pending_delete = None;
        }
    }

    match app.pending_action {
        PendingAction::CloseTab(idx) => {
            let mut open = true;
            let mut save = false;
            let mut discard = false;
            let mut cancel = false;
            egui::Window::new("未保存")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("该文件有未保存的修改，是否保存？");
                    ui.horizontal(|ui| {
                        if ui.button("保存").clicked() {
                            save = true;
                        }
                        if ui.button("不保存").clicked() {
                            discard = true;
                        }
                        if ui.button("取消").clicked() {
                            cancel = true;
                        }
                    });
                });
            if save {
                app.save_current();
                app.close_tab(idx);
                app.pending_action = PendingAction::None;
            } else if discard {
                app.close_tab(idx);
                app.pending_action = PendingAction::None;
            } else if cancel || !open {
                app.pending_action = PendingAction::None;
            }
        }
        PendingAction::None => {}
    }
}