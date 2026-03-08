mod animation;
pub mod containers;
pub mod style;
pub mod utils;
pub mod widgets;

pub use animation::AnimationExt;

pub trait ResponseExt {
    fn hovered_for(&self, duration: f32) -> bool;
}

impl ResponseExt for egui::Response {
    fn hovered_for(&self, duration: f32) -> bool {
        let id = self.id.with("hovered_for");
        if self.hovered() {
            let time = self.ctx.input(|i| i.time);
            let start: f64 = self.ctx.data_mut(|d| *d.get_temp_mut_or(id, time));
            let elapsed = self.ctx.input(|i| i.time) - start;
            self.ctx.request_repaint();
            elapsed >= duration as f64
        } else {
            self.ctx.data_mut(|d| d.remove_temp::<f64>(id));
            false
        }
    }
}
