#[cfg(feature = "conrig")]
mod command_switch;
mod connections;
mod layouts;
mod shortcuts;

#[cfg(feature = "conrig")]
pub use command_switch::CommandSwitchWindow;
pub use connections::ConnectionsWindow;
pub use layouts::LayoutManagerWindow;
pub use shortcuts::ShortcutsWindow;
