// Language: Rust
// File: tests/thread_safety.rs
#[cfg(test)]
mod thread_safety_tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::thread;
    use std::time::Duration;
    use nexus_events::core::shared::SharedEventBus;

    // A dummy event type for thread safety tests.
    #[derive(Debug, Clone)]
    struct TestEvent {
        pub value: i32,
    }

    /// Test that many threads can concurrently publish events without data races.
    #[test]
    fn test_thread_safe_publishing() {
        let shared = SharedEventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        // Subscribe a handler that increments the counter with each event.
        shared.subscribe(move |event: &TestEvent| {
            counter_clone.fetch_add(event.value as usize, Ordering::SeqCst);
        });
        
        // Spawn multiple threads that each publish events.
        let threads: Vec<_> = (0..10)
            .map(|_| {
                let shared_clone = shared.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        shared_clone
                            .publish(&TestEvent { value: 1 })
                            .expect("Publish failed");
                        thread::sleep(Duration::from_millis(2));
                    }
                })
            })
            .collect();
            
        for handle in threads {
            handle.join().unwrap();
        }
        
        // The counter should equal 10 threads * 100 events = 1000.
        assert_eq!(counter.load(Ordering::SeqCst), 1000);
    }

    /// Test that subscriptions can be created concurrently and all handlers are invoked.
    #[test]
    fn test_thread_safe_subscription() {
        let shared = SharedEventBus::new();
        let handler_count = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        
        // From multiple threads, subscribe handlers concurrently.
        for _ in 0..10 {
            let shared_clone = shared.clone();
            let handler_count_clone = Arc::clone(&handler_count);
            let handle = thread::spawn(move || {
                shared_clone.subscribe(move |_: &TestEvent| {
                    handler_count_clone.fetch_add(1, Ordering::SeqCst);
                });
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Publish an event to trigger all subscriptions.
        shared.publish(&TestEvent { value: 1 }).expect("Publish failed");
        // Each of the 10 subscriptions should be invoked exactly once.
        assert_eq!(handler_count.load(Ordering::SeqCst), 10);
    }

    /// Test that unsubscriptions work correctly even when called concurrently.
    #[test]
    fn test_thread_safe_unsubscription() {
        let shared = SharedEventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
        // Create a subscription whose handler increments the counter.
        let handler_id = shared.subscribe({
            let counter_clone = Arc::clone(&counter);
            move |event: &TestEvent| {
                counter_clone.fetch_add(event.value as usize, Ordering::SeqCst);
            }
        });
        
        // Spawn several threads that attempt to unsubscribe the same handler concurrently.
        let mut threads = vec![];
        for _ in 0..5 {
            let shared_clone = shared.clone();
            let id = handler_id; // same handler id
            let handle = thread::spawn(move || {
                let _ = shared_clone.unsubscribe::<TestEvent>(id);
            });
            threads.push(handle);
        }
        for handle in threads {
            handle.join().unwrap();
        }
        
        // Publish an event; no handlers should be called since the handler is unsubscribed.
        shared.publish(&TestEvent { value: 1 }).expect("Publish failed");
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }
}