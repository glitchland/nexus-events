//! Thread-safe shared event bus implementation.
//!
//! This module provides a thread-safe wrapper around the core EventBus,
//! enabling event subscription and publishing from multiple threads.
//! It uses an Arc<Mutex<>> pattern to provide safe concurrent access.

use std::sync::{Arc, Mutex};
use std::any::TypeId;
use crate::core::bus::EventBus;
use crate::core::event::Event;
use crate::core::error::{EventError, EventResult};
use crate::subscriber::subscription::Subscription;

/// Thread-safe shared event bus that can be cloned and used across multiple threads.
///
/// `SharedEventBus` wraps the core `EventBus` implementation in an `Arc<Mutex<>>` to provide
/// thread-safe access. This allows multiple components to publish events and subscribe to them
/// without worrying about thread synchronization.
///
/// # Thread Safety
///
/// This struct is both `Send` and `Sync`, allowing it to be passed between threads and shared
/// with reference counting. The internal mutex ensures that operations on the event bus are atomic.
///
#[derive(Debug, Clone)]
pub struct SharedEventBus(Arc<Mutex<EventBus>>);

impl SharedEventBus {
    /// Creates a new, empty shared event bus.
    ///
    /// This creates a new `EventBus` instance wrapped in thread-safe containers
    /// that can be cloned and passed between threads.
    ///
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(EventBus::new())))
    }
    
    /// Subscribes to events of type `E` with an immutable handler.
    ///
    /// The handler is a function that receives a reference to the event.
    pub fn subscribe<E: Event + 'static, F>(&self, handler: F) -> Subscription
    where
        F: Fn(&E) + Send + Sync + 'static,
    {
        let handler_id = match self.0.lock() {
            Ok(mut bus) => bus.subscribe::<E, F>(handler),
            Err(_) => panic!("EventBus lock poisoned"),
        };

        Subscription::new(TypeId::of::<E>(), handler_id)
    }

    /// Subscribes to events of type `E` with a mutable handler.
    ///
    /// The handler is a function that can mutate its captured variables.
    pub fn subscribe_mut<E: Event + 'static, F>(&self, handler: F) -> Subscription
    where
        F: FnMut(&E) + Send + 'static,
    {
        let handler_id = match self.0.lock() {
            Ok(mut bus) => bus.subscribe_mut::<E, F>(handler),
            Err(_) => panic!("EventBus lock poisoned"),
        };

        Subscription::new(TypeId::of::<E>(), handler_id)
    }

    /// Publishes an event to all subscribers.
    ///
    /// This method sends an event to all handlers that are registered for this event's type.
    /// The event is passed by value and will be cloned if needed by handlers.
    pub fn publish<E: Event + 'static>(&self, event: E) -> EventResult<()> {
        match self.0.lock() {
            Ok(mut bus) => bus.publish(&event), // Note the "mut" keyword added here
            Err(_) => Err(EventError::PublishError { 
                details: "EventBus lock poisoned".to_string() 
            }),
        }
    }
}