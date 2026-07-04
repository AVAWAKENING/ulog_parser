mod backend;
mod gui;

use std::sync::Arc;

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some((font_name, font_data)) = find_chinese_font() {
        fonts.font_data.insert(
            "chinese".to_owned(),
            Arc::new(egui::FontData::from_owned(font_data)),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "chinese".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "chinese".to_owned());

        log::info!("已加载中文字体: {}", font_name);
    } else {
        log::warn!("未找到中文字体，界面可能显示乱码");
    }

    ctx.set_fonts(fonts);
}

fn find_chinese_font() -> Option<(String, Vec<u8>)> {
    let font_paths: &[&str] = if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\msyhbd.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
            "C:\\Windows\\Fonts\\simsun.ttc",
        ]
    } else {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Medium.ttc",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
            "/usr/share/fonts/truetype/arphic/ukai.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        ]
    };

    for path in font_paths {
        if let Ok(data) = std::fs::read(path) {
            return Some((path.to_string(), data));
        }
    }

    None
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("ULoger Parser"),
        ..Default::default()
    };

    eframe::run_native(
        "ULoger Parser",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(gui::UlogerApp::new(cc)))
        }),
    )
}
