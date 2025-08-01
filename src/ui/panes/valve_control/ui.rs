mod valve_control_window;

use egui::{Key, KeyboardShortcut, Modifiers};

// Re-export the modules for the UI modules
use super::{commands, icons, valves};

pub use valve_control_window::ValveControlView;

#[inline]
pub fn map_key_to_shortcut(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::NONE, key)
}
