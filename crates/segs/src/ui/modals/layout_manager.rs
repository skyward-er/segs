mod search;

use std::sync::Arc;

use chrono::Local;
use egui::{
    Align, Align2, Button, Frame, Id, Key, Layout, Margin, Modifiers, Response, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, TextEdit, Tooltip, Ui, UiBuilder, Vec2,
    text::{CCursor, CCursorRange},
    vec2,
};
use segs_assets::icons::{self, Icon};
use segs_memory::MemoryExt;
use segs_ui::containers::Modal;

use crate::{
    layout::LayoutManager,
    ui::{
        layout::{clear_any_control_error, clear_control_error, set_control_error, show_control_error},
        popups::DeleteConfirmationPopup,
        views::ViewTarget,
    },
};

const MANAGER_MODAL_ID: &str = "layout_manager_modal";
const MANAGER_CONTENT_SIZE: Vec2 = vec2(680., 420.);
const SELECTED_SLUG_ID: &str = "layout_manager_selected_slug";
const SEARCH_QUERY_ID: &str = "layout_manager_search_query";
const SEARCH_CACHE_ID: &str = "layout_manager_search_cache";
const SEARCH_INPUT_ID: &str = "layout_manager_search_input";
const INLINE_EDIT_ID: &str = "layout_manager_inline_edit";
const DELETE_CONFIRMATION_ID: &str = "layout_manager_delete_confirmation";
const OPEN_FOLDER_TOOLTIP: &str = concat!(
    "Changes made while the application is running won't appear and may be overwritten. ",
    "A restart is required to reload layouts."
);

/// Identifies the catalog mutation performed by the shared inline name editor.
#[derive(Clone, Debug)]
enum InlineEditAction {
    Create,
    Duplicate { source: String },
    Rename { source: String },
}

/// Persists inline editor input and errors across frames.
#[derive(Clone, Debug)]
struct InlineEdit {
    action: InlineEditAction,
    value: String,
    error: Option<InlineEditError>,
    request_focus: bool,
}

/// Retains a pending deletion and any persistence error while its popup is open.
#[derive(Clone, Debug)]
struct DeleteConfirmation {
    slug: String,
    error: Option<String>,
}

#[derive(Clone, Debug)]
enum InlineEditError {
    Validation(String),
    Store(String),
}

impl InlineEditError {
    /// Returns the message shown beside the inline editor.
    fn message(&self) -> &str {
        match self {
            Self::Validation(message) | Self::Store(message) => message,
        }
    }
}

pub struct LayoutManagerModalResponse {
    pub should_close: bool,
    pub transition: Option<ViewTarget>,
}

/// Modal for selecting and managing saved layouts.
pub struct LayoutManagerModal<'a> {
    layouts: &'a mut LayoutManager,
}

impl<'a> LayoutManagerModal<'a> {
    /// Creates a layout manager backed by the supplied catalog.
    pub fn new(layouts: &'a mut LayoutManager) -> Self {
        Self { layouts }
    }

    /// Shows the manager and reports any requested view transition.
    pub fn show(self, ui: &mut Ui) -> LayoutManagerModalResponse {
        show_manager(ui, self.layouts)
    }
}

/// Returns the slug selected in the manager.
fn selected_slug(ui: &Ui) -> Option<String> {
    ui.mem().get_temp(Id::new(SELECTED_SLUG_ID))
}

/// Updates or clears the slug selected in the manager.
fn set_selected_slug(ui: &Ui, slug: Option<String>) {
    if let Some(slug) = slug {
        ui.mem().insert_temp(Id::new(SELECTED_SLUG_ID), slug);
    } else {
        ui.mem().remove_temp::<String>(Id::new(SELECTED_SLUG_ID));
    }
}

/// Returns the current manager search query.
fn search_query(ui: &Ui) -> String {
    ui.mem().get_temp_or_default(Id::new(SEARCH_QUERY_ID))
}

/// Updates the current manager search query.
fn set_search_query(ui: &Ui, query: String) {
    ui.mem().insert_temp(Id::new(SEARCH_QUERY_ID), query);
}

/// Returns the latest matching cached search or replaces the single cache entry.
fn cached_search(ui: &Ui, layouts: &LayoutManager, query: &str) -> (Arc<search::CachedSearch>, bool) {
    let cached = ui.mem().get_temp(Id::new(SEARCH_CACHE_ID));
    let (cached, recomputed) = search::resolve_cached_search(cached, layouts.layouts(), query);
    if recomputed {
        ui.mem().insert_temp(Id::new(SEARCH_CACHE_ID), cached.clone());
    }
    (cached, recomputed)
}

