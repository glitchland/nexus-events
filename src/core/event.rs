use std::fmt::Debug;
use std::any::Any;

/// A trait for all event types in the system.
/// 
/// This trait acts as a marker for types that can be published and subscribed to.
/// All events must be: Debug (for logging) and 'static.
/// 
/// The `as_any` method enables type-safe downcasting for event handling.
pub trait Event: Debug + 'static {
    /// Returns a reference to self as Any for type casting
    fn as_any(&self) -> &dyn Any;
}

// Blanket implementation for any type that meets the trait bounds
impl<T> Event for T where T: Debug + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

