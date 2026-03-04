use std::sync::Arc;

use egui::{Align, Color32, Direction, Galley, Pos2, Rect, Sense, Ui, Vec2, vec2};
use segs_assets::icons::Icon;
use smallvec::SmallVec;

use crate::style::CtxStyleExt;

pub struct Atoms {
    elements: SmallVec<[AtomKind; 8]>,
    main_dir: Direction,
    main_justified: bool,
    cross_align: Align,
    default_pad: f32,
}

enum AtomKind {
    Text(TextAtom),
    Icon(IconAtom),
    Pad(f32),
}

pub struct TextAtom {
    galley: Arc<Galley>,
    color: Color32,
}

pub struct IconAtom {
    icon: Box<dyn Icon>,
    size: f32,
    tint: Color32,
}

impl Atoms {
    pub fn left_to_right() -> Self {
        Self {
            elements: SmallVec::new(),
            main_dir: Direction::LeftToRight,
            main_justified: false,
            cross_align: Align::Center,
            default_pad: 4.,
        }
    }

    pub fn right_to_left() -> Self {
        Self {
            main_dir: Direction::RightToLeft,
            ..Default::default()
        }
    }

    pub fn top_down() -> Self {
        Self {
            main_dir: Direction::TopDown,
            ..Default::default()
        }
    }

    pub fn bottom_up() -> Self {
        Self {
            main_dir: Direction::BottomUp,
            ..Default::default()
        }
    }

    pub fn all_centered(mut self) -> Self {
        self.main_justified = true;
        self.cross_align = Align::Center;
        self
    }

    pub fn justified(mut self) -> Self {
        self.main_justified = true;
        self
    }

    pub fn with_cross_align(mut self, align: Align) -> Self {
        self.cross_align = align;
        self
    }

    pub fn with_pad(mut self, pad: f32) -> Self {
        self.default_pad = pad;
        self
    }

    pub fn place(mut self, ui: &mut Ui, rect: Rect, add_contents: impl FnOnce(&mut AtomsUi)) {
        let total_size = self.begin(ui, add_contents);

        let Self {
            mut elements,
            main_dir,
            main_justified,
            cross_align,
            ..
        } = self;

        // If justified, we need to add padding at the start and end to center the content
        if main_justified {
            let margin = match main_dir {
                Direction::LeftToRight | Direction::RightToLeft => (rect.width() - total_size.x) / 2.,
                Direction::TopDown | Direction::BottomUp => (rect.height() - total_size.y) / 2.,
            };
            elements.insert(0, AtomKind::Pad(margin));
            elements.push(AtomKind::Pad(margin));
        }

        let cursor = Cursor::new(main_dir, cross_align, &rect);
        Self::paint_elements(cursor, elements, ui);
    }

    pub fn show(mut self, ui: &mut Ui, add_contents: impl FnOnce(&mut AtomsUi)) {
        let total_size = self.begin(ui, add_contents);
        let (rect, _) = ui.allocate_exact_size(total_size, Sense::empty());

        let Self {
            elements,
            main_dir,
            cross_align,
            ..
        } = self;

        let cursor = Cursor::new(main_dir, cross_align, &rect);
        Self::paint_elements(cursor, elements, ui);
    }

    fn begin(&mut self, ui: &mut Ui, add_contents: impl FnOnce(&mut AtomsUi)) -> Vec2 {
        // First, populate elements with add_contents
        let mut atoms_ui = AtomsUi::new(self, ui);
        add_contents(&mut atoms_ui);

        // Pad elements with default pad
        self.pad_elements();

        // Now that we have all the elements, we can compute the size
        self.total_size()
    }

    fn paint_elements(mut cursor: Cursor, elements: SmallVec<[AtomKind; 8]>, ui: &mut Ui) {
        for elem in elements {
            let size = elem.size();
            let rect = cursor.advance(size);
            elem.paint_at(ui, rect);
        }
    }

    fn total_size(&self) -> Vec2 {
        self.elements
            .iter()
            .map(|elem| elem.size())
            .fold(vec2(0., 0.), |acc, size| match self.main_dir {
                egui::Direction::LeftToRight | egui::Direction::RightToLeft => vec2(acc.x + size.x, acc.y.max(size.y)),
                egui::Direction::TopDown | egui::Direction::BottomUp => vec2(acc.x.max(size.x), acc.y + size.y),
            })
    }

