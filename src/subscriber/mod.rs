pub mod subscription;
pub mod event_subscriber;
pub mod handler_info;
pub mod registry;
pub mod registration;

// Re-export the core types
pub use event_subscriber::EventSubscriber;
pub use handler_info::{EventHandlerInfo, HandlerFn};
pub use registry::EventHandlerRegistry;
pub use subscription::{Subscription, SubscriptionSet};

// Create a module namespace for internal APIs
#[doc(hidden)]
pub mod _internal {
    pub use super::registration;
}