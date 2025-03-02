pub mod subscription;
pub mod event_subscriber;
pub mod handler_info;
pub mod registry;
pub mod registration;
mod registration_helper;
mod internal;

// Re-export the core types
pub use event_subscriber::EventSubscriber;
pub use handler_info::{EventHandlerInfo, HandlerFn};
pub use registry::EventHandlerRegistry;
pub use subscription::{Subscription, SubscriptionSet};

pub use internal::invoke_registration_methods;