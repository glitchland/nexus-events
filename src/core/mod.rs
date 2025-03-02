//! Core event system functionality
//!
//! This module contains the foundational components of the nexus-events system:
//! 
//! - `EventBus`: The central hub that manages event subscriptions and publishing
//! - `Event`: The trait that all event types must implement
//! - `EventError`/`EventResult`: Error handling for the event system
//! - `SharedEventBus`: Thread-safe wrapper around EventBus for concurrent usage
//! - `EventPayload`: Trait for event data structures
//! 
//! Together, these components form the backbone of the attribute-based event system,
//! providing the low-level functionality that powers the higher-level attribute API.

/// Event bus implementation and handler management
pub mod bus;

/// Event trait definition and core types
pub mod event;

/// Event handler traits and implementations
pub mod handler;

/// Error types and result wrappers
pub mod error;

/// Core event types and related traits
pub mod types;

/// Thread-safe shared event bus implementation
pub mod shared;

// Re-exports of core types for public API

/// Central event bus that manages subscriptions and event publishing
pub use bus::{EventBus, HandlerId};

/// Trait that all event types must implement
pub use event::Event;

/// Error handling types for the event system
pub use error::{EventError, EventResult};

/// Thread-safe wrapper around EventBus for concurrent usage
pub use shared::SharedEventBus;

/// Type definitions for events and payloads
pub use types::{EventPayload, EventTypeId};

pub use handler::EventHandler;