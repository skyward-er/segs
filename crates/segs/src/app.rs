use eframe::Frame;
use egui::{Context, Id, Ui, ViewportCommand};

use segs_assets::{install_fonts, install_icons};
use segs_memory::MemoryExt;
use segs_ui::style::{AppStyle, setup_style};

use crate::args::AppArgs;
use crate::dataflow::adapter::{AdapterType, DataAdapterInstance};
use crate::dataflow::{adapter::DataAdapter, skyward_mavlink_adapter::SkywardMavlinkAdapter, store::DataStore};
use crate::layout::{LayoutManager, LayoutManagerError};
use crate::ui::views::{View, ViewTarget};
use crate::ui::{command_panel, layout, status_bar, top_bar};
use crate::utils::get_layouts_dirpath;

const DEFAULT_LAYOUT_SLUG_ID: &str = "default_layout_slug";

pub struct App {
    pub view: View,
    pub context: AppContext,
}

pub struct AppContext {
    pub data_store: DataStore,
    pub data_adapter: Option<DataAdapterInstance>,
    pub layouts: LayoutManager,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, args: AppArgs) -> Result<Self, LayoutManagerError> {
        let ctx = &cc.egui_ctx;
        setup_style(ctx);
        install_fonts(ctx);
        install_icons(ctx);

        let default_slug: Option<String> = ctx.mem().get_perm_or_default(Id::new(DEFAULT_LAYOUT_SLUG_ID));
        // TODO: handle layout load errors in a better way than exiting the app
        let layouts = LayoutManager::load(get_layouts_dirpath(), default_slug)?;
        let view = if layouts.active().is_some() {
            View::from_target(ViewTarget::Operator)
        } else {
            View::from_target(ViewTarget::Welcome)
        };

        let mut data_store = DataStore::new();

        let data_adapter = match (args.transport, args.adapter, args.mapping) {
            (Some(transport), Some(AdapterType::SkywardMavlink), Some(mapping)) => {
                println!("Loading Skyward MAVLink adapter\n\tTransport: {transport:?}\n\tMapping: {mapping:?}");
                let adapter = SkywardMavlinkAdapter::new(ctx.clone(), transport, mapping)
                    .expect("Failed to create Skyward MAVLink adapter");
                Some(DataAdapterInstance::new(adapter))
            }
            _ => None,
        };

        if let Some(ref adapter) = data_adapter {
            adapter.prepare_data_store(&mut data_store);
        }

        Ok(Self {
            view,
            context: AppContext {
                data_store,
                data_adapter,
                layouts,
            },
        })
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &Context, _frame: &mut Frame) {
        if let Some(ref mut adapter) = self.context.data_adapter {
            let data_store = &mut self.context.data_store;

            adapter.process_incoming(data_store);
            adapter.process_outgoing(data_store);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        let _guard = AppStyle::sync(ui);

        // A confirmed close is consumed on the frame after the confirmation dialog
        if layout::take_close_request(ui) {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        } else if ui.input(|input| input.viewport().close_requested()) && self.context.layouts.is_dirty() {
            ui.ctx().send_viewport_cmd(ViewportCommand::CancelClose);
            layout::request_close(ui, &self.context.layouts);
        }

        status_bar::show(ui, &mut self.context);
        top_bar::show(ui);
        command_panel::show(ui, &mut self.context);
        self.view.show(ui, &mut self.context);
        layout::show_overlays(ui, &mut self.context.layouts);

        if let Some(target) = layout::take_transition(ui) {
            self.view = View::from_target(target);
        }
        if let Some(default_slug) = self.context.layouts.take_default_update() {
            ui.mem().insert_perm(Id::new(DEFAULT_LAYOUT_SLUG_ID), default_slug);
        }
        ui.mem().sync_persistence().expect("Failed to sync persistent memory");
    }
}
