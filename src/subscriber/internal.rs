use crate::subscriber::registration_helper::{register_component_handlers, EventHandlerRegistration};
use crate::subscriber::subscription::SubscriptionSet;
use crate::core::shared::SharedEventBus;
use std::any::Any;

pub fn invoke_registration_methods<T: Any + EventHandlerRegistration>(
    component: &mut T,
    event_bus: &SharedEventBus,
    subscriptions: &mut SubscriptionSet,
) {
    register_component_handlers(component, event_bus, subscriptions);
}