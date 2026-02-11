mod add_file;
mod add_folder;
mod add_layout;
mod antenna;
mod archive;
mod arrow;
mod bell;
mod charts;
mod cloud;
mod cog;
mod documents;
mod error;
mod ethernet;
mod eye;
mod fold_all;
mod globe;
mod layout;
mod lens;
mod lightning;
mod lock;
mod network;
mod palette;
mod panels;
mod plots;
mod pulse;
mod refresh;
mod reticle;
mod stack;
mod star;
mod tag;
mod themes;
mod usb;
mod warning;
mod window;

pub use add_file::AddFile;
pub use add_folder::AddFolder;
pub use add_layout::AddLayout;
pub use antenna::Antenna;
pub use archive::Archive;
pub use arrow::Arrow;
pub use bell::Bell;
pub use charts::Charts;
pub use cloud::Cloud;
pub use cog::Cog;
pub use documents::Documents;
use egui::{Image, ImageSource, Vec2};
pub use error::Error;
pub use ethernet::Ethernet;
pub use eye::Eye;
pub use fold_all::FoldAll;
pub use globe::Globe;
pub use layout::Layout;
pub use lens::Lens;
pub use lightning::Lightning;
pub use lock::Lock;
pub use network::Network;
pub use palette::Palette;
pub use panels::PanelToggle;
pub use plots::Plots;
pub use pulse::Pulse;
pub use refresh::Refresh;
pub use reticle::Reticle;
pub use stack::Stack;
pub use star::Star;
pub use tag::Tag;
pub use themes::{Moon, Sun};
pub use usb::Usb;
pub use warning::Warning;
pub use window::Window;

/// Icon representation. An icon can be converted to an egui Image and has a
/// svg source representation.
pub trait Icon {
    fn as_image_source(&self) -> &ImageSource<'static>;

    fn aspect_ratio(&self) -> f32 {
        1.0 // Assuming square icons by default
    }

    fn to_image(&self) -> Image<'static> {
        Image::new(self.as_image_source().clone())
    }

    /// Returns a size that fits within the given size while preserving the
    /// icon's aspect ratio.
    fn fit_size(&self, size: Vec2) -> Vec2 {
        let aspect_ratio = self.aspect_ratio();
        if size.x / size.y > aspect_ratio {
            Vec2::new(size.y * aspect_ratio, size.y)
        } else {
            Vec2::new(size.x, size.x / aspect_ratio)
        }
    }
}