/// Clears search results after a successful catalog mutation.
fn invalidate_search_cache(ui: &Ui) {
    ui.mem()
        .remove_temp::<Arc<search::CachedSearch>>(Id::new(SEARCH_CACHE_ID));
}

/// Returns the active inline naming operation.
fn inline_edit(ui: &Ui) -> Option<InlineEdit> {
    ui.mem().get_temp(Id::new(INLINE_EDIT_ID))
}

/// Updates or clears the active inline naming operation.
fn set_inline_edit(ui: &Ui, edit: Option<InlineEdit>) {
    if let Some(edit) = edit {
        ui.mem().insert_temp(Id::new(INLINE_EDIT_ID), edit);
    } else {
        ui.mem().remove_temp::<InlineEdit>(Id::new(INLINE_EDIT_ID));
    }
}

/// Returns the pending layout deletion confirmation.
fn delete_confirmation(ui: &Ui) -> Option<DeleteConfirmation> {
    ui.mem().get_temp(Id::new(DELETE_CONFIRMATION_ID))
}

/// Updates or clears the pending layout deletion confirmation.
fn set_delete_confirmation(ui: &Ui, confirmation: Option<DeleteConfirmation>) {
    if let Some(confirmation) = confirmation {
        ui.mem().insert_temp(Id::new(DELETE_CONFIRMATION_ID), confirmation);
    } else {
        ui.mem()
            .remove_temp::<DeleteConfirmation>(Id::new(DELETE_CONFIRMATION_ID));
    }
}

/// Defers actions from modal controls until after the modal has rendered.
enum ManagerCommand {
    Open(String, Id),
    OpenFolder(Id),
    Edit(String, Id),
    Duplicate(String, String),
    Rename(String, String),
    ToggleDefault(String, Id),
}

/// Describes whether the inline editor should remain open, cancel, or submit.
#[derive(Debug, PartialEq, Eq)]
enum InlineEditIntent {
    Continue,
    Cancel,
    Submit { explicit: bool },
}

