mod checkbox;
mod icon_btn;
mod ribbon_toggle;
mod status_bar;
mod toggle;

pub use checkbox::Checkbox;
use egui::{Response, Ui};
pub use icon_btn::IconBtn;
pub use ribbon_toggle::RibbonToggle;
use segs_assets::icons::Icon;
pub use status_bar::{PaddedStatusBarButton, StatusBarButton, UnpaddedStatusBarButton};
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
