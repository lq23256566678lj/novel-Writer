use crate::app::App;
use eframe::egui;

pub fn draw_tabs(app: &mut App, ui: &mut egui::Ui) {
    if app.tabs.is_empty() {
        return;
    }

    let mut close_idx: Option<usize> = None;
    let mut switch_to: Option<usize> = None;
    let mut close_others: Option<usize> = None;
    let mut close_right: Option<usize> = None;

    egui::ScrollArea::horizontal()
        .id_salt("tab_scroll")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in app.tabs.iter().enumerate() {
                    let label = if tab.dirty {
                        format!("● {}", tab.name())
                    } else {
                        tab.name()
                    };
                    let is_active = app.active_tab == Some(i);

                    let resp = ui.selectable_label(is_active, label);

                    if resp.clicked() {
                        switch_to = Some(i);
                    }
                    if resp.middle_clicked() {
                        close_idx = Some(i);
                    }
                    resp.context_menu(|ui| {
                        if ui.button("关闭").clicked() {
                            close_idx = Some(i);
                            ui.close();
                        }
                        if ui.button("关闭其他").clicked() {
                            close_others = Some(i);
                            ui.close();
                        }
                        if ui.button("关闭右侧").clicked() {
                            close_right = Some(i);
                            ui.close();
                        }
                    });

                    if ui.small_button("×").clicked() {
                        close_idx = Some(i);
                    }
                    ui.add_space(4.0);
                }
            });
        });

    if let Some(i) = switch_to {
        app.active_tab = Some(i);
    }
    if let Some(i) = close_idx {
        app.request_close_tab(i);
    }
    if let Some(keep) = close_others {
        for i in (0..app.tabs.len()).rev() {
            if i != keep {
                app.request_close_tab(i);
            }
        }
    }
    if let Some(from) = close_right {
        for i in (from + 1..app.tabs.len()).rev() {
            app.request_close_tab(i);
        }
    }
}