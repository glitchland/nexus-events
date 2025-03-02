use std::any::Any;
use crate::subscriber::subscription::SubscriptionSet;
use crate::core::shared::SharedEventBus;

/// Implementation detail: Dynamically calls all registration methods on a type
pub fn invoke_registration_methods<T: Any>(
    instance: &mut T, 
    event_bus: &SharedEventBus,
    subscriptions: &mut SubscriptionSet
) {
    // Get type name for debug purposes
    let type_name = std::any::type_name::<T>();
    
    // Use a runtime reflection approach with a predictable pattern:
    // Each event handler method has a corresponding __register_X method
    
    // Get a vector of function pointers to all register functions
    let register_fns = discover_registration_methods::<T>();
    
    // Call each registration function
    for register_fn in register_fns {
        if let Some(subscription) = register_fn(instance, event_bus) {
            subscriptions.add(subscription);
        }
    }
}