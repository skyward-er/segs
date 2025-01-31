mod grid;
mod svg;

use super::PaneBehavior;
use crate::ui::composable_view::PaneResponse;
use egui::{
    epaint::ImageDelta, Color32, Context, CursorIcon, PointerButton, Pos2, Rect, Sense,
    TextureOptions, Theme, Ui,
};
use egui_tiles::TileId;
use grid::Grid;
use resvg::tiny_skia::IntSize;
use serde::{Deserialize, Serialize};
use svg::Svg;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Pid2 {
    svg: Svg,
    grid: Grid,

    #[serde(skip)]
    editable: bool,

    center_content: bool,

    cache_valid: bool,
    texture_id: Option<egui::epaint::TextureId>,

    selected_element: Option<usize>,
}

impl PaneBehavior for Pid2 {
    fn ui(&mut self, ui: &mut egui::Ui, _: TileId) -> PaneResponse {
        let theme = Self::find_theme(ui.ctx());

        self.grid.draw(ui, theme);
        self.draw_svg(ui);

        let (_, response) = ui.allocate_at_least(ui.max_rect().size(), Sense::click_and_drag());
        if let Some(pos) = response.hover_pos().map(|p| self.grid.screen_to_grid(p)) {
            if let Some((idx, elem)) = self
                .svg
                .iter_mut_elements()
                .enumerate()
                .find(|(_, e)| e.hovered(pos))
            {
                // Handle hovering
                if elem.draggable() {
                    ui.ctx().output_mut(|o| o.cursor_icon = CursorIcon::Grab);
                }

                // Handle drag
                if response.drag_started_by(PointerButton::Primary) {
                    println!("Drag started on {}", elem.who_am_i());
                } else if response.drag_stopped_by(PointerButton::Primary) {
                    println!("Drag stopped on {}", elem.who_am_i());
                }

                // Handle clicks
                if response.clicked_by(PointerButton::Secondary) {
                    self.selected_element = Some(idx);
                }
            }
        }

        // Handle context menu
        if let Some(idx) = self.selected_element {
            response.context_menu(|ui| {
                if ui.button("Delete").clicked() {
                    println!("We need to delete {idx}");
                }
            });
        }

        PaneResponse::default()
    }

    fn contains_pointer(&self) -> bool {
        false
    }
}

impl Pid2 {
    /// Returns the currently used theme
    fn find_theme(ctx: &Context) -> Theme {
        // In Egui you can either decide a theme or use the system one.
        // If the system theme cannot be determined, a fallback theme can be set.
        ctx.options(|options| match options.theme_preference {
            egui::ThemePreference::Light => Theme::Light,
            egui::ThemePreference::Dark => Theme::Dark,
            egui::ThemePreference::System => match ctx.system_theme() {
                Some(Theme::Light) => Theme::Light,
                Some(Theme::Dark) => Theme::Dark,
                None => options.fallback_theme,
            },
        })
    }

    pub fn from_file() -> Self {
        let test = String::from_utf8(std::fs::read("test_assets/simple_pid.svg").unwrap()).unwrap();
        let mut des = quick_xml::de::Deserializer::from_str(&test);
        Self {
            svg: Svg::deserialize(&mut des).unwrap(),
            grid: Grid::from_size(50.0),
            editable: false,
            center_content: false,
            cache_valid: false,
            texture_id: None,
            selected_element: None,
        }
    }

    fn draw_svg(&mut self, ui: &mut Ui) {
        let texture_id = match self.texture_id {
            Some(texture_id) => {
                if !self.cache_valid {
                    let image = self.rasterize_svg().unwrap();
                    ui.ctx().tex_manager().write().set(
                        texture_id,
                        ImageDelta::full(image, TextureOptions::default()),
                    );
                    self.cache_valid = true;
                }
                texture_id
            }
            None => {
                let image = self.rasterize_svg().unwrap();
                let texture_id = ui.ctx().tex_manager().write().alloc(
                    "pid".to_string(),
                    image.into(),
                    TextureOptions::default(),
                );
                println!(
                    "Texture meta: {:?}",
                    ui.ctx().tex_manager().read().meta(texture_id)
                );
                self.texture_id = Some(texture_id);
                self.cache_valid = true;
                texture_id
            }
        };
        // egui::Image::from_texture((
        //     texture_id,
        //     egui::Vec2::new(
        //         self.svg.width * self.grid.size(),
        //         self.svg.height * self.grid.size(),
        //     ),
        // ))
        // .paint_at(
        //     ui,
        //     Rect::from_min_size(
        //         Pos2::new(0.0, 0.0),
        //         egui::Vec2::new(
        //             self.svg.width * self.grid.size(),
        //             self.svg.height * self.grid.size(),
        //         ),
        //     ),
        // );

        let painter = ui.painter();
        let rect = Rect::from_min_size(
            Pos2::new(0.0, 0.0),
            egui::Vec2::new(
                self.svg.width * self.grid.size(),
                self.svg.height * self.grid.size(),
            ),
        );
        painter.image(
            texture_id,
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    fn rasterize_svg(&self) -> Result<egui::ColorImage, String> {
        let mut serialized = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut serialized, Some("svg")).unwrap();
        self.svg.serialize(ser).unwrap();
        let svg_bytes: &[u8] = serialized.as_bytes();

        use resvg::tiny_skia::{Pixmap, Transform};
        use resvg::usvg::{Options, Tree};

        let mut opt = Options::default();
        opt.fontdb_mut().load_system_fonts();
        let rtree = Tree::from_data(svg_bytes, &opt).map_err(|err| err.to_string())?;
        let size = rtree.size().to_int_size();
        println!("Original svg size: {size:?}");

        let transform = Transform::from_scale(self.grid.size(), self.grid.size());
        let size = IntSize::from_wh(
            (transform.sx * size.width() as f32).ceil() as u32,
            (transform.sy * size.height() as f32).ceil() as u32,
        )
        .ok_or_else(|| format!("Failed to compute SVG size"))?;
        println!("Scaled svg size: {size:?}");
        let mut pixmap = Pixmap::new(size.width(), size.height())
            .ok_or_else(|| format!("Failed to create SVG Pixmap of size {size:?}"))?;
        resvg::render(&rtree, transform, &mut pixmap.as_mut());
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [size.width() as _, size.height() as _],
            pixmap.data(),
        );
        println!("Rasterized image: {image:?}");
        Ok(image)
    }
}
