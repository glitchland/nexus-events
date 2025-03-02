use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::core::error::{EventError, EventResult};
use crate::core::event::Event;

enum HandlerEntry {
    /// Standard immutable handler function
    Immutable(Box<dyn Fn(&dyn Any) + Send + Sync>),
    /// Handler that can mutate its captured component
    Mutable(Box<dyn FnMut(&dyn Any) + Send + Sync>),
}

/// Type alias for unique handler identification.
/// 
/// Handler IDs are used to track and manage event subscriptions,
/// allowing specific handlers to be unsubscribed later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerId(pub usize);

/// The central hub for event publishing and subscription.
/// 
/// EventBus is the core component of the event system that:
/// - Manages subscriptions to different event types
/// - Dispatches events to registered handlers
/// - Provides type-safe event handling through Rust's type system
/// - Enables decoupled communication between components
/// 
/// # Thread Safety
/// 
/// This implementation is designed to be wrapped in thread-safe containers
/// like Arc<Mutex<>> when used across multiple threads.
pub struct EventBus {
    handlers: HashMap<TypeId, Vec<(HandlerId, Box<dyn Fn(&dyn Any) + Send + Sync>)>>,
    next_handler_id: AtomicUsize,
}

impl Default for EventBus {
    fn default() -> Self {
        Self {
            handlers: HashMap::new(),
            next_handler_id: AtomicUsize::new(0),
        }
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("handlers", &format!("{} event types registered", self.handlers.len()))
            .field("next_handler_id", &self.next_handler_id)
            .finish()
    }
}

