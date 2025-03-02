//! Helper for automatic event handler registration
//!
//! This module provides the core functionality that powers the attribute-based
//! event handler registration system without requiring manual intervention.

use std::any::Any;
use crate::core::shared::SharedEventBus;
use crate::subscriber::subscription::SubscriptionSet;

/// Trait implemented by EventSubscriber derive macro for components
///
/// This trait is automatically implemented by the EventSubscriber derive macro
/// and allows components to register their event handlers with the event system.
pub trait EventHandlerRegistration {
    /// Register all event handlers for this component
    fn register_handlers(&mut self, event_bus: &SharedEventBus) -> Vec<crate::subscriber::subscription::Subscription>;
}

/// Register all event handlers for a component automatically
///
/// This function is called by the EventSubscriber implementation to register
/// all methods annotated with #[event_handler] and return their subscriptions.
pub fn register_component_handlers<T>(component: &mut T, event_bus: &SharedEventBus, subscriptions: &mut SubscriptionSet)
where
    T: Any + EventHandlerRegistration,
{
    let new_subs = component.register_handlers(event_bus);
    
    for sub in new_subs {
        subscriptions.add(sub);
    }
}