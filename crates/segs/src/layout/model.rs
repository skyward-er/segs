use chrono::{DateTime, Utc};
use egui::Id;
use rand::random;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ui::{
    grid::{GRect, GridSettings},
    widgets::{WidgetData, WidgetVariant},
};

pub const CURRENT_LAYOUT_SCHEMA: u32 = 1;
const ADDED_WIDGET_ID_NAMESPACE: &str = "layout_added_widget";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub schema_version: u32,
    pub slug: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub widgets: Vec<WidgetData>,
    pub grid_settings: GridSettings,
}

impl Layout {
    /// Creates a layout with no widgets and the default grid settings.
    pub fn empty(name: String, slug: String) -> Self {
        let now = Utc::now();
        Self {
            schema_version: CURRENT_LAYOUT_SCHEMA,
            slug,
            name,
            created_at: now,
            modified_at: now,
            widgets: Vec::new(),
            grid_settings: GridSettings::new(8, 8),
        }
    }

    /// Adds a widget and returns its randomly generated id.
    pub fn add_widget(&mut self, variant: WidgetVariant, grect: GRect) -> Id {
        self.add_widget_with_id_source(variant, grect, random)
    }

    fn add_widget_with_id_source(&mut self, variant: WidgetVariant, grect: GRect, mut next: impl FnMut() -> u64) -> Id {
        let id = loop {
            let id = Id::new((ADDED_WIDGET_ID_NAMESPACE, next()));
            if self.widgets.iter().all(|widget| widget.id != id) {
                break id;
            }
        };
        self.widgets.push(WidgetData { id, grect, variant });
        id
    }

    /// Removes the widget with the given id when it is present.
    pub fn remove_widget(&mut self, id: Id) {
        self.widgets.retain(|widget| widget.id != id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LayoutNameError {
    #[error("Layout names cannot be empty.")]
    Empty,
    #[error("Layout names may only contain ASCII letters, numbers, spaces, underscores, and dashes.")]
    InvalidCharacter,
    #[error("A layout named '{0}' already exists.")]
    Duplicate(String),
}

/// Trims and validates a user-facing layout name.
pub fn validated_display_name(name: &str) -> Result<String, LayoutNameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(LayoutNameError::Empty);
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-'))
    {
        return Err(LayoutNameError::InvalidCharacter);
    }
    Ok(name.to_owned())
}

/// Converts a display name to its lowercase, dash-separated filename component.
pub fn normalized_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    let mut separator_pending = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            if separator_pending && !normalized.is_empty() {
                normalized.push('-');
            }
            normalized.push(ch.to_ascii_lowercase());
            separator_pending = false;
        } else {
            separator_pending = true;
        }
    }
    normalized
}

/// Combines a normalized display name with an eight-character hexadecimal suffix.
pub fn slug_with_suffix(name: &str, suffix: u32) -> String {
    format!("{}-{suffix:08x}", normalized_name(name))
}

/// Returns the hexadecimal suffix from a valid layout slug.
pub fn slug_suffix(slug: &str) -> Option<&str> {
    let (base, suffix) = slug.rsplit_once('-')?;
    (!base.is_empty()
        && suffix.len() == 8
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then_some(suffix)
}

/// Rebuilds a slug for a renamed layout while preserving its random suffix.
pub fn renamed_slug(name: &str, old_slug: &str) -> Option<String> {
    slug_suffix(old_slug).map(|suffix| format!("{}-{suffix}", normalized_name(name)))
}

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2, vec2};

    use super::*;
    use crate::ui::widgets::ValueDisplayWidget;

    #[test]
    fn validates_and_normalizes_names() {
        assert_eq!(validated_display_name("  Flight__ Main  ").unwrap(), "Flight__ Main");
        assert_eq!(normalized_name("  Flight__ Main  "), "flight-main");
        assert_eq!(slug_with_suffix("Flight Main", 0x12ab), "flight-main-000012ab");
        assert!(matches!(validated_display_name(""), Err(LayoutNameError::Empty)));
        assert!(matches!(
            validated_display_name("Flight/Primary"),
            Err(LayoutNameError::InvalidCharacter)
        ));
    }

    #[test]
    fn rename_preserves_suffix() {
        assert_eq!(
            renamed_slug("New Name", "old-name-deadbeef").as_deref(),
            Some("new-name-deadbeef")
        );
        assert!(renamed_slug("New Name", "invalid").is_none());
    }

    #[test]
    fn retries_widget_id_collisions() {
        let mut layout = Layout::empty("Test".into(), "test-00000001".into());
        let rect = GRect::new(Rect::from_min_size(pos2(0., 0.), vec2(1., 1.)));
        let first = layout.add_widget_with_id_source(ValueDisplayWidget::default().into(), rect, || 7);
        let mut values = [7, 8].into_iter();
        let second =
            layout.add_widget_with_id_source(ValueDisplayWidget::default().into(), rect, || values.next().unwrap());
        assert_ne!(first, second);
    }

    #[test]
    fn complete_layout_round_trips_through_json() {
        let mut layout = Layout::empty("Round Trip".into(), "round-trip-deadbeef".into());
        let rect = GRect::new(Rect::from_min_size(pos2(2., 3.), vec2(4., 5.)));
        layout.add_widget(ValueDisplayWidget::default().into(), rect);
        layout.grid_settings = GridSettings::new(12, 7);

        let json = serde_json::to_string_pretty(&layout).unwrap();
        let restored: Layout = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, layout);
    }
}
