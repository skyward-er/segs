use std::sync::Arc;

use egui::{Color32, Galley, Pos2, Rect, Response, Sense, Theme, Vec2, Widget};
use segs_assets::icons::Icon;
use smallvec::SmallVec;

use crate::StyleExt;

const DEFAULT_PAD: f32 = 4.0;

#[allow(private_bounds)]
pub trait BottomBarButton: BottomBarButtonImpl + Sized {
    fn add_icon(mut self, icon: impl Icon + 'static) -> Self {
        self.contents().push(ContentAtoms::Image {
            size: icon.fit_size(Vec2::splat(15.)),
            image: Arc::new(icon),
        });
        self
    }

    fn add_icon_with_size(mut self, icon: impl Icon + 'static, size: Vec2) -> Self {
        self.contents().push(ContentAtoms::Image {
            size: icon.fit_size(size),
            image: Arc::new(icon),
        });
        self
    }

    fn add_text(mut self, text: impl Into<String>) -> Self {
        self.contents().push(ContentAtoms::Text(text.into()));
        self
    }
}

#[derive(Clone, Default)]
pub struct UnpaddedBottomBarButton {
    contents: Vec<ContentAtoms>,
}

#[derive(Clone)]
pub struct PaddedBottomBarButton {
    contents: Vec<ContentAtoms>,
    padding: f32,
}

#[derive(Clone)]
enum ContentAtoms {
    Image { image: Arc<dyn Icon>, size: Vec2 },
    Text(String),
    Space(f32),
}

impl BottomBarButton for UnpaddedBottomBarButton {}
impl BottomBarButton for PaddedBottomBarButton {}

trait BottomBarButtonImpl {
    fn contents(&mut self) -> &mut Vec<ContentAtoms>;
}

impl BottomBarButtonImpl for UnpaddedBottomBarButton {
    // No-op for unpadded button
    fn contents(&mut self) -> &mut Vec<ContentAtoms> {
        &mut self.contents
    }
}

impl BottomBarButtonImpl for PaddedBottomBarButton {
    fn contents(&mut self) -> &mut Vec<ContentAtoms> {
        &mut self.contents
    }
}

impl UnpaddedBottomBarButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_padding(self, padding: f32) -> PaddedBottomBarButton {
        PaddedBottomBarButton {
            contents: self.contents,
            padding,
        }
    }

    pub fn padded(self) -> PaddedBottomBarButton {
        PaddedBottomBarButton {
            contents: self.contents,
            padding: DEFAULT_PAD,
        }
    }

    pub fn add_space(mut self, space: f32) -> Self {
        self.contents.push(ContentAtoms::Space(space));
        self
    }
}

impl Widget for UnpaddedBottomBarButton {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        bottom_bar_btn(ui, self.contents)
    }
}

impl Widget for PaddedBottomBarButton {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        // Add padding between atoms
        let pad = ContentAtoms::Space(self.padding);
        for i in (1..self.contents.len()).rev() {
            self.contents.insert(i, pad.clone());
        }

        bottom_bar_btn(ui, self.contents)
    }
}

fn bottom_bar_btn(ui: &mut egui::Ui, atoms: Vec<ContentAtoms>) -> Response {
    // Define sizes
    let inner_margin = Vec2::new(4., 3.);

    // Extract atom sizes and allocate galleys for text
    let mut galleys = SmallVec::<[Arc<Galley>; 3]>::new();
    let mut sizes = SmallVec::<[Vec2; 6]>::new();
    for atom in atoms.iter() {
        match atom {
            ContentAtoms::Image { image: _, size } => {
                sizes.push(*size);
            }
            ContentAtoms::Text(text) => {
                let galley = ui.painter().layout_no_wrap(
                    text.clone(),
                    ui.app_style().base_font_of(13.0),
                    ui.visuals().text_color(),
                );
                sizes.push(galley.size());
                galleys.push(galley);
            }
            ContentAtoms::Space(space) => {
                sizes.push(Vec2::new(*space, 0.));
            }
        }
    }

    // Calculate total size
    let content_width: f32 = sizes.iter().map(|s| s.x).sum();
    let content_height: f32 = sizes.iter().map(|s| s.y).fold(0., f32::max);
    let btn_size = Vec2::new(content_width, content_height) + inner_margin * 2.0;

    // Allocate space for the button
    let (btn_rect, response) = ui.allocate_exact_size(btn_size, Sense::click());

    // Only paint if visible
    if ui.is_rect_visible(btn_rect) {
        let painter = ui.painter();

        // Paint shadow on hover
        if response.hovered() {
            let shadow_color = ui.app_visuals().shadow_color;
            painter.rect_filled(btn_rect, 0., shadow_color);
        }

        // Paint contents
        let mut x_offset = btn_rect.min.x + inner_margin.x;
        let v_center = btn_rect.center().y;
        let mut galley_iter = galleys.into_iter();
        for (atom, size) in atoms.into_iter().zip(sizes) {
            match atom {
                ContentAtoms::Image { image, size: _ } => {
                    let icon_pos = Pos2::new(x_offset, v_center - size.y / 2.0);
                    let tint = if ui.ctx().theme() == Theme::Dark {
                        Color32::WHITE
                    } else {
                        Color32::BLACK
                    };
                    image
                        .to_image()
                        .tint(tint)
                        .fit_to_exact_size(size)
                        .paint_at(ui, Rect::from_min_size(icon_pos, size));
                }
                ContentAtoms::Text(_) => {
                    let text_pos = Pos2::new(x_offset, v_center - size.y / 2.0);
                    painter.galley(text_pos, galley_iter.next().unwrap(), ui.visuals().text_color());
                }
                ContentAtoms::Space(_) => (), // No painting for space
            }
            x_offset += size.x;
        }
    }

    response
}
