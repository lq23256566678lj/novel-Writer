mod app;
mod dialogs;
mod editor;
mod model;
mod settings;
mod tabs;
mod tree;
mod util;

use app::App;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 550.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Novel Writer",
        options,
        Box::new(|cc| {
            // 手动加载中文字体
            let mut fonts = egui::FontDefinitions::default();

            // 优先尝试微软雅黑，失败则尝试其他
            let candidates = [
                "C:/Windows/Fonts/msyh.ttc",
                "C:/Windows/Fonts/msyh.ttf",
                "C:/Windows/Fonts/simhei.ttf",
                "C:/Windows/Fonts/simsun.ttc",
            ];
            let mut loaded = false;
            for path in &candidates {
                if let Ok(data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(data)),
                    );
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "chinese".to_owned());
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .insert(0, "chinese".to_owned());
                    loaded = true;
                    break;
                }
            }
            if !loaded {
                eprintln!("警告：未能加载中文字体，中文可能显示为方块");
            }
            cc.egui_ctx.set_fonts(fonts);

            let mut app = App::default();
            app.load_settings(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}