mod ui;
mod utils;

use eframe::egui;
use egui::{Color32, Frame, Id, Response, Sense, SidePanel, Ui, Vec2, ViewportBuilder, vec2};
use mimalloc::MiMalloc;
use segs_assets::{
    icons::{self, Icon},
    install_fonts, install_icons, load_app_icon,
};
use segs_memory::{MemoryExt, init_memory};
use segs_ui::{StyleExt, setup_style};

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

        let frame = Frame::new().fill(ctx.style().visuals.panel_fill);
        let id = Id::new("left_panel_contents").with("bools");
        let mut bools: [bool; 6] = ctx.mem().get_temp_or_default(id);
        let old_bools = bools;
        SidePanel::left("menu_panel")
            .frame(frame)
            .resizable(false)
            .show_separator_line(false)
            .exact_width(34.)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.add_space(5.);
                icon_toggle(ui, icons::Window::outline(), icons::Window::solid(), &mut bools[0]);
                icon_toggle(ui, icons::Layout::outline(), icons::Layout::solid(), &mut bools[1]);
                icon_toggle(ui, icons::Stack::outline(), icons::Stack::solid(), &mut bools[2]);
                icon_toggle(ui, icons::Archive::outline(), icons::Archive::solid(), &mut bools[3]);
                icon_toggle(ui, icons::Cloud::outline(), icons::Cloud::solid(), &mut bools[4]);
                icon_toggle(ui, icons::Charts::outline(), icons::Charts::solid(), &mut bools[5]);
            });
        if bools != old_bools {
            for (new, old) in bools.iter_mut().zip(old_bools.iter()) {
                if *new == *old {
                    *new = false;
                }
            }
            ctx.mem().insert_temp(id, bools);
        }

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

fn icon_toggle(ui: &mut Ui, icon_inactive: impl Icon, icon_active: impl Icon, active: &mut bool) -> Response {
    let toggle_size = vec2(34.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(toggle_size, Sense::click());
    let id = response.id;

    // Paint the button
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        if response.clicked() {
            *active = !*active;
        }

        // Animation factors
        let hover_t = ui
            .ctx()
            .animate_bool_with_time(id.with("anim_hover"), response.hovered(), 0.1);
        let active_t = ui.ctx().animate_bool_with_time(id.with("anim_active"), *active, 0.1);
        let combined_t = hover_t.max(active_t);

        if hover_t > 0. {
            let shadow_color = ui.app_visuals().shadow_color_lerp(hover_t);
            painter.rect_filled(rect.shrink2(vec2(4., 1.)), 3., shadow_color);
        }

        let icon_rect = rect.shrink2(vec2(7., 3.));
        let icon_color = Color32::from_rgb(149, 149, 151).lerp_to_gamma(Color32::WHITE, combined_t);

        icon_inactive
            .to_image()
            .tint(icon_color.gamma_multiply(1. - active_t))
            .fit_to_exact_size(icon_rect.size())
            .paint_at(ui, icon_rect);
        icon_active
            .to_image()
            .tint(icon_color.gamma_multiply(active_t))
            .fit_to_exact_size(icon_rect.size())
            .paint_at(ui, icon_rect);
    }

    response
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
