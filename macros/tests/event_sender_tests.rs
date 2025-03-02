use nexus_events::prelude::*;
use std::sync::{Arc, Mutex};
use std::any::{Any, TypeId};

// Define test events
define_event!(PlayerMoved { player_id: String, x: f32, y: f32 });
define_event!(PlayerDamaged { player_id: String, amount: u32 });
define_event!(SimpleEvent);  // Test empty event

// Test component with event senders
#[derive(Debug)]
struct TestSender {
    sender: EventSender,
}

impl EventEmitter for TestSender {
    fn sender(&self) -> &EventSender {
        &self.sender
    }
}

impl TestSender {
    fn new(event_bus: &SharedEventBus) -> Self {
        Self {
            sender: EventSender::new(event_bus.clone()),
        }
    }
    
    #[event_sender(PlayerMoved)]
    fn move_player(&self, player_id: String, x: f32, y: f32) {
        // Just some preprocessing logic
        println!("Moving player {} to ({}, {})", player_id, x, y);
    }
    
    #[event_sender(PlayerDamaged)]
    fn damage_player(&self, player_id: String, amount: u32) {
        println!("Player {} takes {} damage", player_id, amount);
    }
    
    // Test returning a value
    #[event_sender(PlayerMoved)]
    fn move_player_with_return(&self, player_id: String, x: f32, y: f32) -> u32 {
        // Return some arbitrary value
        42
    }
    
    // Test empty event
    #[event_sender(SimpleEvent)]
    fn trigger_simple_event(&self) {
        println!("Simple event triggered");
    }
}

#[test]
fn test_event_sender_emits_event() {
    let event_bus = SharedEventBus::new();
    
    // Create our test component
    let sender = TestSender::new(&event_bus);
    
    // Track received events
    let moved_events = Arc::new(Mutex::new(Vec::new()));
    let moved_events_clone = moved_events.clone();
    
    // Subscribe to events
    let moved_events_clone = moved_events.clone();
    let subscription1 = event_bus.subscribe(
        TypeId::of::<PlayerMoved>(),
        Box::new(move |event: &dyn Any| {
            if let Some(moved) = event.downcast_ref::<PlayerMoved>() {
                moved_events_clone.borrow_mut().push((
                    moved.player_id.clone(),
                    moved.x,
                    moved.y
                ));
            }
        })
    );
    
    let damaged_events_clone = damaged_events.clone();
    let subscription2 = event_bus.subscribe(
        TypeId::of::<PlayerDamaged>(),
        Box::new(move |event: &dyn Any| {
            if let Some(damaged) = event.downcast_ref::<PlayerDamaged>() {
                damaged_events_clone.borrow_mut().push((
                    damaged.player_id.clone(),
                    damaged.amount
                ));
            }
        })
    );
    
    // Call our event sender methods
    sender.move_player("player1".to_string(), 10.5, 20.8).expect("Failed to send event");
    sender.damage_player("player1".to_string(), 15).expect("Failed to send event");
    
    // Verify events were received
    assert_eq!(moved_events.borrow().len(), 1);
    assert_eq!(damaged_events.borrow().len(), 1);
    
    let moved = &moved_events.borrow()[0];
    assert_eq!(moved.0, "player1");
    assert_eq!(moved.1, 10.5);
    assert_eq!(moved.2, 20.8);
    
    let damaged = &damaged_events.borrow()[0];
    assert_eq!(damaged.0, "player1");
    assert_eq!(damaged.1, 15);
}

#[test]
fn test_event_sender_with_return() {
    let event_bus = SharedEventBus::new();
    let sender = TestSender::new(&event_bus);
    
    let moved_events = Arc::new(Mutex::new(Vec::new()));
    let moved_events_clone = moved_events.clone();
    let _subscription = event_bus.subscribe(
        TypeId::of::<PlayerMoved>(),
        Box::new(move |event: &dyn Any| {
            if let Some(moved) = event.downcast_ref::<PlayerMoved>() {
                moved_events_clone.borrow_mut().push(moved.player_id.clone());
            }
        })
    );
    
    // Call the method and capture return value
    let result = sender.move_player_with_return(
        "player2".to_string(), 5.0, 10.0
    ).expect("Failed to send event");
    
    // Verify the return value was preserved
    assert_eq!(result, 42);
    
    // Verify event was sent
    assert_eq!(moved_events.borrow().len(), 1);
    assert_eq!(moved_events.borrow()[0], "player2");
}

#[test]
fn test_empty_event() {
    let event_bus = SharedEventBus::new();
    let sender = TestSender::new(&event_bus);
    
    // Track received events
    let simple_events = Rc::new(RefCell::new(0));
    
    // Subscribe to events
    let simple_events_clone = simple_events.clone();
    let _subscription = event_bus.subscribe(
        TypeId::of::<SimpleEvent>(),
        Box::new(move |event: &dyn Any| {
            if let Some(_) = event.downcast_ref::<SimpleEvent>() {
                *simple_events_clone.borrow_mut() += 1;
            }
        })
    );
    
    // Call the method
    sender.trigger_simple_event().expect("Failed to send event");
    
    // Verify event was sent
    assert_eq!(*simple_events.borrow(), 1);
}