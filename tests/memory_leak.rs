// Language: Rust
// File: tests/memory_leak.rs
#[cfg(test)]
mod memory_leak_tests {
    use std::sync::{Arc, Mutex};
    use nexus_events::EventBus;
    use nexus_events::core::error::EventResult;
    use nexus_events::core::event::Event;
    use nexus_events::HandlerId; // For wrapping handler IDs when unsubscribing.

    // Dummy event type for this test.
    #[derive(Debug, Clone)]
    struct TestEvent {
        pub value: i32,
    }

    #[test]
    fn memory_leak_test() {
        // Create a new event bus.
        let mut event_bus = EventBus::new();
        // Use an Arc<Mutex<>> counter to detect if any unsubscribed handler is still called.
        let counter = Arc::new(Mutex::new(0));
        let iterations = 10_000;

        // 1. Create many subscriptions and then immediately unsubscribe them.
        for _ in 0..iterations {
            let counter_clone = Arc::clone(&counter);
            let handler_id = event_bus.subscribe(move |_: &TestEvent| {
                let mut cnt = counter_clone.lock().unwrap();
                *cnt += 1;
            });
            // Immediately unsubscribe the handler.
            let _ : EventResult<()> = event_bus.unsubscribe::<TestEvent>(handler_id);
        }
        // Publish an event; no handlers should be active.
        event_bus
            .publish(&TestEvent { value: 100 })
            .expect("Failed to publish event");
        // Ensure the counter was not incremented.
        assert_eq!(*counter.lock().unwrap(), 0);

        // 2. Create a batch of subscriptions that remain active.
        let mut handler_ids = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let counter_clone = Arc::clone(&counter);
            let id = event_bus.subscribe(move |_: &TestEvent| {
                let mut cnt = counter_clone.lock().unwrap();
                *cnt += 1;
            });
            handler_ids.push(id);
        }
        // Publish an event; all active subscriptions should fire.
        event_bus
            .publish(&TestEvent { value: 101 })
            .expect("Failed to publish event");
        // The counter should equal the number of active subscriptions.
        assert_eq!(*counter.lock().unwrap(), iterations);

        // 3. Unsubscribe all active handlers.
        for id in handler_ids {
            let _ : EventResult<()> = event_bus.unsubscribe::<TestEvent>(id);
        }
        // Publish another event; no handlers should now be active,
        // so the counter remains unchanged.
        event_bus
            .publish(&TestEvent { value: 102 })
            .expect("Failed to publish event");
        assert_eq!(*counter.lock().unwrap(), iterations);
    }
}