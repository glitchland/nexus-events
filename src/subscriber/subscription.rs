//! Subscription management for the event system
//!
//! This module provides types for tracking and managing event subscriptions
//! with automatic cleanup when they go out of scope. It uses RAII (Resource
//! Acquisition Is Initialization) principles to ensure that event handlers are
//! properly deactivated when no longer needed, preventing memory leaks and
//! dangling references.
//!
//! The primary types are:
//! - `Subscription`: Represents a single event subscription
//! - `SubscriptionSet`: A collection of subscriptions with automatic cleanup
//!
//! # Thread Safety
//!
//! Both types are designed for thread-safe operation, with `Subscription` using
//! atomic operations to track its activation state.
//!

use std::sync::atomic::{AtomicBool, Ordering};
use std::any::TypeId;
use crate::core::HandlerId;

/// Represents a subscription to a specific event type
///
/// A `Subscription` tracks the connection between an event handler and
/// the event bus. When dropped or deactivated, the handler is automatically
/// unsubscribed, preventing memory leaks and ensuring proper cleanup.
///
/// # Thread Safety
///
/// `Subscription` uses atomic operations to ensure thread-safe activation state,
/// making it safe to share between threads when wrapped in appropriate
/// synchronization primitives.
///
/// # Lifecycle
///
/// Subscriptions are automatically activated when created and can be manually
/// deactivated when no longer needed. The deactivation is also automatic when
/// the subscription is dropped.
#[derive(Debug)]
pub struct Subscription {
    /// The type ID of the event this subscription is for
    pub type_id: TypeId,
    
    /// The unique ID of the handler
    pub handler_id: HandlerId,
    
    /// Whether this subscription is active
    active: AtomicBool,
}

impl Subscription {
    /// Create a new active subscription
    ///
    /// This constructs a subscription that links a specific event type with a handler ID.
    /// The subscription starts in the active state.
    ///
    /// # Arguments
    ///
    /// * `type_id` - The TypeId of the event this subscription is for
    /// * `handler_id` - The unique ID of the handler
    ///
    pub fn new(type_id: TypeId, handler_id: HandlerId) -> Self {
        Self {
            type_id,
            handler_id,
            active: AtomicBool::new(true),
        }
    }

    /// Check if the subscription is currently active
    ///
    /// Active subscriptions will receive events when published.
    /// This method is thread-safe and can be called from any context.
    ///
    /// # Returns
    ///
    /// `true` if the subscription is active, `false` otherwise
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }
    
    /// Deactivate this subscription
    ///
    /// Once deactivated, a subscription cannot be reactivated.
    /// This method is thread-safe and can be called from any context.
    ///
    /// # Thread Safety
    ///
    /// Uses atomic operations to ensure consistent state across threads.
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }
}

/// Collection for managing multiple subscriptions
///
/// `SubscriptionSet` provides RAII-style management of event subscriptions.
/// When the set is dropped or cleared, all contained subscriptions are
/// automatically deactivated, ensuring proper resource cleanup.
///
/// This is the primary subscription management mechanism used by components
/// implementing the `EventSubscriber` trait.
///
/// # Performance
///
/// - Adding subscriptions: O(1) amortized
/// - Clearing subscriptions: O(n) where n is the number of subscriptions
/// - Memory usage: Proportional to the number of active subscriptions
/// ```
#[derive(Default, Debug)]
pub struct SubscriptionSet {
    subscriptions: Vec<Subscription>,
}

impl SubscriptionSet {
    /// Create a new empty subscription set
    ///
    /// # Returns
    ///
    /// An empty `SubscriptionSet` with default capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_events::SubscriptionSet;
    ///
    /// let subscriptions = SubscriptionSet::new();
    /// assert!(subscriptions.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }
    
    /// Create a new subscription set with the specified capacity
    ///
    /// This is an optimization for when the approximate number of
    /// subscriptions is known in advance, reducing reallocations.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of subscriptions the set should be able to hold without reallocating
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_events::SubscriptionSet;
    ///
    /// // For a component that will have around 5 subscriptions
    /// let subscriptions = SubscriptionSet::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            subscriptions: Vec::with_capacity(capacity),
        }
    }
    
    /// Reserve space for at least `additional` more subscriptions
    ///
    /// This is an optimization to reduce reallocations when adding
    /// multiple subscriptions.
    ///
    /// # Arguments
    ///
    /// * `additional` - The number of additional subscriptions to reserve space for
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_events::SubscriptionSet;
    ///
    /// let mut subscriptions = SubscriptionSet::new();
    /// subscriptions.reserve(10); // Reserve space for 10 subscriptions
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.subscriptions.reserve(additional);
    }
    
    /// Add a subscription to the set
    ///
    /// The added subscription will be automatically deactivated when the set is
    /// cleared or dropped.
    ///
    /// # Arguments
    ///
    /// * `subscription` - The subscription to add
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_events::{SubscriptionSet, SharedEventBus};
    ///
    /// let event_bus = SharedEventBus::new();
    /// let mut subscriptions = SubscriptionSet::new();
    ///
    /// let subscription = event_bus.subscribe(|event: &MyEvent| {
    ///     println!("Event received: {:?}", event);
    /// });
    /// subscriptions.add(subscription);
    /// ```
    pub fn add(&mut self, subscription: Subscription) {
        self.subscriptions.push(subscription);
    }
    
    /// Clear all subscriptions, deactivating them
    ///
    /// This method deactivates all subscriptions and removes them from the set.
    /// It's automatically called when the set is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexus_events::SubscriptionSet;
    ///
    /// let mut subscriptions = SubscriptionSet::new();
    /// // Add subscriptions...
    ///
    /// subscriptions.clear(); // All subscriptions are deactivated
    /// assert!(subscriptions.is_empty());
    /// ```
    pub fn clear(&mut self) {
        // Deactivate all subscriptions first
        for sub in &self.subscriptions {
            sub.deactivate();
        }
        self.subscriptions.clear();
    }
    
    /// Returns the number of subscriptions in this set
    ///
    /// # Returns
    ///
    /// The count of subscriptions currently in the set
    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }
    
    /// Returns true if the set contains no subscriptions
    ///
    /// # Returns
    ///
    /// `true` if the set has no subscriptions, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }
}

impl Drop for SubscriptionSet {
    fn drop(&mut self) {
        self.clear();
    }
}