use egui::emath::{Align2, Vec2};

use egui::{
    Area, Color32, Context, CursorIcon, Frame, Id, InnerResponse, Key, Modifiers, Order, Popup, Rect, Response, Sense,
    Ui, UiBuilder, UiKind, pos2, vec2,
};
use segs_assets::icons;

use crate::style::CtxStyleExt;
use crate::widgets::Separator;
use crate::widgets::buttons::IconBtn;

const TOP_BAR_HEIGHT: f32 = 24.0;
const ICON_SIZE: Vec2 = vec2(24., 24.);
const FONT_SIZE: f32 = 15.0;

/// A modal dialog.
///
/// Similar to a [`crate::Window`] but centered and with a backdrop that
/// blocks input to the rest of the UI.
///
/// You can show multiple modals on top of each other. The topmost modal will always be
/// the most recently shown one.
/// If multiple modals are newly shown in the same frame, the order of the modals is undefined
/// (either first or second could be top).
pub struct Modal {
    pub id: Id,
    pub area: Area,
    pub backdrop_color: Color32,
    pub frame: Option<Frame>,
    pub title: String,
}

impl Modal {
    /// Create a new Modal.
    ///
    /// The id is passed to the area.
    pub fn new(id: Id, title: impl Into<String>) -> Self {
        Self {
            id,
            area: Self::default_area(id.with("_area")),
            backdrop_color: Color32::from_black_alpha(100),
            frame: None,
            title: title.into(),
        }
    }

    /// Returns an area customized for a modal.
    ///
    /// Makes these changes to the default area:
    /// - sense: hover
    /// - anchor: center
    /// - order: foreground
    pub fn default_area(id: Id) -> Area {
        Area::new(id)
            .kind(UiKind::Modal)
            .sense(Sense::hover())
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .order(Order::Foreground)
            .interactable(true)
    }

    /// Set the frame of the modal.
    ///
    /// Default is [`Frame::popup`].
    #[inline]
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    /// Set the backdrop color of the modal.
    ///
    /// Default is `Color32::from_black_alpha(100)`.
    #[inline]
    pub fn backdrop_color(mut self, color: Color32) -> Self {
        self.backdrop_color = color;
        self
    }

    /// Set the area of the modal.
    ///
    /// Default is [`Modal::default_area`].
    #[inline]
    pub fn area(mut self, area: Area) -> Self {
        self.area = area;
        self
    }

    /// Show the modal.
    pub fn show<T>(self, ctx: &Context, content: impl FnOnce(&mut Ui) -> T) -> ModalResponse<T> {
        let Self {
            id,
            area,
            backdrop_color,
            frame,
            title,
        } = self;

        let sizing_pass_id = id.with("_sizing_pass");
        let modal_rect_id = id.with("_rect");

        let (is_sizing_pass, modal_rect) = ctx.memory_mut(|mem| {
            let sizing = mem.data.get_temp(sizing_pass_id).unwrap_or(true);
            mem.data.insert_temp(sizing_pass_id, false);
            let rect = mem.data.get_temp(modal_rect_id).unwrap_or(Rect::ZERO);

            (sizing, rect)
        });

        let is_top_modal = ctx.memory_mut(|mem| {
            mem.set_modal_layer(area.layer());
            mem.top_modal_layer() == Some(area.layer())
        });
        let any_popup_open = Popup::is_any_open(ctx);
        let InnerResponse {
            inner: (inner, backdrop_response),
            response,
        } = area.sizing_pass(is_sizing_pass).show(ctx, |ui| {
            let bg_rect = ui.ctx().content_rect();
            let bg_sense = Sense::CLICK | Sense::DRAG;
            let mut backdrop = ui.new_child(UiBuilder::new().sense(bg_sense).max_rect(bg_rect));
            backdrop.set_min_size(bg_rect.size());
            ui.painter().rect_filled(bg_rect, 0.0, backdrop_color);
            let backdrop_response = backdrop.response();

            let frame = frame.unwrap_or_else(|| Frame::popup(ui.style()));

            let mut should_close = false;

            // We need the extra scope with the sense since frame can't have a sense and since we
            // need to prevent the clicks from passing through to the backdrop.
            let inner = ui
                .scope_builder(UiBuilder::new().id(id).sense(Sense::CLICK | Sense::DRAG), |ui| {
                    frame
                        .show(ui, |ui| {
                            let res = show_top_bar(ui, &modal_rect, title);
                            ctx.memory_mut(|mem| mem.data.insert_temp(modal_rect_id, res.rect));

                            ui.add(Separator::default().grow(ui.style().spacing.window_margin.leftf()));

                            // Add user content
                            let inner = content(ui);

                            if res.should_close() {
                                should_close = true;
                            }

                            inner
                        })
                        .inner
                })
                .inner;

            if should_close {
                ui.close();
            }

            (inner, backdrop_response)
        });

        if is_sizing_pass {
            ctx.memory_mut(|mem| mem.data.insert_temp(sizing_pass_id, false));
            ctx.request_discard("Modal sizing pass");
        }

        ModalResponse {
            response,
            backdrop_response,
            inner,
            is_top_modal,
            any_popup_open,
        }
    }
}

