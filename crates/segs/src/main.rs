mod app;
mod args;
mod dataflow;
mod ui;
mod utils;

use egui::ViewportBuilder;
use mimalloc::MiMalloc;

use segs_assets::load_app_icon;
use segs_memory::init_memory;

use crate::app::App;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::parse_args()?;

    init_memory(utils::get_memory_dirpath()).expect("Failed to initialize memory system");
    let app_icon = load_app_icon();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title_shown(false)
            .with_titlebar_shown(false)
            .with_fullsize_content_view(true)
            .with_drag_and_drop(true)
            .with_icon(app_icon),
        ..Default::default()
    };
    eframe::run_native("SEGS", options, Box::new(|cc| Ok(Box::new(App::new(cc, args)))))
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
