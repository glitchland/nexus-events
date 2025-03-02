#[cfg(test)]
mod thread_safety_tests {
    use std::sync::Arc;
    use std::thread;
    use crate::shared::SharedEventBus;

    #[derive(Debug, Clone)]
    struct ThreadTestEvent { counter: usize }

    #[test]
    fn test_multithreaded_publish_subscribe() {
        let event_bus = SharedEventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
        // Subscribe from the main thread
        let counter_clone = counter.clone();
        event_bus.subscribe(move |event: &ThreadTestEvent| {
            counter_clone.fetch_add(event.counter, Ordering::SeqCst);
        });
        
        // Publish from multiple threads
        let mut handles = vec![];
        for i in 0..10 {
            let event_bus = event_bus.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    event_bus.publish(ThreadTestEvent { counter: i * j }).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify the counter
        assert!(counter.load(Ordering::SeqCst) > 0);
    }
}