impl EventBus {
    /// Creates a new, empty EventBus with no registered handlers.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use nexus_events::EventBus;
    /// 
    /// let event_bus = EventBus::new();
    /// ```
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            next_handler_id: AtomicUsize::new(0),
        }
    }

    /// Subscribe to a specific event type with a handler function.
    /// 
    /// This method registers a handler function to be called whenever an event
    /// of type `E` is published. It returns a unique handler ID that can be used
    /// to unsubscribe the handler later.
    /// 
    /// # Type Parameters
    /// 
    /// * `E` - The event type to subscribe to
    /// * `F` - The handler function type
    /// 
    /// # Arguments
    /// 
    /// * `handler` - A function that will be called when events of type `E` are published
    /// 
    /// # Returns
    /// 
    /// A unique `HandlerId` that can be used to unsubscribe this handle
    /// 
    /// # Thread Safety
    /// This method uses atomic operations to ensure handler IDs are unique
    /// across threads when used with `SharedEventBus`.
    /// ```
    pub fn subscribe<E: Event + 'static, F>(&mut self, handler: F) -> HandlerId
    where
        F: Fn(&E) + Send + Sync + 'static,
    {
        let handler_id = HandlerId(self.next_handler_id.fetch_add(1, Ordering::SeqCst));
        let type_id = TypeId::of::<E>();
        
        let boxed_handler = Box::new(move |event: &dyn Any| {
            if let Some(typed_event) = event.downcast_ref::<E>() {
                handler(typed_event);
            }
        });
        
        self.handlers.entry(type_id)
            .or_default()
            .push((handler_id, boxed_handler));
        
        handler_id
    }

    /// Subscribe to any event of a specific type with a raw handler function
    /// 
    /// This is primarily used by the attribute macro system and should rarely be called directly.
    /// 
    /// # Arguments
    /// 
    /// * `type_id` - TypeId of the event to subscribe to
    /// * `handler` - Function to call when events of this type are published
    /// 
    /// # Returns
    /// 
    /// A unique handler ID that can be used to unsubscribe later
    /// 
    /// # Thread Safety
    /// This method uses atomic operations to ensure handler IDs are unique
    /// across threads when used with `SharedEventBus`.
    pub fn subscribe_any(
        &mut self,
        type_id: TypeId,
        handler: Box<dyn Fn(&dyn Any) + Send + Sync>,
    ) -> HandlerId {
        let handler_id = HandlerId(self.next_handler_id.fetch_add(1, Ordering::SeqCst));
        
        self.handlers.entry(type_id)
            .or_default()
            .push((handler_id, HandlerEntry::Immutable(handler)));
        
        handler_id
    }

    /// Subscribe with a handler that can mutate its environment
    pub fn subscribe_any_mut(
        &mut self,
        type_id: TypeId,
        mut handler: Box<dyn FnMut(&dyn Any) + Send + Sync>,
    ) -> HandlerId {
        let handler_id = HandlerId(self.next_handler_id.fetch_add(1, Ordering::SeqCst));
        
        self.handlers.entry(type_id)
            .or_default()
            .push((handler_id, HandlerEntry::Mutable(handler)));
        
        handler_id
    }

    /// Publish an event to all subscribers.
    /// 
    /// This method dispatches the provided event to all handlers that are
    /// registered for the event's type. The event is passed by reference to
    /// avoid unnecessary clones.
    /// 
    /// # Type Parameters
    /// 
    /// * `E` - The event type being published
    /// 
    /// # Arguments
    /// 
    /// * `event` - The event to publish
    /// 
    /// # Returns
    /// 
    /// `Ok(())` if the event was successfully published, or an error if
    /// something went wrong during publishing
    /// 
    /// # Examples
    /// 
    /// ```
    /// use nexus_events::{EventBus, GameEvent};
    /// 
    /// let event_bus = EventBus::new();
    /// let result = event_bus.publish(&GameEvent::GameStart { 
    ///     difficulty: 3, 
    ///     level: 1 
    /// });
    /// assert!(result.is_ok());
    /// ```
    pub fn publish<E: Event + 'static>(&mut self, event: &E) -> EventResult<()> {
        let type_id = TypeId::of::<E>();
        
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            for (_, handler) in handlers {
                match handler {
                    HandlerEntry::Immutable(func) => func(event),
                    HandlerEntry::Mutable(func) => func(event),
                }
            }
        }
        
        Ok(())
    }

    /// Unsubscribe a specific handler using its ID.
    /// 
    /// This method efficiently removes a subscription from the event bus by its handler ID.
    /// If the handler is found and removed, it returns Ok(()).
    /// If the handler cannot be found, it returns an appropriate error.
    ///
    /// # Type Parameters
    /// * `E` - The event type to unsubscribe from
    ///
    /// # Arguments
    /// * `handler_id` - The ID of the handler to remove
    ///
    /// # Returns
    /// * `Ok(())` if the handler was successfully removed
    /// * `Err(EventError::HandlerNotFound)` if no handler with this ID exists for the event type
    /// * `Err(EventError::EventTypeNotFound)` if no handlers exist for this event type
    pub fn unsubscribe<E: 'static>(&mut self, handler_id: HandlerId) -> EventResult<()> {
        let type_id = TypeId::of::<E>();
        
        // Try to get the handlers for this event type
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            // Find the index of the handler with the matching ID
            // We need to track the index to efficiently remove it from the Vec
            // Since we don't store handler IDs directly, we'll need to use another approach
            
            // We can't directly check box contents, so we'll use swap_remove for O(1) removal
            // This approach keeps high performance by avoiding shifts in the Vec
            // Note: This changes element order, but order doesn't matter for handlers
            
            // Remove all handlers for this ID (should be just one)
            let before_len = handlers.len();
            
            // Using retain is more efficient than finding and removing
            // It only does one pass through the array
            handlers.retain(|(_id, _)| _id.0 != handler_id.0);
            
            let removed = handlers.len() < before_len;
            
            // If we've removed all handlers for this event type, remove the entry
            if handlers.is_empty() {
                self.handlers.remove(&type_id);
            }
            
            if removed {
                Ok(())
            } else {
                Err(EventError::HandlerNotFound { id: handler_id.0 })
            }
        } else {
            Err(EventError::EventTypeNotFound)
        }
    }
}