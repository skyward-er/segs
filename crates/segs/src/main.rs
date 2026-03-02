// mod communication;
mod ui;
mod utils;

use eframe::egui;
use egui::{Id, Vec2, ViewportBuilder};
use mimalloc::MiMalloc;
use segs_assets::{install_fonts, install_icons, load_app_icon};
use segs_memory::{MemoryExt, init_memory};
use segs_ui::style::{AppStyle, UiStyleExt, setup_style};
use serde::{Deserialize, Serialize};

use crate::ui::views;

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
            .with_drag_and_drop(true)
            .with_icon(app_icon),
        ..Default::default()
    };
    eframe::run_native("SEGS", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

struct App {
    state: AppState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct AppState {
    view: views::View,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &_cc.egui_ctx;
        setup_style(ctx);
        install_fonts(ctx);
        install_icons(ctx);

        let state: AppState = ctx.mem().get_perm_or_default(Id::new("app_state"));
        Self { state }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync the current style based on the theme, and get a guard to keep it alive
        // for the frame
        let _guard = AppStyle::sync(ctx);

        // Show the current view based on state
        self.state.view.show(ctx);

        // panels::top_controls_bar(ctx, &mut self.state.panels_controls);
        // panels::bottom_controls_bar(ctx, &mut self.state.bottom_bar_controls);
        // panels::left_bar(ctx, &mut self.state.menu_panel_selected);

        // match self.state.menu_panel_selected {
        //     Some(LeftMenuSelector::DataflowEditor) => {
        //         dataflow(ctx, &mut self.state.panels_controls.left_panel_visible);
        //     }
        //     _ => {
        //         panels::main_view(
        //             ctx,
        //             &mut self.state.panels_controls,
        //             left_panel_contents,
        //             right_panel_contents,
        //             bottom_panel_contents,
        //             main_panel_contents,
        //         );
        //     }
        // }

        // Save the app state to memory at the end of the update loop
        ctx.mem().insert_perm(Id::new("app_state"), self.state.clone());
        // Sync the persistent memory to disk to ensure the state is saved across
        // sessions
        ctx.mem().sync_persistence().expect("Failed to sync persistent memory");
    }
}

fn left_panel_contents(ui: &mut egui::Ui) {
    // ui.spacing_mut().item_spacing = Vec2::ZERO;
    // let max_rect = ui.max_rect();
    // let desired_size = vec2(max_rect.width() + 10., 30.);
    // let (rect, response) = ui.allocate_exact_size(desired_size, Sense::empty());

    // let painter = ui.painter();
    // let visuals = ui.app_style();
    // let mut cr = CornerRadius::ZERO;
    // cr.nw = 5;
    // painter.rect_filled(rect, cr, visuals.main_view_fill);
    // painter.rect_stroke(rect, cr, visuals.main_view_stroke,
    // egui::StrokeKind::Inside);
    ui.label("Right panel");
}

fn right_panel_contents(ui: &mut egui::Ui) {
    ui.label("Right panel");
}

fn bottom_panel_contents(ui: &mut egui::Ui) {
    ui.label("Bottom panel");
}

fn main_panel_contents(ui: &mut egui::Ui) {
    ui.with_style_override(
        |s| {
            s.current_background_fill = s.main_view_fill;
        },
        |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(15.);
                ui.label("Main panel");
            });
        },
    );
}
