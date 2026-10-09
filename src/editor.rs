use crate::app::App;
use crate::model::INDENT;
use crate::util::handle_newline_indent;
use eframe::egui;
use std::time::Instant;

pub fn draw_editor(app: &mut App, ui: &mut egui::Ui) {
    let Some(idx) = app.active_tab else {
        ui.centered_and_justified(|ui| {
            ui.label("从左侧目录树点击文件打开");
        });
        return;
    };

    let indent = app.indent_enabled;

    let Some(tab) = app.tabs.get_mut(idx) else {
        return;
    };

    let available = ui.available_size();
    let old_content = tab.content.clone();

    let text_edit = egui::TextEdit::multiline(&mut tab.content)
        .desired_width(available.x)
        .desired_rows((available.y / (app.font_size * app.line_height)).max(10.0) as usize)
        .lock_focus(false)
        .font(egui::TextStyle::Body);

    let response = ui.add_sized(available, text_edit);

    if response.changed() {
        if indent {
            tab.content = handle_newline_indent(&old_content, &tab.content);
        }
        tab.dirty = true;
        app.last_edit = Some(Instant::now());
    }

    let wc = tab.content.replace(INDENT, "").chars().count();
    app.goal_progress = wc;
}