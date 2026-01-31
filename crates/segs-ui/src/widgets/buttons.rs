mod bottom_bar;
mod checkbox;
mod icon_btn;
mod toggle;

use egui::{Response, Ui};

use segs_assets::icons::Icon;

pub use bottom_bar::{BottomBarButton, PaddedBottomBarButton, UnpaddedBottomBarButton};
pub use checkbox::Checkbox;
pub use icon_btn::IconBtn;
pub use toggle::Toggle;

pub fn checkbox(ui: &mut Ui, active: &mut bool) -> Response {
    ui.add(Checkbox::new(active))
}

pub fn toggle(ui: &mut Ui, active: &mut bool) -> Response {
    ui.add(Toggle::new(active))
}

pub fn icon_btn(ui: &mut Ui, icon: impl Icon + 'static) -> Response {
    ui.add(IconBtn::new(icon))
}

pub fn icon_toggle(
    ui: &mut Ui,
    inactive_icon: impl Icon + 'static,
    active_icon: impl Icon + 'static,
    active: &mut bool,
) -> Response {
    ui.add(IconBtn::active(inactive_icon, active_icon, active))
}
