//! Event definitions for the event bus system

use std::fmt::Debug;
use std::any::TypeId;

/// A unique ID for event types
pub type EventTypeId = TypeId;

/// Trait that must be implemented by all event payload types
pub trait EventPayload: Debug + Clone + 'static {}

// Blanket implementation for compatible types
impl<T> EventPayload for T where T: Debug + Clone + 'static {}
