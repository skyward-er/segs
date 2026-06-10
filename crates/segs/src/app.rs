use eframe::Frame;
use egui::{Context, Id, Ui};

use segs_assets::{install_fonts, install_icons};
use segs_memory::MemoryExt;
use segs_ui::style::{AppStyle, setup_style};

use crate::args::AppArgs;
use crate::dataflow::adapter::AdapterType;
use crate::dataflow::{DataStore, adapter::DataAdapter, mavlink_adapter::MavlinkAdapter};
use crate::ui::components::mode_toggle::ViewMode;
use crate::ui::layout::Layout;
use crate::ui::views::configuration::ConfigurationView;
use crate::ui::views::operator::OperatorView;
use crate::ui::views::{self, VIEW_MODE_ID, View};
use crate::ui::{status_bar, top_bar};

pub struct App {
    pub view: views::View,
    pub context: AppContext,
}

pub struct AppContext {
    pub data_store: DataStore,
    pub data_adapter: Option<Box<dyn DataAdapter>>,
    pub layout: Layout,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, args: AppArgs) -> Self {
        let ctx = &_cc.egui_ctx;
        setup_style(ctx);
        install_fonts(ctx);
        install_icons(ctx);

        let view = ctx.mem().get_perm_or_default(Id::new("view_state"));
        let mut data_store = DataStore::new();

        let data_adapter = match (args.transport, args.adapter, args.mapping) {
            (Some(transport), Some(AdapterType::MAVLink), Some(mapping)) => {
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
            view,
            context: AppContext {
                data_store,
                data_adapter,
                layout: Layout::new(),
            },
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &Context, _frame: &mut Frame) {
        // Process incoming data
        if let Some(ref mut adapter) = self.context.data_adapter {
            adapter.process_incoming(&mut self.context.data_store);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        // Sync the current style based on the theme, and get a guard to keep it alive for the frame
        let _guard = AppStyle::sync(ui);

        // Show the status bar at the bottom
        status_bar::show(ui, &mut self.context);
        // Show the top bar
        top_bar::show(ui);

        let view_mode: ViewMode = ui.mem().get_temp_or_default(Id::new(VIEW_MODE_ID));

        // TODO: do this properly without recreating every frame
        self.view = match view_mode {
            ViewMode::Configuration => View::Configuration(ConfigurationView {}),
            ViewMode::Operator(layout) => View::Operator(OperatorView { layout }),
        };

        self.view.show(ui, &mut self.context);

        // Save the app state to memory at the end of the update loop
        ui.mem().insert_perm(Id::new("app_state"), self.view.clone());
        // Sync the persistent memory to disk to ensure the state is saved across sessions
        ui.mem().sync_persistence().expect("Failed to sync persistent memory");
    }
}
