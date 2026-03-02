#[cfg(host_family = "windows")]
macro_rules! PATH_SEPARATOR {
    () => {
        r"\"
    };
}
#[cfg(not(host_family = "windows"))]
macro_rules! PATH_SEPARATOR {
    () => {
        r"/"
    };
}

macro_rules! asset_path {
    ($segment:expr) => {
        $segment
    };
    ($segment:expr, $($rest:expr),+ $(,)?) => {
        concat!($segment, PATH_SEPARATOR!(), asset_path!($($rest),+))
    };
}

macro_rules! include_asset {
    ($($segment:expr),+ $(,)?) => {
        include_bytes!(asset_path!("..", "assets", $($segment),+))
    };
}
macro_rules! include_svg {
    ($($segment:expr),+ $(,)?) => {
        egui::include_image!(asset_path!("..", "assets", $($segment),+))
    };
}

// ~~~ START AUTOMATICALLY GENERATED CODE ~~~
#[rustfmt::skip]
pub mod fonts {
    pub static FIGTREE_BLACK: &[u8] = include_asset!("fonts", "Figtree-Black.ttf");
    pub static FIGTREE_BLACK_ITALIC: &[u8] = include_asset!("fonts", "Figtree-BlackItalic.ttf");
    pub static FIGTREE_BOLD: &[u8] = include_asset!("fonts", "Figtree-Bold.ttf");
    pub static FIGTREE_BOLD_ITALIC: &[u8] = include_asset!("fonts", "Figtree-BoldItalic.ttf");
    pub static FIGTREE_EXTRA_BOLD: &[u8] = include_asset!("fonts", "Figtree-ExtraBold.ttf");
    pub static FIGTREE_EXTRA_BOLD_ITALIC: &[u8] = include_asset!("fonts", "Figtree-ExtraBoldItalic.ttf");
    pub static FIGTREE_ITALIC: &[u8] = include_asset!("fonts", "Figtree-Italic.ttf");
    pub static FIGTREE_LIGHT: &[u8] = include_asset!("fonts", "Figtree-Light.ttf");
    pub static FIGTREE_LIGHT_ITALIC: &[u8] = include_asset!("fonts", "Figtree-LightItalic.ttf");
    pub static FIGTREE_MEDIUM: &[u8] = include_asset!("fonts", "Figtree-Medium.ttf");
    pub static FIGTREE_MEDIUM_ITALIC: &[u8] = include_asset!("fonts", "Figtree-MediumItalic.ttf");
    pub static FIGTREE_REGULAR: &[u8] = include_asset!("fonts", "Figtree-Regular.ttf");
    pub static FIGTREE_SEMI_BOLD: &[u8] = include_asset!("fonts", "Figtree-SemiBold.ttf");
    pub static FIGTREE_SEMI_BOLD_ITALIC: &[u8] = include_asset!("fonts", "Figtree-SemiBoldItalic.ttf");
}

#[rustfmt::skip]
pub mod icons {
    pub static SEGS_1024X1024: &[u8] = include_asset!("icons", "SEGS-1024x1024.png");
}

