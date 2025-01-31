mod icon;
mod svg;

use super::{PaneBehavior, PaneResponse};
use anyhow::{Context, Result};
use egui::{Pos2, Ui};
use egui_tiles::TileId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::string::String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Grid {
    pos: Pos2,
    scale: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            pos: Pos2::ZERO,
            scale: 10.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum Element {
    Icon,
    Label,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct ElementRef {
    id: String,
    pos: Pos2,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Pid3 {
    /// Scale and position of where to draw the diagram on screen
    grid: Grid,

    /// Elements that can be placed in the diagram
    elements: HashMap<String, Element>,

    /// Instances of elements
    references: Vec<ElementRef>,
}

impl PaneBehavior for Pid3 {
    fn ui(&mut self, ui: &mut Ui, _: TileId) -> PaneResponse {
        self.draw_diagram(ui);
        PaneResponse::default()
    }

    fn contains_pointer(&self) -> bool {
        false
    }
}

impl Pid3 {
    fn draw_diagram(&self, ui: &mut Ui) {
        let image = self.rasterize_svg().unwrap();
        let texture_id = ui.ctx().tex_manager().write().alloc(
            "pid".to_string(),
            image.into(),
            egui::TextureOptions::default(),
        );
        let rect = egui::Rect::from_min_size(
            Pos2::new(0.0, 0.0),
            egui::Vec2::new(10.0 * self.grid.scale, 10.0 * self.grid.scale),
        );
        ui.painter().image(
            texture_id,
            rect,
            egui::Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    fn rasterize_svg(&self) -> Result<egui::ColorImage> {
        // resvg uses the library roxmltree to represent internally the xml. The
        // problem is that roxmltree do not allow to build the document, only to
        // parse a text/file. For this reason we have to first serialize the svg
        // and then parse it to do the rasterization

        // Serialization
        let svg = svg::Svg::from(self);
        let mut buffer = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("svg"))?;
        svg.serialize(ser)?;

        // Parsing with usvg
        let mut options = resvg::usvg::Options::default();
        options.fontdb_mut().load_system_fonts(); // TODO: Do it once
        let rtree = resvg::usvg::Tree::from_str(buffer.as_str(), &options)?;
        let size = rtree.size().to_int_size();

        // Configure the scaling with the grid setting
        let transform = resvg::tiny_skia::Transform::from_scale(self.grid.scale, self.grid.scale);
        let size = resvg::tiny_skia::IntSize::from_wh(
            (transform.sx * size.width() as f32).ceil() as u32,
            (transform.sy * size.height() as f32).ceil() as u32,
        )
        .context("Failed to compute SVG size")?;

        // Rasterize
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .context("Failed to create SVG Pixmap of size {size:?}")?;
        resvg::render(&rtree, transform, &mut pixmap.as_mut());
        Ok(egui::ColorImage::from_rgba_unmultiplied(
            [size.width() as _, size.height() as _],
            pixmap.data(),
        ))
    }

    fn load_default_elements(&mut self) {}
}
