//! Event sender utilities for simplified event publishing
//!
//! This module provides a more ergonomic interface for components to publish events
//! without directly interacting with the EventBus.

use std::fmt::Debug;
use crate::core::{Event, EventResult};
use crate::core::shared::SharedEventBus;

/// Helper for sending events that wraps an event bus
///
/// `EventSender` provides a simplified interface for components to publish events.
/// It's typically owned by a component that needs to emit events, and accessed
/// through the `EventEmitter` trait.
///
/// # Examples
///
/// ```
/// use nexus_events::{EventSender, SharedEventBus, define_event};
///
/// define_event!(PlayerDamaged { player_id: String, amount: u32 });
///
/// let event_bus = SharedEventBus::new();
/// let sender = EventSender::new(event_bus);
///
/// sender.emit(PlayerDamaged {
///     player_id: "player1".to_string(),
///     amount: 10
/// }).expect("Failed to emit event");
/// ```
#[derive(Debug, Clone)]
pub struct EventSender {
    event_bus: SharedEventBus,
}

impl EventSender {
    /// Creates a new EventSender that wraps the given event bus
    ///
    /// # Arguments
    ///
    /// * `event_bus` - The SharedEventBus to publish events through
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_events::{EventSender, SharedEventBus};
    ///
    /// let event_bus = SharedEventBus::new();
    /// let sender = EventSender::new(event_bus.clone());
    /// ```
    pub fn new(event_bus: SharedEventBus) -> Self {
        Self { event_bus }
    }
    
    /// Emits an event to all subscribers
    ///
    /// This method publishes an event through the wrapped EventBus.
    /// The method is named "emit" to match the #[event_sender] attribute semantics.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The event type to emit
    ///
    /// # Arguments
    ///
    /// * `event` - The event instance to publish
    ///
    /// # Returns
    ///
    /// `Ok(())` if the event was successfully published, or an error if there was a problem
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_events::{EventSender, SharedEventBus, define_event};
    ///
    /// define_event!(PlayerDied { player_id: String });
    ///
    /// let event_bus = SharedEventBus::new();
    /// let sender = EventSender::new(event_bus);
    ///
    /// sender.emit(PlayerDied { player_id: "player1".to_string() })
    ///     .expect("Failed to emit event");
    /// ```
    pub fn emit<E: Event + 'static>(&self, event: E) -> EventResult<()> {
        self.event_bus.publish(event)
    }
    
    /// Returns a reference to the underlying event bus
    ///
    /// This method provides access to the wrapped SharedEventBus when
    /// more direct control is needed.
    ///
    /// # Returns
    ///
    /// A reference to the wrapped SharedEventBus
    pub fn event_bus(&self) -> &SharedEventBus {
        &self.event_bus
    }
}

/// Trait for components that can emit events
///
/// This trait is implemented by components that need to send events.
/// It provides a standard way to access the component's EventSender.
///
/// # Examples
///
/// ```
/// use nexus_events::{EventEmitter, EventSender, SharedEventBus};
///
/// struct MyComponent {
///     id: String,
///     sender: EventSender,
/// }
///
/// impl EventEmitter for MyComponent {
///     fn sender(&self) -> &EventSender {
///         &self.sender
///     }
/// }
///
/// impl MyComponent {
///     fn new(id: impl Into<String>, event_bus: &SharedEventBus) -> Self {
///         Self {
///             id: id.into(),
///             sender: EventSender::new(event_bus.clone()),
///         }
///     }
/// }
/// ```
pub trait EventEmitter: Debug {
    /// Returns a reference to the component's event sender
    fn sender(&self) -> &EventSender;
}