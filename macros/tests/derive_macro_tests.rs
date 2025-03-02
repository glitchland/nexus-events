use nexus_events::prelude::*;

// Define test events
define_event!(TestData { value: String });

// A basic component with derive
#[derive(Debug, EventSubscriber)]
struct BasicComponent {
    id: String,
    active: bool,
    subscriptions: SubscriptionSet,
}

impl BasicComponent {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            active: false,
            subscriptions: SubscriptionSet::new(),
        }
    }
    
    #[event_handler(TestData)]
    fn handle_data(&mut self, event: &TestData) {
        // No-op for testing
    }
}

// Component with multiple event handlers
#[derive(Debug, EventSubscriber)]
struct MultiHandlerComponent {
    id: String,
    active: bool,
    subscriptions: SubscriptionSet,
    received_data: Vec<String>,
}

impl MultiHandlerComponent {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            active: false,
            subscriptions: SubscriptionSet::new(),
            received_data: Vec::new(),
        }
    }
    
    #[event_handler(TestData)]
    fn handle_data1(&mut self, event: &TestData) {
        self.received_data.push(format!("handler1: {}", event.value));
    }
    
    #[event_handler(TestData)]
    fn handle_data2(&mut self, event: &TestData) {
        self.received_data.push(format!("handler2: {}", event.value));
    }
}

#[test]
fn test_derive_implements_event_subscriber() {
    // Check that our derived implementation gives us all the methods we need
    let mut component = BasicComponent::new("test1");
    
    // ID accessor
    assert_eq!(component.id(), "test1");
    
    // Active flag
    assert!(!component.is_active());
    component.active = true;
    assert!(component.is_active());
    
    // Subscription management
    let event_bus = SharedEventBus::new();
    component.register_event_handlers(&event_bus);
    
    // Should have one handler registered
    assert_eq!(component.subscriptions().len(), 1);
    
    // Cleanup
    component.clear_subscriptions();
    assert_eq!(component.subscriptions().len(), 0);
}

#[test]
fn test_multiple_handlers_registration() {
    let mut component = MultiHandlerComponent::new("multi");
    component.active = true;
    
    let event_bus = SharedEventBus::new();
    component.register_event_handlers(&event_bus);
    
    // Should have two handlers registered
    assert_eq!(component.subscriptions().len(), 2);
    
    // Test that both handlers are called
    event_bus.publish(TestData { value: "test-event".to_string() }).expect("Failed to publish");
    
    assert_eq!(component.received_data.len(), 2);
    assert!(component.received_data.contains(&"handler1: test-event".to_string()));
    assert!(component.received_data.contains(&"handler2: test-event".to_string()));
}