/// Shows the searchable layout manager and dispatches its selected operation.
fn show_manager(ui: &mut Ui, layouts: &mut LayoutManager) -> LayoutManagerModalResponse {
    let mut command = None;
    let mut transition = None;
    let mut should_close = false;
    let mut query = search_query(ui);
    let mut selected = selected_slug(ui);
    let mut edit = inline_edit(ui);
    let mut pending_delete = delete_confirmation(ui);
    let (cached_search, search_recomputed) = cached_search(ui, layouts, &query);
    let results = &cached_search.results;
    if selected.is_none()
        || (search_recomputed
            && selected
                .as_ref()
                .is_some_and(|selected| !results.iter().any(|result| &result.slug == selected)))
    {
        selected = results.first().map(|result| result.slug.clone());
    }

    let modal_frame = Frame::popup(ui.style());
    let modal_inner_margin = modal_frame.inner_margin;
    let response = Modal::new(Id::new(MANAGER_MODAL_ID), "Layout Manager")
        .frame(modal_frame)
        .show(ui.ctx(), |ui| {
            let manager_content_top = ui.cursor().top();
            let mut separator_response = None;
            let mut search_response = None;
            ui.allocate_ui_with_layout(MANAGER_CONTENT_SIZE, Layout::left_to_right(Align::Min), |ui| {
                let content_height = ui.available_height();
                ui.vertical(|ui| {
                    ui.set_width(270.);
                    ui.set_height(content_height);
                    search_response = Some(
                        ui.add(
                            TextEdit::singleline(&mut query)
                                .id_salt(SEARCH_INPUT_ID)
                                .hint_text("Search layouts…"),
                        ),
                    );
                    ui.add_space(6.);
                    let reserved_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
                    let list_height = (ui.available_height() - reserved_height).max(64.);
                    ScrollArea::vertical()
                        .max_height(list_height)
                        .min_scrolled_height(list_height)
                        .auto_shrink([true, false])
                        .show(ui, |ui| {
                            for result in results {
                                let slug = &result.slug;
                                let renaming = matches!(
                                    edit.as_ref().map(|edit| &edit.action),
                                    Some(InlineEditAction::Rename { source }) if source == slug
                                );
                                if renaming {
                                    if let Some(target) = show_inline_editor(ui, layouts, &mut edit, &mut selected) {
                                        transition = Some(target);
                                        should_close = true;
                                    }
                                } else {
                                    show_layout_row(ui, layouts, slug, &result.name, &mut selected);
                                }
                            }

                            let bottom_edit = matches!(
                                edit.as_ref().map(|edit| &edit.action),
                                Some(InlineEditAction::Create | InlineEditAction::Duplicate { .. })
                            );
                            if bottom_edit {
                                if let Some(target) = show_inline_editor(ui, layouts, &mut edit, &mut selected) {
                                    transition = Some(target);
                                    should_close = true;
                                }
                            } else if results.is_empty() {
                                ui.weak("No matching layouts");
                            }

                            if !layouts.warnings().is_empty() {
                                ui.add_space(12.);
                                ui.label(RichText::new("Catalog warnings").strong());
                                for warning in layouts.warnings() {
                                    ui.colored_label(ui.visuals().warn_fg_color, warning);
                                }
                            }
                        });
                    ui.add_space(8.);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(edit.is_none(), Button::new("New Empty Layout"))
                            .clicked()
                        {
                            edit = Some(InlineEdit {
                                action: InlineEditAction::Create,
                                value: String::new(),
                                error: None,
                                request_focus: true,
                            });
                        }

                        let open_folder = ui.add_enabled(edit.is_none(), Button::new("Open Layouts Folder"));
                        if open_folder.clicked() {
                            command = Some(ManagerCommand::OpenFolder(open_folder.id));
                        }
                        let showing_error = show_control_error(ui, &open_folder);
                        if open_folder.hovered() && !showing_error {
                            Tooltip::for_widget(&open_folder).show(|ui| {
                                ui.label(OPEN_FOLDER_TOOLTIP);
                            });
                        }
                    });
                });

                separator_response = Some(ui.separator());
                ui.vertical(|ui| {
                    ui.set_width(380.);
                    ui.set_height(content_height);
                    let Some((slug, name, created_at, modified_at, widget_count, grid_cols, grid_rows)) =
                        selected.as_deref().and_then(|slug| layouts.layout(slug)).map(|layout| {
                            (
                                layout.slug.clone(),
                                layout.name.clone(),
                                layout
                                    .created_at
                                    .with_timezone(&Local)
                                    .format("%Y-%m-%d %H:%M")
                                    .to_string(),
                                layout
                                    .modified_at
                                    .with_timezone(&Local)
                                    .format("%Y-%m-%d %H:%M")
                                    .to_string(),
                                layout.widgets.len(),
                                layout.grid_settings.cols,
                                layout.grid_settings.rows,
                            )
                        })
                    else {
                        ui.heading(RichText::new("No layout selected").weak());
                        return;
                    };

                    ui.heading(&name);
                    ui.monospace(format!("{slug}.json"));
                    ui.add_space(8.);
                    metadata_row(ui, "Created", &created_at);
                    metadata_row(ui, "Modified", &modified_at);
                    metadata_row(ui, "Widgets", &widget_count.to_string());
                    metadata_row(ui, "Grid", &format!("{grid_cols} × {grid_rows}"));

                    ui.add_space(16.);
                    ui.horizontal(|ui| {
                        let open = ui.add_enabled(edit.is_none(), Button::new("Open"));
                        if open.clicked() {
                            command = Some(ManagerCommand::Open(slug.clone(), open.id));
                        }
                        show_control_error(ui, &open);

                        let edit_button = ui.add_enabled(edit.is_none(), Button::new("Edit"));
                        if edit_button.clicked() {
                            command = Some(ManagerCommand::Edit(slug.clone(), edit_button.id));
                        }
                        show_control_error(ui, &edit_button);

                        if ui.add_enabled(edit.is_none(), Button::new("Duplicate")).clicked() {
                            command = Some(ManagerCommand::Duplicate(slug.clone(), format!("{name} Copy")));
                        }
                        if ui.add_enabled(edit.is_none(), Button::new("Rename")).clicked() {
                            command = Some(ManagerCommand::Rename(slug.clone(), name.clone()));
                        }
                    });
                    ui.add_space(6.);
                    ui.horizontal(|ui| {
                        let default_label = if layouts.default_slug() == Some(slug.as_str()) {
                            "Clear Default"
                        } else {
                            "Set as Default"
                        };
                        let default_button = ui.add_enabled(edit.is_none(), Button::new(default_label));
                        if default_button.clicked() {
                            command = Some(ManagerCommand::ToggleDefault(slug.clone(), default_button.id));
                        }
                        show_control_error(ui, &default_button);

                        let delete_button = ui.add_enabled(edit.is_none(), Button::new("Delete"));
                        if delete_button.clicked() {
                            pending_delete = Some(DeleteConfirmation {
                                slug: slug.clone(),
                                error: None,
                            });
                        }
                        if pending_delete
                            .as_ref()
                            .is_some_and(|confirmation| confirmation.slug == slug)
                        {
                            if let Some(target) = show_delete_popup(
                                ui,
                                layouts,
                                &delete_button,
                                &slug,
                                &mut pending_delete,
                                &mut selected,
                            ) {
                                transition = Some(target);
                            }
                        }
                    });
                });
            });

            if let Some(separator) = separator_response {
                let content_rect = ui.min_rect();
                let separator_style = ui.style().separator_style(separator.widget_state());
                let title_separator_y =
                    manager_content_top - ui.spacing().item_spacing.y - separator_style.spacing / 2.;
                ui.painter().vline(
                    separator.rect.center().x,
                    title_separator_y..=(content_rect.bottom() + modal_inner_margin.bottomf()),
                    separator_style.stroke,
                );
            }

            if let Some(search_response) = &search_response {
                retain_search_focus(search_response, edit.is_some());
            }
        });

    set_search_query(ui, query);
    set_selected_slug(ui, selected);
    set_inline_edit(ui, edit);
    set_delete_confirmation(ui, pending_delete);
    if response.should_close() {
        should_close = true;
        set_inline_edit(ui, None);
        set_delete_confirmation(ui, None);
        clear_any_control_error(ui);
    }

    if let Some(command) = command {
        match command {
            ManagerCommand::Open(slug, owner) => {
                if let Some(target) = activate(ui, layouts, slug, ViewTarget::Operator, owner) {
                    transition = Some(target);
                    should_close = true;
                }
            }
            ManagerCommand::OpenFolder(owner) => {
                clear_control_error(ui, owner);
                if let Err(error) = open::that_detached(layouts.directory()) {
                    set_control_error(ui, owner, error);
                }
            }
            ManagerCommand::Edit(slug, owner) => {
                if let Some(target) = activate(ui, layouts, slug, ViewTarget::Configuration, owner) {
                    transition = Some(target);
                    should_close = true;
                }
            }
            ManagerCommand::Duplicate(source, value) => set_inline_edit(
                ui,
                Some(InlineEdit {
                    action: InlineEditAction::Duplicate { source },
                    value,
                    error: None,
                    request_focus: true,
                }),
            ),
            ManagerCommand::Rename(source, value) => set_inline_edit(
                ui,
                Some(InlineEdit {
                    action: InlineEditAction::Rename { source },
                    value,
                    error: None,
                    request_focus: true,
                }),
            ),
            ManagerCommand::ToggleDefault(slug, owner) => {
                let new_default = (layouts.default_slug() != Some(slug.as_str())).then_some(slug.as_str());
                clear_control_error(ui, owner);
                if let Err(error) = layouts.set_default(new_default) {
                    set_control_error(ui, owner, error);
                }
            }
        }
    }

    if should_close {
        set_inline_edit(ui, None);
        set_delete_confirmation(ui, None);
    }

    LayoutManagerModalResponse {
        should_close,
        transition,
    }
}

