use egui::{Align2, Id, Response, Tooltip, Ui};
use segs_memory::MemoryExt;

use crate::{
    layout::LayoutManager,
    ui::{
        modals::{
            LayoutManagerModal, LayoutManagerModalResponse, SaveDiscardModal, SaveDiscardModalChoice,
            SaveDiscardModalResponse,
        },
        popups::{SaveDiscardChoice, SaveDiscardConfirmationPopup},
        views::ViewTarget,
    },
};

const MANAGER_OPEN_ID: &str = "layout_manager_open";
const DIRTY_PROMPT_ID: &str = "layout_dirty_prompt";
const CLOSE_CONFIRMATION_ID: &str = "layout_close_confirmation";
const CONTROL_ERROR_ID: &str = "layout_control_error";
const TRANSITION_ID: &str = "layout_view_transition";
const CLOSE_REQUEST_ID: &str = "layout_close_request";

/// Records the operation to continue after unsaved changes are resolved.
#[derive(Clone, Debug)]
enum PendingAction {
    OpenManager,
    Transition(ViewTarget),
    Close,
}

/// Identifies which control owns the currently displayed dirty-layout prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirtyPromptOwner {
    StatusBar,
    DoneEditing,
}

/// Retains a guarded action and save error while its confirmation popup is open.
#[derive(Clone, Debug)]
struct DirtyPrompt {
    owner: DirtyPromptOwner,
    action: PendingAction,
    error: Option<String>,
}

/// Associates a layout operation failure with the control that triggered it.
#[derive(Clone, Debug)]
struct ControlError {
    owner: Id,
    message: String,
}

/// Opens the full layout manager.
fn open_manager(ui: &Ui, _layouts: &LayoutManager) {
    ui.mem().insert_temp(Id::new(MANAGER_OPEN_ID), true);
}

/// Opens the layout manager immediately or asks how to resolve unsaved changes.
pub fn request_open_manager(ui: &Ui, layouts: &LayoutManager) {
    if layouts.is_dirty() {
        clear_any_control_error(ui);
        set_dirty_prompt(
            ui,
            Some(DirtyPrompt {
                owner: DirtyPromptOwner::StatusBar,
                action: PendingAction::OpenManager,
                error: None,
            }),
        );
    } else {
        open_manager(ui, layouts);
    }
}

/// Renders the layout manager and any top-level close confirmation.
pub fn show_overlays(ui: &mut Ui, layouts: &mut LayoutManager) {
    if manager_is_open(ui) {
        let LayoutManagerModalResponse {
            should_close,
            transition,
        } = LayoutManagerModal::new(layouts).show(ui);
        if should_close {
            set_manager_open(ui, false);
            clear_any_control_error(ui);
        }
        if let Some(target) = transition {
            queue_transition(ui, target);
        }
    }
    if close_confirmation(ui) {
        let SaveDiscardModalResponse { choice, should_close } = SaveDiscardModal::new(layouts).show(ui);
        if should_close {
            set_close_confirmation(ui, false);
            clear_any_control_error(ui);
        }
        if let Some(choice) = choice {
            if choice == SaveDiscardModalChoice::Discard {
                layouts.discard_active();
            }
            set_close_confirmation(ui, false);
            clear_any_control_error(ui);
            execute_action(ui, layouts, PendingAction::Close);
        }
    }
}

/// Saves the active layout and associates any failure with the triggering control.
pub fn save_active(ui: &Ui, layouts: &mut LayoutManager, owner: Id) {
    clear_control_error(ui, owner);
    if let Err(error) = layouts.save_active() {
        set_control_error(ui, owner, error);
    }
}

/// Requests a transition from configuration to operation mode.
pub fn request_done_editing(ui: &Ui, layouts: &LayoutManager) {
    if layouts.is_dirty() {
        clear_any_control_error(ui);
        set_dirty_prompt(
            ui,
            Some(DirtyPrompt {
                owner: DirtyPromptOwner::DoneEditing,
                action: PendingAction::Transition(ViewTarget::Operator),
                error: None,
            }),
        );
    } else {
        execute_action(ui, layouts, PendingAction::Transition(ViewTarget::Operator));
    }
}

/// Shows the dirty-layout prompt above the status-bar layout control.
pub fn show_open_manager_prompt(ui: &mut Ui, layouts: &mut LayoutManager, anchor: &Response) {
    show_dirty_prompt(
        ui,
        layouts,
        DirtyPromptOwner::StatusBar,
        anchor.rect.left_top(),
        Align2::LEFT_BOTTOM,
    );
}

/// Shows the dirty-layout prompt below the Done Editing control.
pub fn show_done_editing_prompt(ui: &mut Ui, layouts: &mut LayoutManager, anchor: &Response) {
    show_dirty_prompt(
        ui,
        layouts,
        DirtyPromptOwner::DoneEditing,
        anchor.rect.right_bottom(),
        Align2::RIGHT_TOP,
    );
}

/// Requests application closing while preserving the dirty-layout guard.
pub fn request_close(ui: &Ui, layouts: &LayoutManager) {
    if layouts.is_dirty() {
        set_close_confirmation(ui, true);
    } else {
        execute_action(ui, layouts, PendingAction::Close);
    }
}

