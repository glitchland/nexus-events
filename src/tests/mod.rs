//! Tests for the event system

mod macro_tests;

#[cfg(test)]
mod integration_tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::{EventBus, subscribe, subscribe_with_args, unsubscribe, event_handler};

    // Test event types
    #[derive(Debug, Clone)]
    struct TestEvent {
        pub value: String,
    }

    #[test]
    fn test_basic_subscribe_publish() {
        let mut event_bus = EventBus::new();
        let result = Rc::new(RefCell::new(None));
        
        let result_clone = result.clone();
        subscribe!(event_bus, |event: &TestEvent| {
            *result_clone.borrow_mut() = Some(event.value.clone());
        });
        
        event_bus.publish(TestEvent { value: "test_value".to_string() });
        
        assert_eq!(*result.borrow(), Some("test_value".to_string()));
    }

    #[test]
    fn test_subscribe_with_args() {
        let mut event_bus = EventBus::new();
        let result = Rc::new(RefCell::new(None));
        
        let multiplier = 5;
        let result_clone = result.clone();
        
        subscribe_with_args!(event_bus, |event: &TestEvent, multiplier| {
            let count = event.value.len() * multiplier;
            *result_clone.borrow_mut() = Some(count);
        }, multiplier);
        
        event_bus.publish(TestEvent { value: "test".to_string() });
        
        assert_eq!(*result.borrow(), Some(20)); // "test" has 4 chars × 5
    }

    #[test]
    fn test_multiple_args() {
        let mut event_bus = EventBus::new();
        let result = Rc::new(RefCell::new(None));
        
        let prefix = "Player: ";
        let suffix = " points";
        let result_clone = result.clone();
        
        subscribe_with_args!(event_bus, |event: &TestEvent, prefix, suffix| {
            let formatted = format!("{}{}{}", prefix, event.value, suffix);
            *result_clone.borrow_mut() = Some(formatted);
        }, prefix, suffix);
        
        event_bus.publish(TestEvent { value: "100".to_string() });
        
        assert_eq!(*result.borrow(), Some("Player: 100 points".to_string()));
    }

    #[test]
    fn test_handler_unsubscribe() {
        let mut event_bus = EventBus::new();
        let counter = Rc::new(RefCell::new(0));
        
        let counter_clone = counter.clone();
        let id = subscribe!(event_bus, |_: &TestEvent| {
            *counter_clone.borrow_mut() += 1;
        });
        
        // First publish
        event_bus.publish(TestEvent { value: "first".to_string() });
        assert_eq!(*counter.borrow(), 1);
        
        // Unsubscribe
        unsubscribe!(event_bus, TestEvent, id);
        
        // Second publish should not increment counter
        event_bus.publish(TestEvent { value: "second".to_string() });
        assert_eq!(*counter.borrow(), 1);
    }
}
