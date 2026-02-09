mod ui;
mod utils;

use eframe::egui;
use egui::{
    Align, Align2, Area, Color32, CursorIcon, Frame, Id, Layout, Margin, Pos2, Rect, Response, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, ViewportBuilder, emath::easing, lerp, pos2, vec2,
};
use mimalloc::MiMalloc;
use segs_assets::{
    Font,
    fonts::Figtree,
    icons::{self, Icon},
    install_fonts, install_icons, load_app_icon,
};
use segs_memory::{MemoryExt, init_memory};
use segs_ui::{
    StyleExt,
    containers::ResizablePanel,
    setup_style,
    widgets::{UiWidgetExt, labels::SelectableLabel},
};

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
    eframe::run_native("SEGS v2", options, Box::new(|cc| Ok(Box::new(MyApp::new(cc)))))
}

struct MyApp {
    top_bar_controls: TopBarControls,
    bottom_bar_controls: BottomBarControls,
    side_panel_selection: Selection,
    test_flag: bool,
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
            side_panel_selection: Selection::None,
            test_flag: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Selection {
    None,
    Layout,
    Charts,
    Commands,
    Settings,
}

impl Selection {}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::panels::top_controls_bar(ctx, &mut self.top_bar_controls);
        ui::panels::bottom_controls_bar(ctx, &mut self.bottom_bar_controls);

        // if self.top_bar_controls.left_panel_visible {
        //     egui::SidePanel::left("left_panel")
        //         .max_width(50.)
        //         .show_separator_line(true)
        //         .resizable(false)
        //         .frame(Frame::new().fill(ctx.style().visuals.panel_fill))
        //         .show(ctx, |ui| {
        //             ui.vertical(|ui| {
        //                 // Top section (empty, but you can add content here)
        //                 ui.with_layout(Layout::top_down(Align::Center), |ui| {
        //                     ui.spacing_mut().item_spacing = Vec2::ZERO;
        //                     side_selector(ui, &mut self.side_panel_selection,
        // Selection::Charts, icons::Charts);
        // side_selector(ui, &mut self.side_panel_selection, Selection::Layout,
        // icons::Layout);                 });

        //                 // Bottom section (your buttons)
        //                 ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        //                     ui.spacing_mut().item_spacing = Vec2::ZERO;
        //                     side_selector(ui, &mut self.side_panel_selection,
        // Selection::Settings, icons::Cog);                 });
        //             });
        //         });
        // }

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     SelectableLabel::new(&mut self.side_panel_selection)
        //         .option(icons::Ethernet, "Ethernet", Selection::Charts)
        //         .option(icons::Usb, "Usb", Selection::Layout)
        //         .show(ui);
        //     ui.add_space(10.);
        //     ui.check(&mut self.test_flag);

        //     let sin: segs_plot::PlotPoints = (0..1000)
        //         .map(|i| {
        //             let x = i as f64 * 0.01;
        //             [x, x.sin()]
        //         })
        //         .collect();
        //     let line = segs_plot::Line::new("sin", sin);
        //     segs_plot::Plot::new("my_plot")
        //         .view_aspect(2.0)
        //         .legend(segs_plot::Legend::default())
        //         .coordinates_formatter(
        //             segs_plot::Corner::RightBottom,
        //             segs_plot::CoordinatesFormatter::new(|pos, _| format!("x: {:.2},
        // y: {:.2}", pos.x, pos.y)),         )
        //         .show(ui, |plot_ui| plot_ui.line(line));

        //     vertical_toggle(ui);

        //     // Overlay at bottom-right
        //     if self.bottom_bar_controls.notifications_active {
        //         Area::new("bottom_right_overlay".into())
        //             .pivot(Align2::RIGHT_BOTTOM)
        //             .fixed_pos(ui.max_rect().right_bottom())
        //             .show(ctx, |ui| {
        //                 notification_panel(ui);
        //             });
        //     }
        // });

