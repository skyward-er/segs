mod ui;
mod utils;

use std::f32::consts::FRAC_PI_2;

use eframe::egui;
use egui::{
    Align, Align2, Area, Color32, CornerRadius, Frame, Id, Layout, Rect, Response, Sense, Separator, Ui, UiBuilder,
    Vec2, ViewportBuilder, vec2,
};
use mimalloc::MiMalloc;
use segs_assets::{
    Font,
    fonts::Figtree,
    icons::{self, Icon},
    install_fonts, install_icons, load_app_icon,
};
use segs_memory::{MemoryExt, init_memory};
use segs_ui::{StyleExt, containers::ResizablePanel, setup_style};

use crate::ui::{
    components::left_menu::LeftMenuSelector,
    panels::{self, BottomBarControls, TopBarControls},
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
            .with_drag_and_drop(true)
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
        let menu_panel_selected = ctx.mem().get_perm_or_default(Id::new("menu_panel_selected"));
        Self {
            top_bar_controls,
            bottom_bar_controls: BottomBarControls::default(),
            menu_panel_selected,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        panels::top_controls_bar(ctx, &mut self.top_bar_controls);
        panels::bottom_controls_bar(ctx, &mut self.bottom_bar_controls);
        panels::left_bar(ctx, &mut self.menu_panel_selected);

        match self.menu_panel_selected {
            Some(LeftMenuSelector::DataflowEditor) => {
                dataflow(ctx, &mut self.top_bar_controls.panels_controls.left_panel_visible);
            }
            _ => {
                panels::main_view(
                    ctx,
                    &mut self.top_bar_controls.panels_controls,
                    left_panel_contents,
                    right_panel_contents,
                    bottom_panel_contents,
                    main_panel_contents,
                );
            }
        }

        ctx.mem()
            .insert_perm(Id::new("menu_panel_selected"), self.menu_panel_selected);
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
    ui.vertical_centered(|ui| {
        let frame = egui::Frame::group(ui.style())
            .fill(egui::Color32::from_gray(30))
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(200.0, 100.0));
                ui.label("Drop files here");
            });

        // 1. Check if files are currently being hovered over the window
        if !ui.input(|i| i.raw.hovered_files.is_empty()) {
            let text = "Release to upload!";
            // You can draw a highlight or a tooltip
            ui.ctx().debug_painter().text(
                frame.response.rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(20.0),
                egui::Color32::WHITE,
            );
        }

        // 2. Collect dropped files
        ui.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    println!("Dropped file: {:?}", path);
                    let downloaded_dor = utils::get_downloaded_dirpath().join("dialects");
                    std::fs::create_dir_all(&downloaded_dor).expect("Failed to create downloaded/dialects directory");
                    let filename = path.file_name().unwrap();
                    let dest_path = downloaded_dor.join(filename);
                    std::fs::copy(path, dest_path).expect("Failed to copy dropped file");
                    println!("File copied to downloaded/dialects");
                } else if let Some(bytes) = &file.bytes {
                    println!("Dropped {} bytes", bytes.len());
                }
            }
        });
    });
}

fn dataflow(ctx: &egui::Context, left_panel_visible: &mut bool) {
    let visuals = ctx.app_visuals();
    let back_frame = Frame::new().fill(visuals.egui().panel_fill);
    let mut cr = CornerRadius::same(5);
    // Only round the left corners
    cr.ne = 0;
    cr.se = 0;
    let front_frame = Frame::new()
        .corner_radius(cr)
        .fill(visuals.main_panels_fill)
        .stroke(visuals.main_view_stroke);
    egui::CentralPanel::default().frame(back_frame).show(ctx, |ui| {
        // Define collapse state based on visibility
        let mut collapsed_left = !*left_panel_visible;

        let visuals = ctx.app_visuals();
        let panel_outer_frame = Frame::new().corner_radius(5.).fill(visuals.main_panels_fill);
        let panel_inner_frame = Frame::NONE;
        let main_inner_frame = panel_inner_frame.corner_radius(5.).fill(visuals.main_panels_fill);

        let left = ResizablePanel::horizontal_left()
            .set_minimum_size(180.)
            .inactive_separator_stroke(visuals.main_view_stroke)
            .left_frame(panel_outer_frame)
            .collapsed(&mut collapsed_left);

        let layout = Layout::top_down(Align::Min);

        front_frame.show(ui, |ui| {
            left.show(ui, |panel| {
                panel
                    .show_left(|ui| {
                        panel_inner_frame.show(ui, |ui| {
                            ui.set_min_size(ui.available_size());
                            ui.set_clip_rect(ui.max_rect());
                            ui.with_layout(layout, dataflow_left);
                        });
                    })
                    .show_right(|ui| {
                        main_inner_frame.show(ui, |ui| {
                            ui.set_min_size(ui.available_size());
                            ui.with_layout(layout, dataflow_right);
                        });
                    });
            });
        });

        // Update visibility state based on collapses
        *left_panel_visible = !collapsed_left;
    });
}

