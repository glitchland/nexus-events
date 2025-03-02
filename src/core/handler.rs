// Keep only this small part
use std::any::{Any, TypeId};
use std::fmt::Debug;

/// Trait for handling events.
/// This is used internally by the EventBus.
pub trait EventHandler: Debug {
    fn handle(&mut self, event: &dyn Any);
    fn event_type_id(&self) -> TypeId;
}