//! This module collects all the colors used in the UI, so that they can be
//! easily changed and maintained in one place.

use egui::Color32;

/// Panel fill, color of the background.
pub const LEVEL_0_COLOR_DARK: Color32 = Color32::from_rgb(9, 9, 9);
pub const LEVEL_0_COLOR_LIGHT: Color32 = Color32::from_rgb(240, 240, 240);
/// Color or collapsed panels, in the middle.
pub const LEVEL_1_COLOR_DARK: Color32 = Color32::from_rgb(16, 16, 17);
pub const LEVEL_1_COLOR_LIGHT: Color32 = Color32::from_rgb(249, 249, 249);
/// Color of the main view, the layer above panels.
pub const LEVEL_2_COLOR_DARK: Color32 = Color32::from_rgb(18, 18, 19);
pub const LEVEL_2_COLOR_LIGHT: Color32 = Color32::from_rgb(252, 252, 252);
/// Popup fill, color of the uppermost layer.
pub const LEVEL_3_COLOR_DARK: Color32 = Color32::from_rgb(28, 29, 31);
pub const LEVEL_3_COLOR_LIGHT: Color32 = Color32::WHITE;

// -----------------------------------------------------------------------------
// Shades & Tints: a range of greys from black to white, used for various UI
// elements and backgrounds.
// -----------------------------------------------------------------------------

/// rgb(0, 0, 0) - #000000
pub const TOTAL_BLACK: Color32 = Color32::from_rgb(0, 0, 0);

/// rgb(10, 10, 10  ) - #0a0a0a
pub const DARK_GREY_SHADE_10: Color32 = Color32::from_rgb(10, 10, 10);

/// rgb(20, 20, 20) - #141414
pub const DARK_GREY_SHADE_20: Color32 = Color32::from_rgb(20, 20, 20);

/// rgb(30, 30, 30) - #1e1e1e
pub const DARK_GREY_SHADE_30: Color32 = Color32::from_rgb(30, 30, 30);

/// rgb(40, 40, 40) - #282828
pub const DARK_GREY_SHADE_40: Color32 = Color32::from_rgb(40, 40, 40);

/// rgb(50, 50, 50) - #323232
pub const DARK_GREY_SHADE_50: Color32 = Color32::from_rgb(50, 50, 50);

/// rgb(255, 255, 255) - #ffffff
pub const TOTAL_WHITE: Color32 = Color32::from_rgb(255, 255, 255);

/// rgb(245, 245, 245) - #f5f5f5
pub const LIGHT_GREY_SHADE_10: Color32 = Color32::from_rgb(245, 245, 245);

/// rgb(235, 235, 235) - #ebebeb
pub const LIGHT_GREY_SHADE_20: Color32 = Color32::from_rgb(235, 235, 235);

/// rgb(224, 224, 224) - #e0e0e0
pub const LIGHT_GREY_SHADE_30: Color32 = Color32::from_rgb(224, 224, 224);

/// rgb(214, 214, 214) - #d6d6d6
pub const LIGHT_GREY_SHADE_40: Color32 = Color32::from_rgb(214, 214, 214);

/// rgb(204, 204, 204) - #cccccc
pub const LIGHT_GREY_SHADE_50: Color32 = Color32::from_rgb(204, 204, 204);

/// rgb(39, 40, 45) - #27282d
pub const C001: Color32 = Color32::from_rgb(39, 40, 45);

/// rgb(149, 149, 151) - #959597
pub const C002: Color32 = Color32::from_rgb(149, 149, 151);

/// rgb(21, 22, 25) - #151619
pub const C003: Color32 = Color32::from_rgb(21, 22, 25);

/// rgb(30, 31, 34) - #1e1f22
pub const C004: Color32 = Color32::from_rgb(30, 31, 34);

pub const C005: Color32 = Color32::from_rgb(42, 43, 48);
// pub const C006: Color32 = Color32::from_white_alpha(40);
pub const C007: Color32 = Color32::from_rgb(0, 132, 255);
pub const C008: Color32 = Color32::from_rgb(23, 150, 87);
pub const C009: Color32 = Color32::from_rgb(216, 216, 216);
pub const C010: Color32 = Color32::from_rgb(89, 90, 91);
pub const C011: Color32 = Color32::from_rgb(26, 26, 26);
pub const C012: Color32 = Color32::from_rgb(232, 232, 232);
pub const C013: Color32 = Color32::from_rgb(226, 226, 226);
pub const C014: Color32 = Color32::from_rgb(232, 232, 232);
pub const C015: Color32 = Color32::from_black_alpha(20);
pub const C016: Color32 = Color32::from_rgb(232, 157, 86);
pub const C017: Color32 = Color32::from_rgb(88, 232, 160);
pub const C018: Color32 = Color32::from_rgb(57, 59, 66);
pub const C019: Color32 = Color32::from_rgb(57, 59, 66);
pub const C020: Color32 = Color32::from_rgb(44, 44, 44);
pub const C021: Color32 = Color32::from_rgb(216, 216, 216);
pub const C022: Color32 = Color32::from_rgb(232, 232, 232);
pub const C023: Color32 = Color32::from_rgb(27, 27, 27);
pub const C024: Color32 = Color32::from_rgb(237, 237, 237);
pub const C025: Color32 = Color32::from_rgb(216, 216, 216);
pub const C026: Color32 = Color32::from_rgb(92, 92, 92);