        let frame = Frame::new().fill(ctx.style().visuals.panel_fill);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let mut collapsed_left = !self.top_bar_controls.left_panel_visible;
            let mut collapsed_right = !self.top_bar_controls.right_panel_visible;
            let panel_frame = Frame::NONE.fill(Color32::from_rgb(246, 246, 246));
            let main_frame = panel_frame
                .corner_radius(5.0)
                .stroke(Stroke::new(1., Color32::from_rgb(242, 242, 242)))
                .fill(Color32::from_rgb(252, 252, 252));
            let right = ResizablePanel::horizontal_right()
                .collapsed(&mut collapsed_right)
                .inactive_separator_width(0.)
                .right_frame(panel_frame);
            ResizablePanel::horizontal_left()
                .collapsed(&mut collapsed_left)
                .inactive_separator_width(0.)
                .left_frame(panel_frame)
                .right_frame(main_frame)
                .show(ui, |panel| {
                    panel
                        .show_left(|ui| {
                            ui.with_layout(Layout::top_down(Align::Min).with_main_wrap(true), |ui| {
                                ui.label("Left Panel");
                            });
                        })
                        // .show_right(|ui| {
                        //     ui.label("asdasd");
                        // });
                        .show_right(|ui| {
                            right.show(ui, |panel| {
                                panel
                                    .show_left(|ui| {
                                        ui.with_layout(Layout::top_down(Align::Min).with_main_wrap(true), |ui| {
                                            ui.label("Main Panel");
                                        });
                                    })
                                    .show_right(|ui| {
                                        ui.label("Right panel");
                                    });
                            });
                        });
                });
            self.top_bar_controls.left_panel_visible = !collapsed_left;
            self.top_bar_controls.right_panel_visible = !collapsed_right;
        });

        ctx.mem().sync_persistence().expect("Failed to sync persistent memory");
    }
}

fn vertical_toggle(ui: &mut Ui) {
    let width = 17.5;
    let height = 35.0;

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let id = ui.next_auto_id().with("toggle_asd");

        if response.clicked() {
            ui.data_mut(|data| {
                let flag: &mut bool = data.get_temp_mut_or_default(id);
                *flag = !*flag;
            });
        }

        // Change cursor on hover
        if ui.rect_contains_pointer(rect) {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        // Animation factor
        let active = ui.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
        let click_t = ui
            .ctx()
            .animate_bool_with_time_and_easing(id.with("anim"), active, 0.1, easing::cubic_in);

        // Paint background
        let accent_color = ui.app_visuals().accent_color;
        let fill_color = ui.visuals().widgets.inactive.bg_fill;
        let bg_color = fill_color.lerp_to_gamma(accent_color, click_t);
        let corner_radius = width / 2.0;
        painter.rect_filled(rect, corner_radius, bg_color);

        // Paint circle
        let off_y = rect.min.y + corner_radius;
        let on_y = rect.max.y - corner_radius;
        let y = lerp(off_y..=on_y, click_t);
        let x = rect.min.x + corner_radius;
        let center = Pos2::new(x, y);
        let radius = corner_radius - 2.0;

        let circle_color = ui.visuals().panel_fill;
        painter.circle_filled(center, radius, circle_color);

        let small_radius = radius / 4.0;
        let small_circle_color = circle_color.gamma_multiply(0.6);
        painter.circle_filled(Pos2::new(x, on_y), small_radius, small_circle_color);
        painter.circle_filled(Pos2::new(x, off_y), small_radius, small_circle_color);

        // Paint text
        let center_a = Pos2::new(rect.max.x + 5.0, off_y);
        let center_b = Pos2::new(rect.max.x + 5.0, on_y);
        let active_text_color = ui.visuals().text_color();
        let unactive_text_color = active_text_color.gamma_multiply(0.5);
        let base_font = ui.app_style().base_font_of(12.0);
        let bold_font = ui.app_style().bold_font_of(12.0);

        let text = "ETHERNET";
        let (font, text_color) = if click_t > 0.5 {
            (base_font.clone(), unactive_text_color)
        } else {
            (bold_font.clone(), active_text_color)
        };
        painter.text(center_a, Align2::LEFT_CENTER, text, font, text_color);
        let text = "SERIAL";
        let (font, text_color) = if click_t > 0.5 {
            (bold_font, active_text_color)
        } else {
            (base_font, unactive_text_color)
        };
        painter.text(center_b, Align2::LEFT_CENTER, text, font, text_color);
    }
}

fn notification_panel(ui: &mut Ui) {
    const WIDTH: f32 = 320.0;
    const MIN_HEIGHT: f32 = 120.0;
    const MAX_HEIGHT: f32 = 500.0;
    const MARGIN: f32 = 8.0;

    Frame::NONE
        .fill(ui.visuals().panel_fill)
        .stroke(ui.visuals().window_stroke())
        .corner_radius(8.0)
        .inner_margin(MARGIN)
        .show(ui, |ui| {
            ui.set_width(WIDTH);

            // Scrollable content
            ui.spacing_mut().scroll.active_background_opacity = 0.5;
            ui.spacing_mut().scroll.active_handle_opacity = 0.6;
            ui.spacing_mut().scroll.interact_handle_opacity = 0.8;
            ui.spacing_mut().scroll.bar_width = 5.0;
            ScrollArea::vertical()
                .max_height(MAX_HEIGHT)
                .auto_shrink([false, true]) // Shrink vertically, not horizontally
                .show(ui, |ui| {
                    ui.set_min_height(MIN_HEIGHT);
                    ui.set_width(WIDTH - 10.0); // Full width minus margins

                    notifications_list(ui);
                });
        });
}

