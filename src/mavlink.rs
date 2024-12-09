mod base;
mod message_broker;
mod reflection;

// Export all the types from the base module as if they were defined in this module
pub use base::*;
pub use message_broker::{MessageBroker, MessageView};
pub use reflection::ReflectionContext;

pub const DEFAULT_ETHERNET_PORT: u16 = 42069;
