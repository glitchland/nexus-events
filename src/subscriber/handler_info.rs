//! Event handler metadata and function types
//!
//! This module provides the types used by the attribute macros to store
//! information about event handlers and register them with the system.

use std::any::{Any, TypeId};

/// Function pointer type for event handler methods
///
/// This type represents a function that can handle events for a component of type `T`.
/// The handler receives mutable access to the component instance and a reference to 
/// the event (as Any, which will be downcast to the correct type).
///
/// # Type Parameters
///
/// * `T` - The component type that owns this handler
pub type HandlerFn<T> = for<'a> fn(&'a mut T, &'a dyn Any);

/// Information about an event handler method
///
/// This structure stores metadata about handler methods that are decorated with
/// the `#[event_handler]` attribute. It's created at compile time by the attribute
/// macro and registered with the `EventHandlerRegistry`.
///
/// # Type Parameters
///
/// * `T` - The component type that owns this handler
///
/// # Fields
///
/// * `name` - The name of the handler method, used for debugging
/// * `handler_fn` - The function pointer to the actual handler implementation
/// * `event_type_id` - The TypeId of the event this handler processes
#[derive(Debug)]
pub struct EventHandlerInfo<T> {
    /// Name of the handler function for debugging
    pub name: &'static str,
    
    /// Function pointer to the handler implementation
    pub handler_fn: HandlerFn<T>,
    
    /// TypeId of the event this handler processes
    pub event_type_id: TypeId,
}