fn show_top_bar(ui: &mut Ui, rect: &Rect, title: String) -> Response {
    // Allocate space for the tob bar area
    let size = vec2(rect.width(), TOP_BAR_HEIGHT);
    let (rect, mut res) = ui.allocate_exact_size(size, Sense::empty());

    let painter = ui.painter();
    let text_color = ui.visuals().text_color();
    let app_style = ui.app_style();

    // Calculate space for the title text
    let galley = painter.layout_no_wrap(title, app_style.base_font_of(FONT_SIZE), text_color);
    let text_center = pos2(rect.min.x + (galley.size().x / 2.), rect.center().y);
    let text_rect = Rect::from_center_size(text_center, galley.size());
    // Paint the title text
    painter.galley(text_rect.min, galley, text_color);

    // Calculate space for the close button
    let close_center = pos2(rect.max.x - ICON_SIZE.x / 2., rect.center().y);
    let close_rect = Rect::from_center_size(close_center, ICON_SIZE);
    let close_btn = IconBtn::new(icons::X).with_size(ICON_SIZE);
    // Paint the close button
    let close_res = ui.place(close_rect, close_btn);
    // Handle close icon interactions
    if close_res.on_hover_cursor(CursorIcon::PointingHand).clicked() {
        res.set_close();
    }

    let total_width = text_rect.width() + close_rect.width() + 48.;
    ui.set_min_width(total_width);
    let rect = Rect::from_min_size(rect.min, vec2(total_width, rect.height()));

    res.with_new_rect(rect)
}

/// The response of a modal dialog.
pub struct ModalResponse<T> {
    /// The response of the modal contents
    pub response: Response,

    /// The response of the modal backdrop.
    ///
    /// A click on this means the user clicked outside the modal,
    /// in which case you might want to close the modal.
    pub backdrop_response: Response,

    /// The inner response from the content closure
    pub inner: T,

    /// Is this the topmost modal?
    pub is_top_modal: bool,

    /// Is there any popup open?
    /// We need to check this before the modal contents are shown, so we can know if any popup
    /// was open when checking if the escape key was clicked.
    pub any_popup_open: bool,
}

impl<T> ModalResponse<T> {
    /// Should the modal be closed?
    /// Returns true if:
    ///  - the backdrop was clicked
    ///  - this is the topmost modal, no popup is open and the escape key was pressed
    pub fn should_close(&self) -> bool {
        let ctx = &self.response.ctx;

        // this is a closure so that `Esc` is consumed only if the modal is topmost
        let escape_clicked = || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));

        let ui_close_called = self.response.should_close();

        ui_close_called || (self.is_top_modal && !self.any_popup_open && escape_clicked())
    }
}