fn dataflow_left(ui: &mut egui::Ui) {
    section_selector(ui);
    add_separator(ui);
    section_controls(ui);
    // add_separator(ui);
    // panel_content(ui);
}

fn dataflow_right(ui: &mut egui::Ui) {
    Frame::new().inner_margin(8.).show(ui, |ui| {
        ui.label("Dataflow main panel");
    });
}

fn add_separator(ui: &mut Ui) {
    ui.visuals_mut().widgets.noninteractive.bg_stroke = ui.ctx().app_visuals().main_view_stroke;
    ui.add(Separator::default().spacing(0.));
}

fn section_selector(ui: &mut Ui) -> Response {
    let size = vec2(ui.available_width(), 30.);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let painter = ui.painter();

    let text = "Data Input Schemas";
    let text_color = ui.visuals().text_color();
    let galley = painter.layout_no_wrap(text.to_owned(), Figtree::medium().sized(14.), text_color);

    let mut cr = CornerRadius::ZERO;
    cr.nw = 5;
    let color = Color32::from_rgb(26, 26, 28);
    painter.rect_filled(rect, cr, color);

    let (icon_rect, text_rect) = rect.split_left_right_at_x(rect.left() + 20.);

    let id_toggled = ui.id().with("toggled");
    let mut toggled: bool = ui.ctx().mem().get_temp_or_default(id_toggled);

    if response.clicked() {
        toggled = !toggled;
        ui.ctx().mem().insert_temp(id_toggled, toggled);
    }

    if toggled {
        let id = ui.id().with("section_selector_area");
        Area::new(id)
            .pivot(Align2::LEFT_TOP)
            .fixed_pos(rect.left_bottom() + Vec2::splat(5.))
            .show(ui.ctx(), |ui| {
                let style = ui.style();
                Frame::new()
                    .corner_radius(style.visuals.menu_corner_radius)
                    .shadow(style.visuals.popup_shadow)
                    .fill(style.visuals.window_fill())
                    .stroke(style.visuals.window_stroke())
                    .show(ui, |ui| {
                        // Frame::new().inner_margin(vec2(10., 5.)).show(ui, |ui| {
                        //     asfas(ui);
                        // });
                        // ui.add(Separator::default().spacing(0.));
                        Frame::new().inner_margin(vec2(5., 5.)).show(ui, |ui| {
                            ui.set_min_width(rect.width() - 22.);
                            ui.spacing_mut().item_spacing = Vec2::splat(7.);
                            ui.label("Option 1");
                            ui.label("Option 2");
                            ui.label("Option 3");
                        });
                    })
            });
    }

    let id = ui.id().with("active_animation");
    let active_t = ui.ctx().animate_bool_with_time(id, toggled, 0.1);

    painter.galley(
        text_rect.left_center() - vec2(0., galley.size().y / 2.),
        galley,
        text_color,
    );

    let icon_rot = (1. - active_t) * -FRAC_PI_2;
    let icon = if toggled {
        icons::CaretDown::solid()
    } else {
        icons::CaretDown::outline()
    };
    let icon_rect = Rect::from_center_size(icon_rect.center(), vec2(10., 10.));
    let icon_color = ui.app_visuals().menu_icon_active_color;
    icon.to_image()
        .tint(icon_color)
        .fit_to_exact_size(icon_rect.size())
        .rotate(icon_rot, Vec2::splat(0.5))
        .paint_at(ui, icon_rect);

    response
}

fn section_controls(ui: &mut Ui) -> Response {
    let size = vec2(ui.available_width(), 25.);
    let (rect, response) = ui.allocate_exact_size(size, Sense::empty());

    if response.hovered() {
        let painter = ui.painter();
        let shadow_color = ui.app_visuals().menu_icon_shadow_color_hover;
        painter.rect_filled(rect.shrink2(vec2(0., 3.)), 0., shadow_color);
    }

    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.);
            ui.add_space(5.);
            ribbon_control(ui);
            let pos = ui.cursor().left_center();
            let text_color = ui.app_visuals().menu_icon_inactive_color;
            ui.painter().text(
                pos,
                Align2::LEFT_CENTER,
                "Add repository...".to_owned(),
                Figtree::medium().sized(13.),
                text_color,
            )
        },
    );

    response
}

fn ribbon_control(ui: &mut Ui) -> Response {
    let size = vec2(20., 20.);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(17.));
    let icon_color = ui.app_visuals().menu_icon_inactive_color;
    icons::Cloud::plus()
        .to_image()
        .tint(icon_color)
        .fit_to_exact_size(icon_rect.size())
        .paint_at(ui, icon_rect);

    response
}

fn panel_content(ui: &mut Ui) -> Response {
    Frame::new()
        .inner_margin(8.)
        .show(ui, |ui| {
            ui.label("Dataflow left panel");
        })
        .response
}
