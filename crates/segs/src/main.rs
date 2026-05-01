mod args;
mod csd;
mod dataflow;
mod ui;
mod utils;

use eframe::Frame;
use egui::{Context, Id, Ui, ViewportBuilder};
use mimalloc::MiMalloc;
use serde::{Deserialize, Serialize};

use segs_assets::{install_fonts, install_icons, load_app_icon};
use segs_memory::{MemoryExt, init_memory};
use segs_ui::style::{AppStyle, setup_style};

use crate::args::AppArgs;
use crate::dataflow::adapter::AdapterType;
use crate::dataflow::{DataStore, adapter::DataAdapter, mavlink_adapter::MavlinkAdapter};
use crate::ui::status_bar;
use crate::ui::views;

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
            .with_icon(app_icon)
            .with_decorations(false) // Draw window frame ourselves
            .with_transparent(true), // Needed for rounded corners
        ..Default::default()
    };
    eframe::run_native("SEGS", options, Box::new(|cc| Ok(Box::new(App::new(cc, args)))))
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

struct App {
    state: AppState,
    data_store: DataStore,
    data_adapter: Option<Box<dyn DataAdapter>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct AppState {
    view: views::View,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>, args: AppArgs) -> Self {
        let ctx = &_cc.egui_ctx;
        setup_style(ctx);
        install_fonts(ctx);
        install_icons(ctx);

        let state: AppState = ctx.mem().get_perm_or_default(Id::new("app_state"));
        let mut data_store = DataStore::new();

        let data_adapter = match (args.transport, args.adapter, args.mapping) {
            (Some(transport), Some(AdapterType::Mavlink), Some(mapping)) => {
                println!("Loading MAVLink adapter\n\tTransport: {transport:?}\n\tMapping: {mapping:?}");
                let adapter = MavlinkAdapter::new(transport, mapping).expect("Failed to create MAVLink adapter");
                Some(Box::new(adapter) as Box<dyn DataAdapter>)
            }
            _ => None,
        };

        if let Some(ref adapter) = data_adapter {
            adapter.prepare_data_store(&mut data_store);
        }

        Self {
            state,
            data_store,
            data_adapter,
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &Context, _frame: &mut Frame) {
        // Process incoming data
        if let Some(ref mut adapter) = self.data_adapter {
            adapter.process_incoming(&mut self.data_store);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        // Sync the current style based on the theme, and get a guard to keep it alive
        // for the frame
        let _guard = AppStyle::sync(ui);

        csd::window_frame(ui, "Telemetry", |ui| {
            // Show the status bar at the bottom
            status_bar::show_inside(ui, self);

            // Show the current view based on state
            self.state.view.show_inside(ui);
        });

        // Save the app state to memory at the end of the update loop
        ui.mem().insert_perm(Id::new("app_state"), self.state.clone());
        // Sync the persistent memory to disk to ensure the state is saved across
        // sessions
        ui.mem().sync_persistence().expect("Failed to sync persistent memory");
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}
