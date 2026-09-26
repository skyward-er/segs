use egui::{
    Color32, Margin, Painter, Popup, PopupCloseBehavior, Rect, Sense, StrokeKind, Ui, color_picker, pos2, vec2,
};
use segs_assets::icons;
use segs_ui::{
    containers::{RoundedGrid, RoundedGridColumn},
    style::CtxStyleExt,
    widgets::{
        buttons::IconBtn,
        text::{TextEdit, default_singleline_height},
    },
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

const VALUE_WIDTH: f32 = 42.;
const COLOR_WIDTH: f32 = 42.;
const ACTION_WIDTH: f32 = 24.;
const CELL_PADDING: f32 = 4.;

/// Shows editable mapping drafts with full-width validation feedback.
/// Applies row additions and removals without reordering active edits.
/// When `colors` is set, each row also edits its optional background color.
pub fn show(ui: &mut Ui, mappings: &mut Vec<IntegerTextMapping>, colors: bool) {
    ui.scope(|ui| {
        let row_height = default_singleline_height(ui) + 2.;
        let mut remove = None;
        let mut seen = std::collections::HashSet::new();
        let painter = ui.painter().clone();
        let error_color = ui.app_style().error_fg_color;

        // Let the shared container own all table geometry and border rendering
        let color_column = colors.then_some(RoundedGridColumn::Fixed(COLOR_WIDTH));
        let color_title = colors.then_some("Color");
        RoundedGrid::new("table")
            .columns(
                [RoundedGridColumn::Fixed(VALUE_WIDTH), RoundedGridColumn::Remainder]
                    .into_iter()
                    .chain(color_column)
                    .chain([RoundedGridColumn::Fixed(ACTION_WIDTH)]),
            )
            .min_row_height(row_height)
            .cell_padding(vec2(1., 1.))
            .header_rows(1)
            .show(ui, |grid| {
                for title in ["Value", "Text"].into_iter().chain(color_title).chain([""]) {
                    grid.cell(|ui| {
                        ui.add_space(CELL_PADDING - 1.);
                        ui.label(title);
                    });
                }
                grid.end_row();

                for (index, mapping) in mappings.iter_mut().enumerate() {
                    // Preserve invalid drafts and reject only later duplicate keys
                    let error = match mapping.parsed_value() {
                        None if mapping.value.trim().is_empty() => Some("Value is required"),
                        None => Some("Enter a whole number"),
                        Some(value) if !seen.insert(value) => Some("Value must be unique"),
                        Some(_) => None,
                    };
                    grid.cell(|ui| {
                        if error.is_some() {
                            ui.painter()
                                .rect_filled(ui.max_rect(), 0., ui.app_style().text_edit.invalid_fill);
                        }
                        edit_cell(ui, &mut mapping.value);
                    });
                    grid.cell(|ui| edit_cell(ui, &mut mapping.text));
                    if colors {
                        grid.cell(|ui| edit_color_cell(ui, &mut mapping.color));
                    }
                    grid.cell(|ui| {
                        let rect = ui.max_rect();
                        let size = 18_f32.min((rect.height() - 2.).max(0.)).min(rect.width());
                        let button_rect = Rect::from_center_size(rect.center(), vec2(size, size));
                        if IconBtn::new(icons::Trash)
                            .with_padding(3.)
                            .show_at(ui, button_rect, ui.id().with("delete"))
                            .on_hover_text("Delete mapping")
                            .clicked()
                        {
                            remove = Some(index);
                        }
                    });
                    grid.end_row();

                    // Measure feedback against the container's usable merged-row width
                    if let Some(error) = error {
                        let inset = CELL_PADDING - 1.;
                        let galley = painter.layout(
                            error.to_owned(),
                            egui::FontId::proportional(10.),
                            error_color,
                            (grid.available_width() - inset * 2.).max(1.),
                        );
                        grid.full_width_row(galley.size().y + CELL_PADDING * 2., |ui| {
                            ui.painter()
                                .galley(ui.max_rect().min + vec2(inset, inset), galley, error_color);
                        });
                    }
                }
            });

        // Apply structural edits only after the current table has been rendered
        if let Some(index) = remove {
            mappings.remove(index);
        }
        ui.add_space(4.);
        if ui.button("+ Add mapping").clicked() {
            mappings.push(IntegerTextMapping {
                value: String::new(),
                text: String::new(),
                color: None,
            });
        }
    });
}

/// Edits one frameless draft within the container-provided bounded cell.
fn edit_cell(ui: &mut Ui, text: &mut String) {
    // Keep editor identity distinct from the cell container's interaction identity
    ui.add(
        TextEdit::singleline(text)
            .id(ui.id().with("editor"))
            .frameless()
            .margin(Margin::symmetric(4, 2))
            .desired_width((ui.available_width() - 8.).max(0.))
            .clip_text(true),
    );
}

/// Edits one optional row background through a swatch centered in its cell.
/// `None` keeps the widget's own background and is shown as transparency checkers.
fn edit_color_cell(ui: &mut Ui, color: &mut Option<Color32>) {
    // Keep background colors opaque even when loading a manually edited layout
    if let Some(color) = color {
        *color = color.to_opaque();
    }

    // Draw the swatch like egui's color button, with theme checkers standing in for transparency
    let rect = ui.max_rect();
    let size = vec2((rect.width() - 8.).max(0.), (rect.height() - 6.).max(0.));
    let swatch_rect = Rect::from_center_size(rect.center(), size);
    let popup_id = ui.id().with("color_popup");
    let response = ui
        .interact(swatch_rect, ui.id().with("color_swatch"), Sense::click())
        .on_hover_text(if color.is_some() {
            "Background color"
        } else {
            "No color"
        });
    let visuals = if Popup::is_id_open(ui.ctx(), popup_id) {
        &ui.visuals().widgets.open
    } else {
        ui.style().interact(&response)
    };
    let background = ui.app_style().main_panels_fill;
    let painter = ui.painter();
    match color {
        Some(color) => {
            painter.rect_filled(swatch_rect.shrink(1.), 0., *color);
        }
        None => paint_checkers(painter, swatch_rect.shrink(1.), background, ui.visuals().text_color()),
    }
    painter.rect_stroke(swatch_rect, 2., (1., visuals.bg_fill), StrokeKind::Inside);

    // Offer the no-color default above a picker that starts from the widget background
    Popup::menu(&response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            if ui.selectable_label(color.is_none(), "No Color").clicked() {
                *color = None;
                ui.close();
            }
            ui.spacing_mut().slider_width = 275.;
            let mut picked = color.unwrap_or(background);
            if color_picker::color_picker_color32(ui, &mut picked, color_picker::Alpha::Opaque) {
                *color = Some(picked);
            }
        });
}