/// Keeps ordinary modal keyboard input routed to search while yielding to name editors.
fn retain_search_focus(search_response: &Response, inline_edit_active: bool) {
    if !inline_edit_active {
        search_response.request_focus();
    }
}

/// Shows a saved layout as one full-width selectable row.
fn show_layout_row(ui: &mut Ui, layouts: &LayoutManager, slug: &str, name: &str, selected: &mut Option<String>) {
    let mut text = name.to_owned();
    if layouts.active_slug() == Some(slug) {
        text.push_str("  • Active");
    }
    let selected_row = selected.as_deref() == Some(slug);
    let mut button = Button::selectable(selected_row, text).min_size(vec2(ui.available_width(), 0.));
    if layouts.default_slug() == Some(slug) {
        // Reserve room for the separately interactive default-layout icon
        button = button.right_text("    ");
    }
    let response = ui.add(button);
    if response.clicked() {
        *selected = Some(slug.to_owned());
    }
    if layouts.default_slug() == Some(slug) {
        let icon_size = vec2(14., 14.);
        let icon_pos = response.rect.right_center() - vec2(6., 0.);
        let icon_rect = Align2::RIGHT_CENTER.anchor_size(icon_pos, icon_size);
        let tint = ui.style().interact(&response).fg_stroke.color;
        icons::Star::solid()
            .to_image()
            .tint(tint)
            .fit_to_exact_size(icon_size)
            .paint_at(ui, icon_rect);
        ui.interact(icon_rect, response.id.with("default_layout_icon"), Sense::hover())
            .on_hover_text("Default layout loaded at startup");
    }
}

