#[test]
fn test_unsubscribe() {
    let mut event_bus = EventBus::new();
    let counter = Rc::new(RefCell::new(0));
    
    // Add a subscriber and get its ID
    let counter_clone = counter.clone();
    let handler_id = event_bus.subscribe(move |_: &TestEvent| {
        *counter_clone.borrow_mut() += 1;
    });
    
    // Publish once
    event_bus.publish(TestEvent { value: 42 }).expect("Failed to publish event");
    assert_eq!(*counter.borrow(), 1);
    
    // Unsubscribe
    let result = event_bus.unsubscribe::<TestEvent>(handler_id);
    assert!(result.is_ok()); // Should return Ok for successful unsubscription
    
    // Publish again
    event_bus.publish(TestEvent { value: 43 }).expect("Failed to publish event");
    assert_eq!(*counter.borrow(), 1); // Counter shouldn't change
}

#[test]
fn test_unsubscribe_nonexistent() {
    let mut event_bus = EventBus::new();
    
    // Try to unsubscribe a handler that doesn't exist
    let result = event_bus.unsubscribe::<TestEvent>(999);
    assert!(result.is_err());
    if let Err(EventError::HandlerNotFound { id }) = result {
        assert_eq!(id, 999);
    } else {
        panic!("Expected HandlerNotFound error");
    }
}