fn notifications_list(ui: &mut Ui) {
    let notifications = vec![
        (
            "System Update",
            "A new system update is available. Click here to install.",
            "2m ago",
        ),
        (
            "Message Received",
            "You have received a new message from John: Hey, are you available for a meeting tomorrow?",
            "5m ago",
        ),
        (
            "Task Completed",
            "Your export task has completed successfully.",
            "10m ago",
        ),
        (
            "Warning",
            "Disk space is running low. Please free up some space.",
            "1h ago",
        ),
        (
            "Information",
            "This is a very long notification message that will wrap to multiple lines to demonstrate how text wrapping works in the notification center. It should handle long text gracefully.",
            "2h ago",
        ),
    ];

    for (i, (title, message, time)) in notifications.iter().enumerate() {
        notification_card(ui, title, message, time);

        if i < notifications.len() - 1 {
            ui.add_space(6.0);
        }
    }
}

fn notification_card(ui: &mut Ui, title: &str, message: &str, time: &str) {
    let (_rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 0.0), // Width known, height unknown
        Sense::click(),
    );

    Frame::NONE
        .fill(ui.visuals().extreme_bg_color)
        .stroke(if response.hovered() {
            ui.visuals().widgets.hovered.bg_stroke
        } else {
            Stroke::NONE
        })
        .corner_radius(6.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.vertical(|ui| {
                // Title and time
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(time).small().weak());
                    });
                });

                ui.add_space(4.0);

                // Message with wrapping - this is key!
                ui.label(RichText::new(message).color(ui.visuals().text_color()));
            });
        });

    if response.clicked() {
        println!("Notification clicked: {}", title);
    }
}

fn side_selector<I: Icon>(ui: &mut Ui, panel_selection: &mut Selection, if_selected: Selection, icon: I) {
    let mut active = *panel_selection == if_selected;
    let response = side_button(ui, icon, &mut active);
    if active {
        *panel_selection = if_selected;
    } else if response.clicked() {
        *panel_selection = Selection::None;
    }
}

fn side_button<I: Icon>(ui: &mut Ui, icon: I, active: &mut bool) -> Response {
    let btn_size = Vec2::new(50., 50.);

    let (rect, response) = ui.allocate_exact_size(btn_size, Sense::click());

    // Paint the button
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Paint left border on hover
        if *active {
            let border_rect = Rect::from_min_max(
                Pos2::new(rect.min.x, rect.min.y),
                Pos2::new(rect.min.x + 3.0, rect.max.y),
            );
            painter.rect_filled(border_rect, 0.0, ui.visuals().text_color());
        }

        // Toggle active state on click
        if response.clicked() {
            *active = !*active;
        }

        // Paint icon
        let icon_rect = rect.shrink(11.);
        let snapped_rect = snap_rect_to_pixels(icon_rect, ui.ctx().pixels_per_point());

        let image = icon.to_image();
        let mut icon_color = ui.visuals().text_color();
        if !response.hovered() && !*active {
            icon_color = icon_color.gamma_multiply(0.5);
        }

        // Get the appropriate texture and tint based on active state
        image
            .tint(icon_color)
            .fit_to_exact_size(snapped_rect.size())
            .paint_at(ui, snapped_rect);
    }

    response
}

fn snap_rect_to_pixels(rect: Rect, pixels_per_point: f32) -> Rect {
    let min_px = Pos2::new(rect.min.x * pixels_per_point, rect.min.y * pixels_per_point);
    let max_px = Pos2::new(rect.max.x * pixels_per_point, rect.max.y * pixels_per_point);

    let min_px = Pos2::new(min_px.x.round(), min_px.y.round());
    let max_px = Pos2::new(max_px.x.round(), max_px.y.round());

    Rect::from_min_max(
        Pos2::new(min_px.x / pixels_per_point, min_px.y / pixels_per_point),
        Pos2::new(max_px.x / pixels_per_point, max_px.y / pixels_per_point),
    )
}
