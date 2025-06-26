use std::collections::HashSet;

use egui::{Context, Key, KeyboardShortcut, Modifiers};

/// Contains all keyboard shortcuts added by the UI.
///
/// [`ShortcutHandler`] is used to register shortcuts and consume them, while
/// keeping tracks of all enabled shortcuts and filter active shortcut based on
/// UI views and modes (see [`ShortcutModeStack`]).
#[derive(Debug, Clone)]
pub struct ShortcutHandler {
    /// The egui context. Needed to consume shortcuts.
    ctx: Context,

    /// Set of all enabled shortcuts.
    enabled_shortcuts: HashSet<KeyboardShortcut>,

    /// Stack layers of keyboard shortcuts. Controls which shortcuts are active at any given time.
    mode_stack: ShortcutModeStack,
}

impl ShortcutHandler {
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx,
            enabled_shortcuts: Default::default(),
            mode_stack: Default::default(),
        }
    }

    fn add_shortcut_action_pair<A>(
        &mut self,
        modifier: Modifiers,
        key: Key,
        action: A,
        mode: ShortcutMode,
    ) -> Option<A> {
        let shortcut = KeyboardShortcut::new(modifier, key);
        if self.mode_stack.is_active(mode) {
            let action = self
                .ctx
                .input_mut(|i| i.consume_shortcut(&shortcut).then_some(action));
            self.enabled_shortcuts.insert(shortcut);
            action
        } else {
            None
        }
    }

    /// Consume the keyboard shortcut provided and return the action associated
    /// with it if the active mode is the provided one.
    pub fn consume_if_mode_is<A: Clone>(
        &mut self,
        mode: ShortcutMode,
        shortcuts: &[(Modifiers, Key, A)],
    ) -> Option<A> {
        for (modifier, key, action) in shortcuts {
            if let Some(action) = self.add_shortcut_action_pair(*modifier, *key, action, mode) {
                return Some(action.clone());
            };
        }
        None
    }

    /// Activate a mode (see [`ShortcutModeStack`] for more).
    #[inline]
    pub fn activate_mode(&mut self, mode: ShortcutMode) {
        if !self.mode_stack.is_active(mode) {
            self.mode_stack.activate(mode);
            self.enabled_shortcuts.clear();
        }
    }

    /// Deactivate a mode, switching back to the previous layer (if any).
    #[inline]
    pub fn deactivate_mode(&mut self, mode: ShortcutMode) {
        if self.mode_stack.is_active(mode) {
            self.mode_stack.deactivate(mode);
            self.enabled_shortcuts.clear();
        }
    }

    pub fn is_active(&self, mode: ShortcutMode) -> bool {
        self.mode_stack.is_active(mode)
    }
}

/// Stack layers of keyboard shortcuts. Controls which shortcuts are active at any given time.
///
/// The first layer is the default layer, which is active when the user is in the main view.
/// The second layer is active when the user is in a modal/dialog/window that needs full keyboard control.
/// When the modal/dialog/window is closed the second layer is removed and the first layer is active again.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ShortcutModeStack {
    first: FirstLayerModes,
    second: Option<SecondLayerModes>,
}

impl ShortcutModeStack {
    fn is_active(&self, mode: ShortcutMode) -> bool {
        match mode {
            ShortcutMode::FirstLayer(first) => self.first == first && self.second.is_none(),
            ShortcutMode::SecondLayer(second) => self.second == Some(second),
        }
    }

    fn activate(&mut self, mode: ShortcutMode) {
        match mode {
            ShortcutMode::FirstLayer(first) => {
                self.first = first;
                self.second = None;
            }
            ShortcutMode::SecondLayer(second) => self.second = Some(second),
        }
    }

    fn deactivate(&mut self, mode: ShortcutMode) {
        match mode {
            ShortcutMode::FirstLayer(first) => {
                if self.first == first {
                    self.first = FirstLayerModes::default();
                }
            }
            ShortcutMode::SecondLayer(second) => {
                if self.second == Some(second) {
                    self.second = None;
                }
            }
        }
    }
}

/// Layers of keyboard shortcuts. See [`ShortcutModeStack`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutMode {
    FirstLayer(FirstLayerModes),
    SecondLayer(SecondLayerModes),
}

impl ShortcutMode {
    #[inline]
    pub fn composition() -> Self {
        Self::FirstLayer(FirstLayerModes::Composition)
    }

    #[inline]
    pub fn operation() -> Self {
        Self::FirstLayer(FirstLayerModes::Operation)
    }

    #[inline]
    pub fn valve_control() -> Self {
        Self::SecondLayer(SecondLayerModes::ValveControl)
    }
}

/// First layer of keyboard shortcuts.
///
/// Active when the user is on the main view choosing how to customize their layout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FirstLayerModes {
    /// Shortcuts that are active when the user is in the main menu.
    #[default]
    Composition,
    Operation,
}

/// Second layer of keyboard shortcuts, sits on top of the first layer.
///
/// Active when the user is in a modal, dialog or window that needs full keyboard control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondLayerModes {
    /// Shortcuts that are active when the user is in the main menu.
    ValveControl,
}
