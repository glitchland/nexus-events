#[cfg(test)]
mod tests{
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use nexus_events::core::event::Event;
use nexus_events::core::shared::SharedEventBus;
use nexus_events::core::error::EventResult;

/// A dummy event for testing.
#[derive(Debug, Clone)]
struct TestEvent {
    pub value: String,
}

/// A dummy event for mutable testing.
#[derive(Debug, Clone)]
struct MutableTestEvent {
    pub num: usize,
}

#[test]
fn test_basic_subscribe_publish() {
    let result = Arc::new(Mutex::new(None));
    let result_clone = Arc::clone(&result);

    let shared = SharedEventBus::new();
    shared.subscribe(move |event: &TestEvent| {
        let mut res = result_clone.lock().unwrap();
        *res = Some(event.value.clone());
    });

    // Publish directly via SharedEventBus
    shared.publish(TestEvent { value: "hello".to_string() })
        .expect("Publish failed");

    thread::sleep(Duration::from_millis(10));
    assert_eq!(*result.lock().unwrap(), Some("hello".to_string()));
}

#[test]
fn test_subscribe_mut_publish() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    let shared = SharedEventBus::new();
    shared.subscribe_mut(move |event: &MutableTestEvent| {
        let mut count = counter_clone.lock().unwrap();
        *count += event.num;
    });

    shared.publish(MutableTestEvent { num: 5 })
        .expect("Publish failed");

    thread::sleep(Duration::from_millis(10));
    assert_eq!(*counter.lock().unwrap(), 5);
}

#[test]
fn test_multithreaded_publish_subscribe() {
    let shared = SharedEventBus::new();
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    shared.subscribe(move |event: &MutableTestEvent| {
        let mut count = counter_clone.lock().unwrap();
        *count += event.num;
    });

    let mut handles = vec![];
    for i in 0..10 {
        let shared_clone = shared.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                shared_clone.publish(MutableTestEvent { num: i })
                    .expect("Publish failed");
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    assert!(*counter.lock().unwrap() > 0);
}
}