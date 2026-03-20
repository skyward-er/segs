//! This module collects all the colors used in the UI, so that they can be
//! easily changed and maintained in one place.

use egui::Color32;

// -----------------------------------------------------------------------------
// Background Fills
// -----------------------------------------------------------------------------

// ---- Background Fill --------------------------------------------------------
pub const BACKGROUND_DARK: Color32 = Color32::from_rgb(9, 9, 9);
pub const BACKGROUND_LIGHT: Color32 = Color32::from_rgb(240, 240, 240);
// ---- Panel (Left, Right & Bottom) Fill --------------------------------------
pub const PANEL_DARK: Color32 = Color32::from_rgb(16, 16, 16);
pub const PANEL_LIGHT: Color32 = Color32::from_rgb(249, 249, 249);
// ---- Main View Fill
// -----------------------------------------------------------------------------
pub const MAIN_VIEW_DARK: Color32 = Color32::from_rgb(22, 22, 22);
pub const MAIN_VIEW_LIGHT: Color32 = Color32::from_rgb(252, 252, 252);
// ---- Foreground Fill (Popups) -----------------------------------------------
pub const FOREGROUND_DARK: Color32 = Color32::from_rgb(29, 29, 29);
pub const FOREGROUND_LIGHT: Color32 = Color32::WHITE;

// -----------------------------------------------------------------------------
// Stroke Colors
// -----------------------------------------------------------------------------

// ---- Text color -------------------------------------------------------------
pub const TEXT_DARK: Color32 = Color32::from_rgb(227, 228, 229);
pub const TEXT_LIGHT: Color32 = Color32::from_rgb(27, 27, 27);
// ---- Main View --------------------------------------------------------------
pub const MAIN_VIEW_STROKE_DARK: Color32 = Color32::from_rgb(36, 36, 36);
pub const MAIN_VIEW_STROKE_LIGHT: Color32 = Color32::from_rgb(216, 216, 216);
// ---- Inactive Icons on Background -------------------------------------------
pub const ICON_INACTIVE_ON_BACKGROUND_DARK: Color32 = Color32::from_rgb(149, 149, 149);
pub const ICON_INACTIVE_ON_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(89, 90, 91);
// ---- Active Icons on Background ---------------------------------------------
pub const ICON_ACTIVE_ON_BACKGROUND_DARK: Color32 = Color32::WHITE;
pub const ICON_ACTIVE_ON_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(26, 26, 26);
// ---- Popup stroke color -----------------------------------------------------
pub const POPUP_STROKE_DARK: Color32 = Color32::from_rgb(57, 59, 66);
pub const POPUP_STROKE_LIGHT: Color32 = Color32::from_rgb(216, 216, 216);

// -----------------------------------------------------------------------------
// Widgets Colors
// -----------------------------------------------------------------------------

pub const NONINTERACTIVE_WIDGET_BG_FILL_DARK: Color32 = Color32::from_gray(27);
pub const NONINTERACTIVE_WIDGET_BG_FILL_LIGHT: Color32 = Color32::from_gray(248);
pub const NONINTERACTIVE_WIDGET_BG_STROKE_DARK: Color32 = Color32::from_gray(60);
pub const NONINTERACTIVE_WIDGET_BG_STROKE_LIGHT: Color32 = Color32::from_gray(190);
pub const NONINTERACTIVE_WIDGET_FG_STROKE_DARK: Color32 = Color32::from_gray(140);
pub const NONINTERACTIVE_WIDGET_FG_STROKE_LIGHT: Color32 = Color32::from_gray(80);

pub const INACTIVE_WIDGET_BG_FILL_DARK: Color32 = Color32::from_gray(60);
pub const INACTIVE_WIDGET_BG_FILL_LIGHT: Color32 = Color32::from_gray(230);
pub const INACTIVE_WIDGET_BG_STROKE_DARK: Color32 = Color32::TRANSPARENT;
pub const INACTIVE_WIDGET_BG_STROKE_LIGHT: Color32 = Color32::TRANSPARENT;
pub const INACTIVE_WIDGET_FG_STROKE_DARK: Color32 = Color32::from_gray(180);
pub const INACTIVE_WIDGET_FG_STROKE_LIGHT: Color32 = Color32::from_gray(60);

pub const HOVERED_WIDGET_BG_FILL_DARK: Color32 = Color32::from_gray(70);
pub const HOVERED_WIDGET_BG_FILL_LIGHT: Color32 = Color32::from_gray(220);
pub const HOVERED_WIDGET_BG_STROKE_DARK: Color32 = Color32::from_gray(150);
pub const HOVERED_WIDGET_BG_STROKE_LIGHT: Color32 = Color32::from_gray(105);
pub const HOVERED_WIDGET_FG_STROKE_DARK: Color32 = Color32::from_gray(240);
pub const HOVERED_WIDGET_FG_STROKE_LIGHT: Color32 = Color32::BLACK;