/// Shows and resolves the active create, duplicate, or rename editor.
fn show_inline_editor(
    ui: &mut Ui,
    layouts: &mut LayoutManager,
    edit: &mut Option<InlineEdit>,
    selected: &mut Option<String>,
) -> Option<ViewTarget> {
    let Some(mut current) = edit.take() else {
        return None;
    };
    let excluding = match &current.action {
        InlineEditAction::Rename { source } => Some(source.as_str()),
        InlineEditAction::Create | InlineEditAction::Duplicate { .. } => None,
    };

    if !matches!(current.error, Some(InlineEditError::Store(_))) {
        current.error = (!current.value.trim().is_empty())
            .then(|| layouts.validate_name(&current.value, excluding).err())
            .flatten()
            .map(|error| InlineEditError::Validation(error.to_string()));
    }

    let frame = current.error.as_ref().map(|_| {
        Frame::new()
            .inner_margin(Margin::symmetric(4, 2))
            .fill(ui.visuals().text_edit_bg_color())
            .stroke(Stroke::new(1., ui.visuals().error_fg_color))
    });
    let mut editor = TextEdit::singleline(&mut current.value)
        .id_salt("layout_inline_name")
        .desired_width(ui.available_width());
    if let Some(frame) = frame {
        editor = editor.frame(frame);
    }
    let mut output = editor.show(ui);
    if current.request_focus {
        output.response.request_focus();
        let cursor = cursor_at_text_end(&current.value);
        output.state.cursor.set_char_range(Some(CCursorRange::one(cursor)));
        output.state.store(ui.ctx(), output.response.id);
        current.request_focus = false;
    }
    let response = output.response;

    if response.changed() {
        current.error = (!current.value.trim().is_empty())
            .then(|| layouts.validate_name(&current.value, excluding).err())
            .flatten()
            .map(|error| InlineEditError::Validation(error.to_string()));
    }
    if let Some(error) = &current.error {
        ui.painter().rect_stroke(
            response.rect,
            ui.visuals().widgets.inactive.corner_radius,
            Stroke::new(1., ui.visuals().error_fg_color),
            StrokeKind::Inside,
        );
        Tooltip::always_open(
            ui.ctx().clone(),
            response.layer_id,
            response.id.with("inline_layout_name_error"),
            response.rect,
        )
        .show(|ui| {
            ui.colored_label(ui.visuals().error_fg_color, error.message());
        });
    }

    let escape = (response.has_focus() || response.lost_focus())
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
    let enter_pressed =
        (response.has_focus() || response.lost_focus()) && ui.input(|input| input.key_pressed(Key::Enter));
    if enter_pressed {
        ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));
    }
    let explicit_submit = match inline_edit_intent(escape, enter_pressed, response.lost_focus()) {
        InlineEditIntent::Continue => {
            *edit = Some(current);
            return None;
        }
        InlineEditIntent::Cancel => return None,
        InlineEditIntent::Submit { explicit } => explicit,
    };

    let validation = layouts.validate_name(&current.value, excluding);
    let Ok(name) = validation else {
        if explicit_submit {
            current.error = Some(InlineEditError::Validation(validation.unwrap_err().to_string()));
            current.request_focus = true;
            *edit = Some(current);
        }
        return None;
    };

    let result = match &current.action {
        InlineEditAction::Create => layouts.create_empty(&name),
        InlineEditAction::Duplicate { source } => layouts.duplicate(source, &name),
        InlineEditAction::Rename { source } => layouts.rename(source, &name),
    };
    match result {
        Ok(slug) => {
            invalidate_search_cache(ui);
            match current.action {
                InlineEditAction::Create | InlineEditAction::Duplicate { .. } => Some(ViewTarget::Configuration),
                InlineEditAction::Rename { .. } => {
                    *selected = Some(slug);
                    None
                }
            }
        }
        Err(error) => {
            current.error = Some(InlineEditError::Store(error.to_string()));
            current.request_focus = true;
            *edit = Some(current);
            None
        }
    }
}