#[rustfmt::skip]
pub mod svgs {
    pub static ADD_FILE: egui::ImageSource = include_svg!("svgs", "add_file.svg");
    pub static ADD_FOLDER: egui::ImageSource = include_svg!("svgs", "add_folder.svg");
    pub static ADD_LAYOUT: egui::ImageSource = include_svg!("svgs", "add_layout.svg");
    pub static ALERT: egui::ImageSource = include_svg!("svgs", "alert.svg");
    pub static ANTENNA_OUTLINE: egui::ImageSource = include_svg!("svgs", "antenna_outline.svg");
    pub static ANTENNA_SOLID: egui::ImageSource = include_svg!("svgs", "antenna_solid.svg");
    pub static ARCHIVE_OUTLINE: egui::ImageSource = include_svg!("svgs", "archive_outline.svg");
    pub static ARCHIVE_SOLID: egui::ImageSource = include_svg!("svgs", "archive_solid.svg");
    pub static ARROW_DOWN: egui::ImageSource = include_svg!("svgs", "arrow_down.svg");
    pub static ARROW_UP: egui::ImageSource = include_svg!("svgs", "arrow_up.svg");
    pub static BELL_OUTLINE: egui::ImageSource = include_svg!("svgs", "bell_outline.svg");
    pub static BELL_SOLID: egui::ImageSource = include_svg!("svgs", "bell_solid.svg");
    pub static BOTTOM_PANEL_OUTLINE: egui::ImageSource = include_svg!("svgs", "bottom_panel_outline.svg");
    pub static BOTTOM_PANEL_SOLID: egui::ImageSource = include_svg!("svgs", "bottom_panel_solid.svg");
    pub static CHARTS_OUTLINE: egui::ImageSource = include_svg!("svgs", "charts_outline.svg");
    pub static CHARTS_SOLID: egui::ImageSource = include_svg!("svgs", "charts_solid.svg");
    pub static CIRCLED_CROSS: egui::ImageSource = include_svg!("svgs", "circled_cross.svg");
    pub static CLOUD_OUTLINE: egui::ImageSource = include_svg!("svgs", "cloud_outline.svg");
    pub static CLOUD_SOLID: egui::ImageSource = include_svg!("svgs", "cloud_solid.svg");
    pub static COG: egui::ImageSource = include_svg!("svgs", "cog.svg");
    pub static DOCUMENTS: egui::ImageSource = include_svg!("svgs", "documents.svg");
    pub static ETHERNET_PORT: egui::ImageSource = include_svg!("svgs", "ethernet-port.svg");
    pub static EYE_CLOSED: egui::ImageSource = include_svg!("svgs", "eye_closed.svg");
    pub static EYE_CROSSED: egui::ImageSource = include_svg!("svgs", "eye_crossed.svg");
    pub static EYE_OPEN: egui::ImageSource = include_svg!("svgs", "eye_open.svg");
    pub static FOLD_ALL: egui::ImageSource = include_svg!("svgs", "fold_all.svg");
    pub static FUNCTION_OUTLINE: egui::ImageSource = include_svg!("svgs", "function_outline.svg");
    pub static FUNCTION_SOLID: egui::ImageSource = include_svg!("svgs", "function_solid.svg");
    pub static GAUGE: egui::ImageSource = include_svg!("svgs", "gauge.svg");
    pub static GLOBE: egui::ImageSource = include_svg!("svgs", "globe.svg");
    pub static LAYOUT_GRID_OUTLINE: egui::ImageSource = include_svg!("svgs", "layout_grid_outline.svg");
    pub static LAYOUT_GRID_SOLID: egui::ImageSource = include_svg!("svgs", "layout_grid_solid.svg");
    pub static LAYOUT_OUTLINE: egui::ImageSource = include_svg!("svgs", "layout_outline.svg");
    pub static LAYOUT_SOLID: egui::ImageSource = include_svg!("svgs", "layout_solid.svg");
    pub static LEFT_PANEL_OUTLINE: egui::ImageSource = include_svg!("svgs", "left_panel_outline.svg");
    pub static LEFT_PANEL_SOLID: egui::ImageSource = include_svg!("svgs", "left_panel_solid.svg");
    pub static LENS: egui::ImageSource = include_svg!("svgs", "lens.svg");
    pub static LIGHTNING: egui::ImageSource = include_svg!("svgs", "lightning.svg");
    pub static LOCK_LOCKED: egui::ImageSource = include_svg!("svgs", "lock_locked.svg");
    pub static LOCK_UNLOCKED: egui::ImageSource = include_svg!("svgs", "lock_unlocked.svg");
    pub static MOON_OUTLINE: egui::ImageSource = include_svg!("svgs", "moon_outline.svg");
    pub static MOON_SOLID: egui::ImageSource = include_svg!("svgs", "moon_solid.svg");
    pub static NETWORK: egui::ImageSource = include_svg!("svgs", "network.svg");
    pub static PALETTE: egui::ImageSource = include_svg!("svgs", "palette.svg");
    pub static PLOTS: egui::ImageSource = include_svg!("svgs", "plots.svg");
    pub static PULSE: egui::ImageSource = include_svg!("svgs", "pulse.svg");
    pub static RECTANGLE_VERTICAL_OUTLINE: egui::ImageSource = include_svg!("svgs", "rectangle_vertical_outline.svg");
    pub static RECTANGLE_VERTICAL_SOLID: egui::ImageSource = include_svg!("svgs", "rectangle_vertical_solid.svg");
    pub static REFRESH: egui::ImageSource = include_svg!("svgs", "refresh.svg");
    pub static RETICLE_EMPTY: egui::ImageSource = include_svg!("svgs", "reticle_empty.svg");
    pub static RETICLE_OUTLINE: egui::ImageSource = include_svg!("svgs", "reticle_outline.svg");
    pub static RETICLE_SOLID: egui::ImageSource = include_svg!("svgs", "reticle_solid.svg");
    pub static RIGHT_PANEL_OUTLINE: egui::ImageSource = include_svg!("svgs", "right_panel_outline.svg");
    pub static RIGHT_PANEL_SOLID: egui::ImageSource = include_svg!("svgs", "right_panel_solid.svg");
    pub static SQUARE_ROTATED_SOLID: egui::ImageSource = include_svg!("svgs", "square_rotated_solid.svg");
    pub static STACK_OUTLINE: egui::ImageSource = include_svg!("svgs", "stack_outline.svg");
    pub static STACK_SOLID: egui::ImageSource = include_svg!("svgs", "stack_solid.svg");
    pub static STAR_OUTLINE: egui::ImageSource = include_svg!("svgs", "star_outline.svg");
    pub static STAR_SOLID: egui::ImageSource = include_svg!("svgs", "star_solid.svg");
    pub static SUN_OUTLINE: egui::ImageSource = include_svg!("svgs", "sun_outline.svg");
    pub static SUN_SOLID: egui::ImageSource = include_svg!("svgs", "sun_solid.svg");
    pub static TAG: egui::ImageSource = include_svg!("svgs", "tag.svg");
    pub static TOOLS: egui::ImageSource = include_svg!("svgs", "tools.svg");
    pub static USB: egui::ImageSource = include_svg!("svgs", "usb.svg");
    pub static WINDOW_OUTLINE: egui::ImageSource = include_svg!("svgs", "window_outline.svg");
    pub static WINDOW_SOLID: egui::ImageSource = include_svg!("svgs", "window_solid.svg");
}
// ~~~ END AUTOMATICALLY GENERATED CODE ~~~
