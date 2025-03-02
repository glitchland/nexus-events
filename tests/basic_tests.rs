// File: tests/basic_tests.rs
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use nexus_events::EventBus;
    use nexus_events::core::error::{EventError, EventResult};

    // Define a dummy event type for testing.
    #[derive(Debug, Clone)]
    struct TestEvent {
        pub value: i32,
    }

    #[test]
    fn test_unsubscribe() {
        let mut event_bus = EventBus::new();
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        
        // Subscribe with an immutable handler.
        let handler_id = event_bus.subscribe(move |_: &TestEvent| {
            let mut cnt = counter_clone.lock().unwrap();
            *cnt += 1;
        });
        
        // Publish an event; the counter should increment.
        event_bus
            .publish(&TestEvent { value: 42 })
            .expect("Failed to publish event");
        assert_eq!(*counter.lock().unwrap(), 1);
        
        // Unsubscribe the handler.
        let result: EventResult<()> = event_bus.unsubscribe::<TestEvent>(handler_id);
        assert!(result.is_ok());
        
        // Publish again; the counter should not change.
        event_bus
            .publish(&TestEvent { value: 43 })
            .expect("Failed to publish event");
        assert_eq!(*counter.lock().unwrap(), 1);
    }
    
    #[test]
    fn test_unsubscribe_nonexistent() {
        let mut event_bus = EventBus::new();
        // Try to unsubscribe a handler that doesn't exist.
        let result = event_bus.unsubscribe::<TestEvent>(nexus_events::HandlerId(999));
        assert!(result.is_err());
        match result {
            Err(EventError::HandlerNotFound { id }) => assert_eq!(id, 999),
            Err(EventError::EventTypeNotFound) => { /* Acceptable for an empty event type */ },
            _ => panic!("Expected HandlerNotFound or EventTypeNotFound error"),
        }
    }
}