use std::cell::RefCell;
use std::rc::Rc;
use crate::EventBus;
use crate::{subscribe, unsubscribe, event_handler};

// Test event types
#[derive(Debug, Clone)]
struct ButtonClickEvent {
    pub button_id: String,
}

#[derive(Debug, Clone)]
struct MouseMoveEvent {
    pub x: f32,
    pub y: f32,
}

#[test]
fn test_subscribe_macro() {
    let mut event_bus = EventBus::new();
    let result = Rc::new(RefCell::new(None));
    
    let result_clone = result.clone();
    subscribe!(event_bus, |event: &ButtonClickEvent| {
        *result_clone.borrow_mut() = Some(event.button_id.clone());
    });
    
    event_bus.publish(ButtonClickEvent { button_id: "start_button".to_string() });
    
    assert_eq!(*result.borrow(), Some("start_button".to_string()));
}

#[test]
fn test_unsubscribe_macro() {
    let mut event_bus = EventBus::new();
    let counter = Rc::new(RefCell::new(0));
    
    let counter_clone = counter.clone();
    let handler_id = subscribe!(event_bus, |_: &ButtonClickEvent| {
        *counter_clone.borrow_mut() += 1;
    });
    
    // First publish
    event_bus.publish(ButtonClickEvent { button_id: "btn1".to_string() });
    assert_eq!(*counter.borrow(), 1);
    
    // Unsubscribe
    let result = unsubscribe!(event_bus, ButtonClickEvent, handler_id);
    assert!(result);
    
    // Second publish should not increment counter
    event_bus.publish(ButtonClickEvent { button_id: "btn2".to_string() });
    assert_eq!(*counter.borrow(), 1);
}

#[test]
fn test_event_handler_macro() {
    let mut event_bus = EventBus::new();
    let counter = Rc::new(RefCell::new(0));
    
    // Create handler using the macro
    let counter_clone = counter.clone();
    let handler = event_handler!(|event: &ButtonClickEvent| {
        *counter_clone.borrow_mut() += 1;
    });
    
    // Subscribe using the created handler
    event_bus.subscribe(move |event: &ButtonClickEvent| {
        handler(event);
    });
    
    // Publish an event
    event_bus.publish(ButtonClickEvent { button_id: "test_button".to_string() });
    
    assert_eq!(*counter.borrow(), 1);
}

#[test]
fn test_multiple_event_types_with_macros() {
    let mut event_bus = EventBus::new();
    let button_clicks = Rc::new(RefCell::new(0));
    let mouse_moves = Rc::new(RefCell::new(0));
    
    // Subscribe to different event types
    let button_clone = button_clicks.clone();
    subscribe!(event_bus, |_: &ButtonClickEvent| {
        *button_clone.borrow_mut() += 1;
    });
    
    let mouse_clone = mouse_moves.clone();
    subscribe!(event_bus, |_: &MouseMoveEvent| {
        *mouse_clone.borrow_mut() += 1;
    });
    
    // Publish different events
    event_bus.publish(ButtonClickEvent { button_id: "btn".to_string() });
    event_bus.publish(MouseMoveEvent { x: 10.0, y: 20.0 });
    event_bus.publish(ButtonClickEvent { button_id: "btn2".to_string() });
    
    assert_eq!(*button_clicks.borrow(), 2);
    assert_eq!(*mouse_moves.borrow(), 1);
}