/// Returns and clears the next requested view transition.
pub fn take_transition(ui: &Ui) -> Option<ViewTarget> {
    ui.mem().remove_temp(Id::new(TRANSITION_ID))
}

/// Returns and clears a close request resolved during the previous frame.
pub fn take_close_request(ui: &Ui) -> bool {
    ui.mem().remove_temp::<bool>(Id::new(CLOSE_REQUEST_ID)).unwrap_or(false)
}

/// Shows a stored error tooltip when it belongs to the supplied control.
pub fn show_control_error(ui: &Ui, response: &Response) {
    let Some(error) = control_error(ui).filter(|error| error.owner == response.id) else {
        return;
    };
    Tooltip::always_open(
        ui.ctx().clone(),
        response.layer_id,
        response.id.with("layout_error_tooltip"),
        response.rect,
    )
    .show(|ui| {
        ui.colored_label(ui.visuals().error_fg_color, error.message);
    });
}

/// Returns whether the full layout manager is open.
fn manager_is_open(ui: &Ui) -> bool {
    ui.mem().get_temp_or_default(Id::new(MANAGER_OPEN_ID))
}

/// Updates the full layout manager visibility.
fn set_manager_open(ui: &Ui, open: bool) {
    ui.mem().insert_temp(Id::new(MANAGER_OPEN_ID), open);
}

/// Returns the pending dirty-layout prompt.
fn dirty_prompt(ui: &Ui) -> Option<DirtyPrompt> {
    ui.mem().get_temp(Id::new(DIRTY_PROMPT_ID))
}

/// Updates or clears the pending dirty-layout prompt.
fn set_dirty_prompt(ui: &Ui, prompt: Option<DirtyPrompt>) {
    if let Some(prompt) = prompt {
        ui.mem().insert_temp(Id::new(DIRTY_PROMPT_ID), prompt);
    } else {
        ui.mem().remove_temp::<DirtyPrompt>(Id::new(DIRTY_PROMPT_ID));
    }
}

/// Returns whether application closing awaits dirty-layout confirmation.
fn close_confirmation(ui: &Ui) -> bool {
    ui.mem().get_temp_or_default(Id::new(CLOSE_CONFIRMATION_ID))
}

/// Updates application-close confirmation visibility.
fn set_close_confirmation(ui: &Ui, open: bool) {
    ui.mem().insert_temp(Id::new(CLOSE_CONFIRMATION_ID), open);
}

/// Records an error for a particular UI control.
pub fn set_control_error(ui: &Ui, owner: Id, error: impl std::fmt::Display) {
    ui.mem().insert_temp(
        Id::new(CONTROL_ERROR_ID),
        ControlError {
            owner,
            message: error.to_string(),
        },
    );
}

/// Returns the current control-associated error.
fn control_error(ui: &Ui) -> Option<ControlError> {
    ui.mem().get_temp(Id::new(CONTROL_ERROR_ID))
}

/// Clears an error only when it belongs to the supplied control.
pub fn clear_control_error(ui: &Ui, owner: Id) {
    if control_error(ui).is_some_and(|error| error.owner == owner) {
        ui.mem().remove_temp::<ControlError>(Id::new(CONTROL_ERROR_ID));
    }
}

/// Clears any control-associated layout error.
pub fn clear_any_control_error(ui: &Ui) {
    ui.mem().remove_temp::<ControlError>(Id::new(CONTROL_ERROR_ID));
}

/// Resolves a dirty-layout choice from its anchored control.
fn show_dirty_prompt(
    ui: &mut Ui,
    layouts: &mut LayoutManager,
    owner: DirtyPromptOwner,
    pivot_pos: egui::Pos2,
    pivot_align: Align2,
) {
    let Some(mut prompt) = dirty_prompt(ui).filter(|prompt| prompt.owner == owner) else {
        return;
    };
    let mut open = true;
    let choice = SaveDiscardConfirmationPopup::new(&mut open, pivot_pos)
        .pivot(pivot_align)
        .error(prompt.error.as_deref())
        .show(ui);

    match choice {
        Some(SaveDiscardChoice::Save) => match layouts.save_active() {
            Ok(()) => {
                set_dirty_prompt(ui, None);
                execute_action(ui, layouts, prompt.action);
            }
            Err(error) => {
                prompt.error = Some(error.to_string());
                set_dirty_prompt(ui, Some(prompt));
            }
        },
        Some(SaveDiscardChoice::Discard) => {
            layouts.discard_active();
            set_dirty_prompt(ui, None);
            execute_action(ui, layouts, prompt.action);
        }
        None if !open => set_dirty_prompt(ui, None),
        None => {}
    }
}

/// Applies a resolved action and queues effects consumed by the application.
fn execute_action(ui: &Ui, layouts: &LayoutManager, action: PendingAction) {
    match action {
        PendingAction::OpenManager => open_manager(ui, layouts),
        PendingAction::Transition(target) => {
            set_manager_open(ui, false);
            queue_transition(ui, target);
        }
        PendingAction::Close => {
            ui.mem().insert_temp(Id::new(CLOSE_REQUEST_ID), true);
        }
    }
}

/// Queues a view transition without changing modal visibility.
fn queue_transition(ui: &Ui, target: ViewTarget) {
    ui.mem().insert_temp(Id::new(TRANSITION_ID), target);
}
