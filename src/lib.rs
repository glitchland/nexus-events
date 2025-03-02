//! Event bus system for decoupled communication between game components.
//!
//! # Overview
//!
//! Nexus Events provides a clean, attribute-based event system for game development,
//! allowing components to communicate without direct dependencies. It focuses on
//! type safety, performance, and ergonomics.
//!
//! # Key Features
//!
//! - **Attribute-based API**: Use simple attributes to define event handlers and senders
//! - **Type safety**: Leverages Rust's type system for compile-time safety
//! - **Thread safety**: Event bus can be shared across threads
//! - **RAII subscriptions**: Automatic cleanup when components are dropped
//! - **Low overhead**: Designed for performance in game environments
//!
//! # Getting Started
//!
//! 1. Define component types with `#[derive(EventSubscriber)]`
//! 2. Define event handler methods with `#[event_handler(EventType)]`
//! 3. Define event sending methods with `#[event_sender(EventType)]`
//!
//! Your components can now handle and send events through a clean, attribute-based API.
//!
//! # Basic Example
//! 
//! ```rust
//! use nexus_events::prelude::*;
//! 
//! // Define an event
//! define_event!(PlayerMoved { player_id: String, x: f32, y: f32 });
//! 
//! // Define a component that subscribes to events
//! #[derive(Debug, EventSubscriber)]
//! struct PlayerComponent {
//!     id: String,
//!     active: bool,
//!     subscriptions: SubscriptionSet,
//!     position: (f32, f32),
//!     sender: EventSender,
//! }
//! 
//! impl EventEmitter for PlayerComponent {
//!     fn sender(&self) -> &EventSender {
//!         &self.sender
//!     }
//! }
//! 
//! impl PlayerComponent {
//!     fn new(id: impl Into<String>, event_bus: &SharedEventBus) -> Self {
//!         Self {
//!             id: id.into(),
//!             active: false,
//!             subscriptions: SubscriptionSet::new(),
//!             position: (0.0, 0.0),
//!             sender: EventSender::new(event_bus.clone()),
//!         }
//!     }
//!     
//!     fn activate(&mut self, event_bus: &SharedEventBus) {
//!         self.active = true;
//!         self.register_event_handlers(event_bus);
//!     }
//!     
//!     // Define event handler with attribute
//!     #[event_handler(PlayerMoved)]
//!     fn handle_movement(&mut self, event: &PlayerMoved) {
//!         if event.player_id == self.id {
//!             self.position = (event.x, event.y);
//!         }
//!     }
//!     
//!     // Define event sender with attribute
//!     #[event_sender(PlayerMoved)]
//!     fn move_to(&self, player_id: String, x: f32, y: f32) {
//!         // Any preprocessing logic here
//!         println!("Moving to ({}, {})", x, y);
//!         // Event is automatically sent after method returns
//!     }
//! }
//! ```
//!
//! # Module Structure
//!
//! - `core`: Core event system functionality (bus, events, errors)
//! - `subscriber`: Event subscription and handler registration
//! - `emit`: Event sending utilities
//! - `macros`: Runtime macros for event definition

// Module imports
pub mod core;
pub mod subscriber;
pub mod emit;
pub mod macros;

// Re-export proc-macros from companion crate
/// Attribute for marking methods as event handlers.
///
/// Add this attribute to methods in `EventSubscriber` implementations to handle specific event types.
///
pub use nexus_events_macros::event_handler;

/// Attribute for marking methods as event senders.
///
/// Add this attribute to methods in types implementing `EventEmitter` to send events.
///
pub use nexus_events_macros::event_sender;

/// Derive macro for implementing the `EventSubscriber` trait.
///
/// This macro automatically implements the `EventSubscriber` trait for a struct,
/// enabling it to register event handlers with the `#[event_handler]` attribute.
///
/// # Requirements
///
/// The struct must have the following fields:
/// - `id`: A string identifier
/// - `active`: A boolean indicating if the component is active
/// - `subscriptions`: A `SubscriptionSet` to store subscriptions
///
/// # Example
///
/// ```
/// #[derive(Debug, EventSubscriber)]
/// struct MyComponent {
///     id: String,
///     active: bool,
///     subscriptions: SubscriptionSet,
/// }
/// ```
pub use nexus_events_macros::EventSubscriber;

// Core exports
pub use core::error::{EventError, EventResult};
pub use core::event::Event;
pub use core::bus::{EventBus, HandlerId};
pub use core::types::EventPayload;
pub use core::shared::SharedEventBus;

// Subscriber exports
pub use subscriber::EventHandlerInfo;
pub use subscriber::SubscriptionSet;

// Emit exports
pub use emit::sender::{EventSender, EventEmitter};


/// Convenience re-exports for commonly used types and functions.
///
/// Import this module with `use nexus_events::prelude::*;` to get access
/// to all the essential components of the event system.
pub mod prelude {
    pub use crate::core::error::EventResult;
    pub use crate::core::event::Event;
    pub use crate::core::bus::EventBus;
    pub use crate::core::types::EventPayload;
    pub use crate::core::shared::SharedEventBus;
    
    pub use crate::subscriber::EventSubscriber;
    pub use crate::subscriber::{Subscription, SubscriptionSet};
    
    pub use crate::emit::sender::{EventSender, EventEmitter};
    
    // Convenience macros
    pub use crate::event_handler;
    pub use crate::event_sender;
    pub use crate::define_event;
    
    // Re-export derive macros
    pub use crate::EventSubscriber;
}

// Backward compatibility aliases for existing code
#[doc(hidden)]
pub use subscriber::subscription::SubscriptionSet as EventSubscriptionSet;

#[doc(hidden)]
pub use crate::subscriber::invoke_registration_methods;
