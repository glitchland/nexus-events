//! EventSubscriber trait for components that receive events
//! 
//! This trait defines the core interface for components that want to receive 
//! events using the attribute-based event system.

use std::fmt::Debug;
use crate::subscriber::subscription::SubscriptionSet;
use crate::core::shared::SharedEventBus;

/// The main trait for components that subscribe to events
///
/// This trait must be implemented by any component that wants to use the
/// attribute-based event system. It's typically derived using the
/// `#[derive(EventSubscriber)]` macro.
///
/// The trait provides methods for managing event subscriptions and lifecycle:
/// - Tracking whether the component is active
/// - Storing and managing subscriptions
/// - Registering event handlers when activated
/// - Cleaning up subscriptions when deactivated
///
/// # Implementation
///
/// Components implementing this trait should:
/// 1. Have an `id` field for identification
/// 2. Have an `active` field for tracking activation state
/// 3. Have a `subscriptions` field of type `SubscriptionSet`
/// 
/// # Example
///
/// ```
/// use nexus_events::{EventSubscriber, SubscriptionSet, SharedEventBus};
///
/// #[derive(Debug, EventSubscriber)]
/// struct MyComponent {
///     id: String,
///     active: bool,
///     subscriptions: SubscriptionSet,
/// }
///
/// impl MyComponent {
///     fn new(id: &str) -> Self {
///         Self {
///             id: id.to_string(),
///             active: false,
///             subscriptions: SubscriptionSet::new(),
///         }
///     }
///
///     fn activate(&mut self, event_bus: &SharedEventBus) {
///         self.active = true;
///         self.register_event_handlers(event_bus);
///     }
///
///     fn deactivate(&mut self) {
///         self.active = false;
///         self.clear_subscriptions();
///     }
/// }
/// ```
pub trait EventSubscriber: Debug {
    /// Returns the component's unique identifier
    ///
    /// This ID is typically used for debugging and logging purposes,
    /// as well as filtering events intended for this specific component.
    ///
    /// # Returns
    ///
    /// A string slice containing the component's ID
    fn id(&self) -> &str;
    
    /// Returns whether this component is currently active
    ///
    /// Active components receive events, while inactive ones don't.
    /// This method is used to check if event handlers should be registered.
    ///
    /// # Returns
    ///
    /// `true` if the component is active, `false` otherwise
    fn is_active(&self) -> bool;
    
    /// Returns a mutable reference to the subscription set
    ///
    /// This method provides access to the set of active subscriptions,
    /// allowing new subscriptions to be added or existing ones to be cleared.
    ///
    /// # Returns
    ///
    /// A mutable reference to the component's `SubscriptionSet`
    fn subscriptions(&mut self) -> &mut SubscriptionSet;
    
    /// Registers all event handlers annotated with #[event_handler]
    ///
    /// This method is called when a component is activated to register
    /// all of its event handlers with the event bus. The implementation
    /// is typically provided by the `#[derive(EventSubscriber)]` macro.
    ///
    /// # Arguments
    ///
    /// * `event_bus` - The event bus to register handlers with
    fn register_event_handlers(&mut self, event_bus: &SharedEventBus);
    
    /// Clears all subscriptions when the component is deactivated
    ///
    /// This method removes all event subscriptions from the event bus,
    /// ensuring that inactive components don't receive events.
    fn clear_subscriptions(&mut self) {
        self.subscriptions().clear();
    }
}