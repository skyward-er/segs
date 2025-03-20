use egui::epaint::text::{FontInsert, InsertFontFamily};

pub const NOTO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fonts/NotoSans-VariableFont_wdth,wght.ttf"
));

pub const JETBRAINS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fonts/JetBrainsMono-VariableFont_wght.ttf"
));

pub fn add_font(ctx: &egui::Context) {
    ctx.add_font(FontInsert::new(
        "noto_sans",
        egui::FontData::from_static(NOTO),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
    ctx.add_font(FontInsert::new(
        "jetbrains_mono",
        egui::FontData::from_static(JETBRAINS),
        vec![InsertFontFamily {
            family: egui::FontFamily::Monospace,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
}