    fn pad_elements(&mut self) {
        let pad = self.default_pad;
        for i in (1..self.elements.len()).rev() {
            if !matches!(self.elements[i - 1], AtomKind::Pad(_)) && !matches!(self.elements[i], AtomKind::Pad(_)) {
                self.elements.insert(i, AtomKind::Pad(pad));
            }
        }
    }
}

struct Cursor {
    main_dir: Direction,
    cross_align: Align,
    pos: Pos2,
}

impl Cursor {
    fn new(main_dir: Direction, cross_align: Align, rect: &Rect) -> Self {
        let pos = match (main_dir, cross_align) {
            (Direction::LeftToRight, Align::TOP) => rect.left_top(),
            (Direction::LeftToRight, Align::Center) => rect.left_center(),
            (Direction::LeftToRight, Align::BOTTOM) => rect.left_bottom(),
            (Direction::RightToLeft, Align::TOP) => rect.right_top(),
            (Direction::RightToLeft, Align::Center) => rect.right_center(),
            (Direction::RightToLeft, Align::BOTTOM) => rect.right_bottom(),
            (Direction::TopDown, Align::LEFT) => rect.left_top(),
            (Direction::TopDown, Align::Center) => rect.center_top(),
            (Direction::TopDown, Align::RIGHT) => rect.right_top(),
            (Direction::BottomUp, Align::LEFT) => rect.left_bottom(),
            (Direction::BottomUp, Align::Center) => rect.center_bottom(),
            (Direction::BottomUp, Align::RIGHT) => rect.right_bottom(),
        };
        Self {
            main_dir,
            cross_align,
            pos,
        }
    }

    fn advance(&mut self, size: Vec2) -> Rect {
        match (self.main_dir, self.cross_align) {
            (Direction::LeftToRight, Align::TOP) => {
                let rect = Rect::from_min_size(self.pos, size);
                self.pos.x += size.x;
                rect
            }
            (Direction::LeftToRight, Align::Center) => {
                let rect = Rect::from_center_size(self.pos + vec2(size.x / 2., 0.), size);
                self.pos.x += size.x;
                rect
            }
            (Direction::LeftToRight, Align::BOTTOM) => {
                let rect = Rect::from_min_max(self.pos - vec2(0., size.y), self.pos + vec2(size.x, 0.));
                self.pos.x += size.x;
                rect
            }
            (Direction::RightToLeft, Align::TOP) => {
                let rect = Rect::from_min_size(self.pos - vec2(size.x, 0.), size);
                self.pos.x -= size.x;
                rect
            }
            (Direction::RightToLeft, Align::Center) => {
                let rect = Rect::from_center_size(self.pos - vec2(size.x / 2., 0.), size);
                self.pos.x -= size.x;
                rect
            }
            (Direction::RightToLeft, Align::BOTTOM) => {
                let rect = Rect::from_min_max(self.pos - vec2(size.x, size.y), self.pos);
                self.pos.x -= size.x;
                rect
            }
            (Direction::TopDown, Align::LEFT) => {
                let rect = Rect::from_min_size(self.pos, size);
                self.pos.y += size.y;
                rect
            }
            (Direction::TopDown, Align::Center) => {
                let rect = Rect::from_center_size(self.pos + vec2(0., size.y / 2.), size);
                self.pos.y += size.y;
                rect
            }
            (Direction::TopDown, Align::RIGHT) => {
                let rect = Rect::from_min_max(self.pos - vec2(size.x, 0.), self.pos + vec2(0., size.y));
                self.pos.y += size.y;
                rect
            }
            (Direction::BottomUp, Align::LEFT) => {
                let rect = Rect::from_min_size(self.pos - vec2(0., size.y), size);
                self.pos.y -= size.y;
                rect
            }
            (Direction::BottomUp, Align::Center) => {
                let rect = Rect::from_center_size(self.pos - vec2(0., size.y / 2.), size);
                self.pos.y -= size.y;
                rect
            }
            (Direction::BottomUp, Align::RIGHT) => {
                let rect = Rect::from_min_max(self.pos - vec2(size.x, size.y), self.pos);
                self.pos.y -= size.y;
                rect
            }
        }
    }
}

