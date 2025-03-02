#[cfg(test)]
mod memory_leak_tests {
    use std::sync::Arc;
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::core::EventBus;
    use crate::subscription::SubscriptionSet;

    struct DropCounter {
        count: Rc<RefCell<usize>>,
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            *self.count.borrow_mut() += 1;
        }
    }

    #[test]
    fn test_subscription_cleanup() {
        let drop_count = Rc::new(RefCell::new(0));
        let event_bus = EventBus::new();
        
        // Create a scope to test automatic cleanup
        {
            let mut subscriptions = SubscriptionSet::new();
            let counter = DropCounter { count: drop_count.clone() };
            
            // Create a subscription that captures the counter
            let sub = event_bus.subscribe(move |_: &()| {
                let _ = &counter; // Capture the counter
            });
            
            subscriptions.add(sub);
            
            // Subscriptions still alive here
            assert_eq!(*drop_count.borrow(), 0);
        }
        
        // After subscriptions are dropped, counter should be dropped too
        assert_eq!(*drop_count.borrow(), 1);
    }
}