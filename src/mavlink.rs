//! This module contains all the structs and functions to work with Mavlink messages.
//!
//! It serves also as an abstraction wrapper around the `skyward_mavlink` crate, facilitating
//! rapid switching between different mavlink versions and profiles (_dialects_).

mod base;
mod error;
mod reflection;

// Export all the types from the base module as if they were defined in this module
pub use base::*;
pub use reflection::ReflectionContext;

/// Default port for the Ethernet connection
pub const DEFAULT_ETHERNET_PORT: u16 = 42069;