/// Converts editor input and focus changes into a naming action.
fn inline_edit_intent(escape: bool, enter: bool, lost_focus: bool) -> InlineEditIntent {
    if escape {
        InlineEditIntent::Cancel
    } else if enter {
        InlineEditIntent::Submit { explicit: true }
    } else if lost_focus {
        InlineEditIntent::Submit { explicit: false }
    } else {
        InlineEditIntent::Continue
    }
}

/// Returns a collapsed cursor positioned after the complete text value.
fn cursor_at_text_end(value: &str) -> CCursor {
    CCursor::new(value.chars().count())
}

/// Shows a compact deletion confirmation anchored to its Delete button.
fn show_delete_popup(
    ui: &mut Ui,
    layouts: &mut LayoutManager,
    anchor: &egui::Response,
    slug: &str,
    pending_delete: &mut Option<DeleteConfirmation>,
    selected: &mut Option<String>,
) -> Option<ViewTarget> {
    let Some(mut confirmation) = pending_delete.take() else {
        return None;
    };
    let mut open = true;
    let confirmed = DeleteConfirmationPopup::new(&mut open, anchor.rect.right_bottom())
        .pivot(Align2::RIGHT_TOP)
        .error(confirmation.error.as_deref())
        .show(ui);
    if confirmed {
        let was_active = layouts.active_slug() == Some(slug);
        match layouts.delete(slug) {
            Ok(()) => {
                invalidate_search_cache(ui);
                *selected = layouts.layouts().next().map(|layout| layout.slug.clone());
                if was_active {
                    return Some(ViewTarget::Welcome);
                }
            }
            Err(error) => {
                confirmation.error = Some(error.to_string());
                *pending_delete = Some(confirmation);
            }
        }
    } else if open {
        *pending_delete = Some(confirmation);
    }
    None
}

/// Activates a layout and closes the manager on success.
fn activate(ui: &Ui, layouts: &mut LayoutManager, slug: String, target: ViewTarget, owner: Id) -> Option<ViewTarget> {
    clear_control_error(ui, owner);
    match layouts.activate(&slug) {
        Ok(()) => Some(target),
        Err(error) => {
            set_control_error(ui, owner, error);
            None
        }
    }
}

/// Shows one label-value pair in the layout metadata panel.
fn metadata_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        let (label_rect, _) = ui.allocate_exact_size(Vec2::new(60., 18.), Sense::hover());
        let mut label_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(label_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        label_ui.label(RichText::new(label).weak());
        ui.label(value);
    });
}

#[cfg(test)]
mod tests {
    use egui::TextEdit;

    use super::{InlineEditIntent, cursor_at_text_end, inline_edit_intent, retain_search_focus};

    #[test]
    fn search_reclaims_focus_from_other_modal_controls() {
        egui::__run_test_ui(|ui| {
            let mut query = String::new();
            let search = ui.add(TextEdit::singleline(&mut query));
            let button = ui.button("Layout action");

            button.request_focus();
            retain_search_focus(&search, false);

            assert!(search.has_focus());
            assert!(!button.has_focus());
        });
    }

    #[test]
    fn search_yields_focus_to_an_inline_name_editor() {
        egui::__run_test_ui(|ui| {
            let mut query = String::new();
            let mut name = String::new();
            let search = ui.add(TextEdit::singleline(&mut query));
            let name_editor = ui.add(TextEdit::singleline(&mut name));

            name_editor.request_focus();
            retain_search_focus(&search, true);

            assert!(name_editor.has_focus());
            assert!(!search.has_focus());
        });
    }

    #[test]
    fn inline_edit_keyboard_and_focus_intents_are_prioritized() {
        // With no terminating input, the editor should remain active.
        assert_eq!(inline_edit_intent(false, false, false), InlineEditIntent::Continue);

        // Escape should take priority over simultaneous submission or focus loss.
        assert_eq!(inline_edit_intent(true, true, true), InlineEditIntent::Cancel);

        // Enter should be recorded as an explicit submission even when focus is also lost.
        assert_eq!(
            inline_edit_intent(false, true, true),
            InlineEditIntent::Submit { explicit: true }
        );

        // Focus loss alone should request an implicit submission.
        assert_eq!(
            inline_edit_intent(false, false, true),
            InlineEditIntent::Submit { explicit: false }
        );
    }

    #[test]
    fn duplicate_cursor_uses_the_end_of_the_complete_value() {
        // ASCII duplicate names should place the cursor after the complete suggested value.
        assert_eq!(cursor_at_text_end("Flight Copy").index, 11);

        // Cursor placement should count characters rather than UTF-8 bytes.
        assert_eq!(cursor_at_text_end("Café Copy").index, 9);
    }
}
