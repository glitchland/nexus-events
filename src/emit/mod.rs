//! Event emission utilities and traits
//!
//! This module provides components for sending events through the event bus system.
//! The design focuses on making event emission ergonomic and type-safe through:
//!
//! 1. The `EventSender` struct for publishing events in a lightweight way
//! 2. The `EventEmitter` trait for components that need to send events
//! 3. Integration with the `#[event_sender]` attribute API
//!
//! Together, these components enable the clean attribute-based API for sending events,
//! where methods can be annotated with `#[event_sender(EventType)]` to automatically
//! publish events with their parameters.
//!
//! # Usage
//!
//! Components that need to send events should:
//!
//! 1. Have an `EventSender` field
//! 2. Implement the `EventEmitter` trait
//! 3. Use the `#[event_sender]` attribute on relevant methods
//!
//! ```rust
//! use nexus_events::{EventEmitter, EventSender, SharedEventBus, event_sender};
//!
//! #[derive(Debug)]
//! struct PlayerController {
//!     sender: EventSender,
//! }
//!
//! impl EventEmitter for PlayerController {
//!     fn sender(&self) -> &EventSender {
//!         &self.sender
//!     }
//! }
//!
//! impl PlayerController {
//!     fn new(event_bus: &SharedEventBus) -> Self {
//!         Self {
//!             sender: EventSender::new(event_bus.clone()),
//!         }
//!     }
//!
//!     // This method will automatically emit a PlayerMoved event
//!     #[event_sender(PlayerMoved)]
//!     fn move_player(&self, player_id: String, x: f32, y: f32) {
//!         // Pre-processing logic here
//!         println!("Moving player to ({}, {})", x, y);
//!         // Event is automatically sent after method returns
//!     }
//! }
//! ```

pub mod sender;

pub use sender::{EventSender, EventEmitter};