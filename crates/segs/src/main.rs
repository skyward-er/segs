mod ui;
mod utils;

use eframe::egui;
use egui::{CornerRadius, Id, Sense, Vec2, ViewportBuilder, vec2};
use mimalloc::MiMalloc;
use segs_assets::{install_fonts, install_icons, load_app_icon};
use segs_memory::{MemoryExt, init_memory};
use segs_ui::{StyleExt, setup_style};

use crate::ui::{
    components::left_menu::LeftMenuSelector,
    panels::{BottomBarControls, TopBarControls},
};

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
    menu_panel_selected: Option<LeftMenuSelector>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &_cc.egui_ctx;
        setup_style(ctx);
        install_fonts(ctx);
        install_icons(ctx);

        let top_bar_controls = ctx.mem().get_perm_or_default(Id::new("top_controls"));
        Self {
            top_bar_controls,
            bottom_bar_controls: BottomBarControls::default(),
            menu_panel_selected: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::panels::top_controls_bar(ctx, &mut self.top_bar_controls);
        ui::panels::bottom_controls_bar(ctx, &mut self.bottom_bar_controls);
        ui::panels::left_bar(ctx, &mut self.menu_panel_selected);
        ui::panels::main_view(
            ctx,
            &mut self.top_bar_controls.panels_controls,
            left_panel_contents,
            right_panel_contents,
            bottom_panel_contents,
            main_panel_contents,
        );

        ctx.mem()
            .insert_perm(Id::new("top_controls"), self.top_bar_controls.clone());
        ctx.mem().sync_persistence().expect("Failed to sync persistent memory");
    }
}

fn left_panel_contents(ui: &mut egui::Ui) {
    // ui.spacing_mut().item_spacing = Vec2::ZERO;
    // let max_rect = ui.max_rect();
    // let desired_size = vec2(max_rect.width() + 10., 30.);
    // let (rect, response) = ui.allocate_exact_size(desired_size, Sense::empty());

    // let painter = ui.painter();
    // let visuals = ui.app_visuals();
    // let mut cr = CornerRadius::ZERO;
    // cr.nw = 5;
    // painter.rect_filled(rect, cr, visuals.main_view_fill);
    // painter.rect_stroke(rect, cr, visuals.main_view_stroke, egui::StrokeKind::Inside);
    ui.label("Right panel");
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