impl AtomKind {
    fn size(&self) -> egui::Vec2 {
        match self {
            AtomKind::Text(text) => text.size(),
            AtomKind::Icon(icon) => icon.size(),
            AtomKind::Pad(pad) => vec2(*pad, *pad),
        }
    }

    fn paint_at(self, ui: &mut Ui, rect: Rect) {
        match self {
            AtomKind::Text(text) => text.paint_at(ui, rect),
            AtomKind::Icon(icon) => icon.paint_at(ui, rect),
            AtomKind::Pad(_) => { /* No painting needed for padding */ }
        }
    }
}

impl TextAtom {
    fn size(&self) -> egui::Vec2 {
        self.galley.size()
    }

    fn paint_at(self, ui: &mut Ui, rect: Rect) {
        let Self { galley, color } = self;
        ui.painter().galley(rect.left_top(), galley, color);
    }
}

impl IconAtom {
    fn size(&self) -> egui::Vec2 {
        vec2(self.size, self.size)
    }

    /// Assumption: Rect must be square
    fn paint_at(self, ui: &mut Ui, rect: Rect) {
        let Self { icon, tint, .. } = self;
        icon.to_image()
            .tint(tint)
            .fit_to_exact_size(rect.size())
            .paint_at(ui, rect);
    }
}

pub struct AtomsUi<'a> {
    ui: &'a mut Ui,
    atoms: &'a mut Atoms,
}

trait AtomBuilder {
    fn build(self, ui: &mut Ui) -> AtomKind;
}

impl<'a> AtomsUi<'a> {
    fn new(atoms: &'a mut Atoms, ui: &'a mut Ui) -> Self {
        Self { ui, atoms }
    }

    #[allow(private_bounds)]
    pub fn add(&mut self, atom: impl AtomBuilder) -> &mut Self {
        let atom_kind = atom.build(self.ui);
        self.atoms.elements.push(atom_kind);
        self
    }

    pub fn text(text: impl Into<String>) -> TextAtomBuilder {
        TextAtomBuilder::new(text.into())
    }

    pub fn icon(icon: impl Icon + 'static, size: f32) -> IconAtomBuilder {
        IconAtomBuilder::new(icon, size)
    }

    pub fn add_pad(&mut self, pad: f32) -> &mut Self {
        self.atoms.elements.push(AtomKind::Pad(pad));
        self
    }
}

pub struct TextAtomBuilder {
    text: String,
    text_size: f32,
    color: Option<Color32>,
}

pub struct IconAtomBuilder {
    icon: Box<dyn Icon>,
    size: f32,
    tint: Option<Color32>,
}

impl TextAtomBuilder {
    fn new(text: String) -> Self {
        Self {
            text,
            text_size: 12.,
            color: None,
        }
    }

    pub fn with_text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl IconAtomBuilder {
    fn new(icon: impl Icon + 'static, size: f32) -> Self {
        Self {
            icon: Box::new(icon),
            size,
            tint: None,
        }
    }

    pub fn with_tint(mut self, tint: Color32) -> Self {
        self.tint = Some(tint);
        self
    }
}

impl AtomBuilder for TextAtomBuilder {
    fn build(self, ui: &mut Ui) -> AtomKind {
        let Self { text, text_size, color } = self;
        let stroke_color = color.unwrap_or_else(|| ui.visuals().text_color());
        let galley = ui
            .painter()
            .layout_no_wrap(text, ui.app_style().base_font_of(text_size), stroke_color);
        AtomKind::Text(TextAtom {
            galley,
            color: stroke_color,
        })
    }
}

impl AtomBuilder for IconAtomBuilder {
    fn build(self, ui: &mut Ui) -> AtomKind {
        let Self { icon, size, tint } = self;
        let tint_color = tint.unwrap_or(ui.visuals().text_color());
        AtomKind::Icon(IconAtom {
            icon,
            size,
            tint: tint_color,
        })
    }
}

impl Default for Atoms {
    fn default() -> Self {
        Self::left_to_right()
    }
}