pub const ACTIVE_WIDGET_BG_FILL_DARK: Color32 = Color32::from_gray(55);
pub const ACTIVE_WIDGET_BG_FILL_LIGHT: Color32 = Color32::from_gray(165);
pub const ACTIVE_WIDGET_BG_STROKE_DARK: Color32 = Color32::WHITE;
pub const ACTIVE_WIDGET_BG_STROKE_LIGHT: Color32 = Color32::BLACK;
pub const ACTIVE_WIDGET_FG_STROKE_DARK: Color32 = Color32::WHITE;
pub const ACTIVE_WIDGET_FG_STROKE_LIGHT: Color32 = Color32::BLACK;

pub const TEXT_EDIT_INACTIVE_FILL_DARK: Color32 = Color32::from_rgb(26, 26, 26);
pub const TEXT_EDIT_INACTIVE_FILL_LIGHT: Color32 = Color32::from_rgb(228, 228, 228);
pub const TEXT_EDIT_HOVER_FILL_DARK: Color32 = Color32::from_rgb(40, 40, 40);
pub const TEXT_EDIT_HOVER_FILL_LIGHT: Color32 = Color32::from_rgb(214, 214, 214);
pub const TEXT_EDIT_ACTIVE_FILL_DARK: Color32 = Color32::from_rgb(54, 54, 54);
pub const TEXT_EDIT_ACTIVE_FILL_LIGHT: Color32 = Color32::from_rgb(200, 200, 200);

pub const DRAG_VALUE_INACTIVE_FILL_DARK: Color32 = Color32::from_rgb(26, 26, 26);
pub const DRAG_VALUE_INACTIVE_FILL_LIGHT: Color32 = Color32::from_rgb(228, 228, 228);
pub const DRAG_VALUE_HOVER_FILL_DARK: Color32 = Color32::from_rgb(40, 40, 40);
pub const DRAG_VALUE_HOVER_FILL_LIGHT: Color32 = Color32::from_rgb(214, 214, 214);
pub const DRAG_VALUE_ACTIVE_FILL_DARK: Color32 = Color32::from_rgb(54, 54, 54);
pub const DRAG_VALUE_ACTIVE_FILL_LIGHT: Color32 = Color32::from_rgb(200, 200, 200);

// -----------------------------------------------------------------------------
// Shadow Colors
// -----------------------------------------------------------------------------

// ---- Color of a light shadow (e.g. for hover effects in the left bar) -------
pub const SHADOW_LIGHT_ON_BACKGROUND_DARK: Color32 = Color32::from_rgb(21, 22, 25);
pub const SHADOW_LIGHT_ON_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(232, 232, 232);
// ---- Color of the shadown on background (e.g. for the active left bar icons)
pub const SHADOW_MEDIUM_ON_BACKGROUND_DARK: Color32 = Color32::from_rgb(30, 31, 34);
pub const SHADOW_MEDIUM_ON_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(226, 226, 226);
// ---- Color of a stronger shadow (e.g. for the icon buttons) -----------------
pub const SHADOW_STRONG_ON_BACKGROUND_DARK: Color32 = SHADOW_MEDIUM_ON_BACKGROUND_DARK; // FIXME
pub const SHADOW_STRONG_ON_BACKGROUND_LIGHT: Color32 = Color32::from_rgb(217, 217, 217);
// ----- Popup shadow color ----------------------------------------------------
pub const POPUP_SHADOW_DARK: Color32 = Color32::from_rgb(21, 22, 25);
pub const POPUP_SHADOW_LIGHT: Color32 = Color32::from_rgb(232, 232, 232);

// -----------------------------------------------------------------------------
// Theme Colors (e.g. for accents, highlights, confirmation states)
// -----------------------------------------------------------------------------

// ---- Accent Color (e.g. for buttons, highlights) ----------------------------
pub const ACCENT_FILL_DARK: Color32 = Color32::from_rgb(0, 132, 255);
pub const ACCENT_FILL_LIGHT: Color32 = Color32::from_rgb(232, 157, 86);
// ---- Confirmation Color (e.g. for confirmation states) ----------------------
pub const CONFIRMATION_FILL_DARK: Color32 = Color32::from_rgb(23, 150, 87);
pub const CONFIRMATION_FILL_LIGHT: Color32 = Color32::from_rgb(88, 232, 160);
