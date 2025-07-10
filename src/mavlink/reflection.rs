//! Reflection utilities for MAVLink messages.
//!
//! This module provides a reflection context that allows querying information about MAVLink messages
//! and their fields. This is useful for dynamically generating UI elements or performing generic
//! operations based on the available messages and fields. The reflection system is built around
//! a singleton [`ReflectionContext`] that provides access to MAVLink message definitions and
//! supports field lookup and manipulation via traits and helper types.

mod conversion;
mod fields;
mod profile;

use std::sync::LazyLock;

pub use skyward_mavlink::mavlink::reflection::{FieldType, MapConvertible, MessageMap};

use super::MAVLINK_PROFILE_SERIALIZED;

pub use conversion::FieldLike;
pub use fields::IndexedField;
pub use profile::ReflectionContext;

/// ReflectionContext singleton, used to get access to the Mavlink message definitions.
///
/// This static instance is lazily initialized and provides access to the MAVLink message
/// definitions loaded from the profile. It can be used everywhere to resolve message
/// details from user selection.
pub static MAVLINK_PROFILE: LazyLock<ReflectionContext> = LazyLock::new(ReflectionContext::new);

/// Trait for looking up fields in a MAVLink message map.
///
/// This trait abstracts the process of retrieving and mutating fields in a [`MessageMap`]
/// using a field identifier that implements [`FieldLike`]. It provides both immutable and
/// mutable access to fields, supporting type conversion via [`TryFrom`].
pub trait FieldLookup {
    /// Retrieves a field from the message map by field identifier.
    ///
    /// # Type Parameters
    /// - `T`: The target type to convert the field to. Must implement `TryFrom<&FieldType>`.
    /// - `F`: The field identifier type. Must implement [`FieldLike`].
    ///
    /// # Arguments
    /// - `field`: The field identifier.
    ///
    /// # Returns
    /// - `Some(T)` if the field exists and conversion succeeds.
    /// - `None` if the field does not exist or conversion fails.
    fn get_field<'a, T: TryFrom<&'a FieldType>, F: FieldLike>(&'a self, field: F) -> Option<T>;

    /// Retrieves a mutable reference to a field from the message map by field identifier.
    ///
    /// # Type Parameters
    /// - `T`: The target type to convert the field to (by mutable reference).
    /// - `F`: The field identifier type. Must implement [`FieldLike`].
    ///
    /// # Arguments
    /// - `field`: The field identifier.
    ///
    /// # Returns
    /// - `Some(&mut T)` if the field exists and conversion succeeds.
    /// - `None` if the field does not exist or conversion fails.
    fn get_mut_field<'a, 'b, T, F>(&'a mut self, field: F) -> Option<&'a mut T>
    where
        'b: 'a,
        T: 'b,
        &'a mut T: TryFrom<&'a mut FieldType>,
        F: FieldLike;
}

impl FieldLookup for MessageMap {
    /// Retrieves a field from the message map by field identifier.
    ///
    /// This implementation uses the [`FieldLike`] trait to resolve the field metadata,
    /// then attempts to retrieve and convert the field value from the underlying field map.
    ///
    /// # Example
    /// ```
    /// let value: Option<u32> = message_map.get_field("some_field_name");
    /// ```
    fn get_field<'a, T: TryFrom<&'a FieldType>, F: FieldLike>(&'a self, field: F) -> Option<T> {
        // Convert the field identifier to a MAVLink field using the reflection context.
        let field = field
            .to_mav_field(self.message_id(), &MAVLINK_PROFILE)
            .ok()?;
        // Retrieve the field from the field map and attempt conversion.
        self.field_map()
            .get(field.id())
            .and_then(|f| T::try_from(f).ok())
    }

    /// Retrieves a mutable reference to a field from the message map by field identifier.
    ///
    /// This implementation uses the [`FieldLike`] trait to resolve the field metadata,
    /// then attempts to retrieve and convert the field value from the underlying field map
    /// as a mutable reference.
    ///
    /// # Example
    /// ```
    /// if let Some(val) = message_map.get_mut_field::<u32, _>("some_field_name") {
    ///     *val = 42;
    /// }
    /// ```
    fn get_mut_field<'a, 'b, T, F>(&'a mut self, field: F) -> Option<&'a mut T>
    where
        'b: 'a,
        T: 'b,
        &'a mut T: TryFrom<&'a mut FieldType>,
        F: FieldLike,
    {
        // Convert the field identifier to a MAVLink field using the reflection context.
        let field = field
            .to_mav_field(self.message_id(), &MAVLINK_PROFILE)
            .ok()?;
        // Get a mutable reference to the field map.
        let mut field_map = self.field_map_mut();
        // Check if the field id is within bounds.
        if field_map.len() <= field.id() {
            return None;
        }
        // Attempt to remove the field and convert it to the desired mutable reference type.
        <&mut T>::try_from(field_map.remove(field.id())).ok()
    }
}