/// Paints checkers in the widget background tones to mark a transparent swatch.
/// Unlike egui's fixed dark checkers, they follow the active light or dark theme.
fn paint_checkers(painter: &Painter, rect: Rect, background: Color32, foreground: Color32) {
    if !rect.is_positive() {
        return; // Nothing to paint in a collapsed cell
    }

    // Alternate square-ish tiles between the background and a faint foreground tint
    painter.rect_filled(rect, 0., background);
    let checker = background.lerp_to_gamma(foreground, 0.25);
    let tile_height = rect.height() / 2.;
    let columns = (rect.width() / tile_height).round().max(1.);
    let tile_width = rect.width() / columns;
    for column in 0..columns as u32 {
        let top = if column % 2 == 0 { rect.top() } else { rect.center().y };
        let min = pos2(rect.left() + tile_width * column as f32, top);
        painter.rect_filled(Rect::from_min_size(min, vec2(tile_width, tile_height)), 0., checker);
    }
}

/// Removes invalid or duplicate rows and canonicalizes keys in numeric order.
pub fn prepare_for_save(mappings: &mut Vec<IntegerTextMapping>) {
    let mut values = std::collections::HashSet::new();

    // Retain the first occurrence of each valid integer in display order
    mappings.retain_mut(|mapping| {
        let Some(value) = mapping.parsed_value() else {
            return false;
        };
        if !values.insert(value) {
            return false;
        }
        mapping.value = value.to_string();
        true
    });

    // Keep saved snapshots sorted so in-memory activation matches file loading
    mappings.sort_by_key(IntegerTextMapping::parsed_value);
}

