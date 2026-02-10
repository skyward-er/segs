mod ui;
mod utils;

use eframe::egui;
use egui::ViewportBuilder;
use mimalloc::MiMalloc;
use segs_assets::{install_fonts, install_icons, load_app_icon};
use segs_memory::{MemoryExt, init_memory};
use segs_ui::setup_style;

use crate::ui::panels::{BottomBarControls, TopBarControls};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Standard eframe setup to run the app
fn main() -> eframe::Result<()> {
    init_memory(utils::get_memory_dirpath()).expect("Failed to initialize memory system");
    let app_icon = load_app_icon();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title_shown(false)
            .with_titlebar_shown(false)
            .with_fullsize_content_view(true)
            .with_icon(app_icon),
        ..Default::default()
    };
    eframe::run_native("SEGS", options, Box::new(|cc| Ok(Box::new(MyApp::new(cc)))))
}

struct MyApp {
    top_bar_controls: TopBarControls,
    bottom_bar_controls: BottomBarControls,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &_cc.egui_ctx;
        setup_style(ctx);
        install_fonts(ctx);
        install_icons(ctx);

        Self {
            top_bar_controls: TopBarControls::default(),
            bottom_bar_controls: BottomBarControls::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::panels::top_controls_bar(ctx, &mut self.top_bar_controls);
        ui::panels::bottom_controls_bar(ctx, &mut self.bottom_bar_controls);

        ui::panels::main_view(
            ctx,
            &mut self.top_bar_controls.panels_controls,
            left_panel_contents,
            right_panel_contents,
            bottom_panel_contents,
            main_panel_contents,
        );

        ctx.mem().sync_persistence().expect("Failed to sync persistent memory");
    }
}

fn left_panel_contents(ui: &mut egui::Ui) {
    ui.label("Left panel");
}

fn right_panel_contents(ui: &mut egui::Ui) {
    ui.label("Right panel");
}

fn bottom_panel_contents(ui: &mut egui::Ui) {
    ui.label("Bottom panel");
}

fn main_panel_contents(ui: &mut egui::Ui) {
    ui.label("Main panel");
}
