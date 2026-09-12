#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod args;
mod dataflow;
mod layout;
mod ui;
mod utils;

use egui::ViewportBuilder;
use mimalloc::MiMalloc;

use segs_assets::load_app_icon;
use segs_memory::init_memory;

use crate::app::App;

const APP_ID: &str = "eu.skywarder.segs2";
const APP_TITLE: &str = "SEGS 2";

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse startup configuration and initialize persistent application state
    let args = args::parse_args()?;
    init_memory(utils::get_memory_dirpath()).expect("Failed to initialize memory system");

    // Configure the native window identity and branded runtime icon
    let app_icon = load_app_icon();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(APP_TITLE)
            .with_app_id(APP_ID)
            .with_drag_and_drop(true)
            .with_icon(app_icon),
        ..Default::default()
    };

    // Start the application with the stable platform identifier
    eframe::run_native(APP_ID, options, Box::new(|cc| Ok(Box::new(App::new(cc, args)?))))
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