/// Loads mappings in ascending numeric order, retaining duplicate-key order.
///
/// Returns the sorted rows with malformed drafts last, or the deserializer's
/// error when the mapping collection cannot be decoded.
pub fn deserialize_mappings<'de, D>(deserializer: D) -> Result<Vec<IntegerTextMapping>, D::Error>
where
    D: Deserializer<'de>,
{
    // Sort only at the persistence boundary so active edits retain their positions
    let mut mappings = Vec::<IntegerTextMapping>::deserialize(deserializer)?;
    mappings.sort_by_key(|mapping| {
        let value = mapping.parsed_value();
        (value.is_none(), value)
    });
    Ok(mappings)
}

/// One persisted integer-to-text mapping edited by a widget setting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegerTextMapping {
    /// Editable integer stream value matched by this mapping.
    #[serde(
        serialize_with = "serialize_integer_text",
        deserialize_with = "deserialize_integer_text"
    )]
    pub value: String,
    /// Text displayed when `value` is the latest stream value.
    pub text: String,
    /// Opaque background shown behind `text`, or `None` to keep the widget's own
    /// background. Always `None` in tables that do not edit colors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color32>,
}

impl IntegerTextMapping {
    /// Parses the editable key as a signed 64-bit integer.
    ///
    /// Returns the parsed value after trimming whitespace, or `None` when the
    /// draft is empty or malformed.
    pub fn parsed_value(&self) -> Option<i64> {
        self.value.trim().parse().ok()
    }
}

/// Serializes a valid editable integer as a native JSON number.
fn serialize_integer_text<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value
        .trim()
        .parse::<i64>()
        .map_err(serde::ser::Error::custom)?
        .serialize(serializer)
}

/// Deserializes numeric mapping keys while accepting string drafts defensively.
fn deserialize_integer_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntegerTextWire {
        Integer(i64),
        Text(String),
    }

    Ok(match IntegerTextWire::deserialize(deserializer)? {
        IntegerTextWire::Integer(value) => value.to_string(),
        IntegerTextWire::Text(value) => value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_normalization_filters_duplicates_and_sorts_idempotently() {
        // Mix editable drafts, equivalent keys, numeric boundaries and unsorted values
        let mut mappings: Vec<_> = [
            ("10", "ten"),
            ("01", "first one"),
            ("", "empty draft"),
            (" 1 ", "duplicate one"),
            ("+1", "another duplicate"),
            ("-", "incomplete draft"),
            ("no", "malformed draft"),
            ("9223372036854775808", "overflow"),
            ("-9223372036854775809", "underflow"),
            ("2", ""),
            (" -02 ", "negative"),
            ("9223372036854775807", "maximum"),
            ("-9223372036854775808", "minimum"),
        ]
        .into_iter()
        .map(|(value, text)| IntegerTextMapping {
            value: value.to_owned(),
            text: text.to_owned(),
            color: None,
        })
        .collect();

        // Preserve the first text for each integer and retain valid empty output
        prepare_for_save(&mut mappings);
        let rows: Vec<_> = mappings
            .iter()
            .map(|row| (row.value.as_str(), row.text.as_str()))
            .collect();
        assert_eq!(
            rows,
            [
                ("-9223372036854775808", "minimum"),
                ("-2", "negative"),
                ("1", "first one"),
                ("2", ""),
                ("10", "ten"),
                ("9223372036854775807", "maximum"),
            ]
        );

        // A second save must leave the normalized result unchanged
        let normalized = mappings.clone();
        prepare_for_save(&mut mappings);
        assert_eq!(mappings, normalized);
    }
}
