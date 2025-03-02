use nexus_events::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// Define test event types
define_event!(TestMoved { id: String, x: f32, y: f32 });
define_event!(TestDamaged { id: String, amount: u32 });

// Define a component with event handlers
#[derive(Debug, EventSubscriber)]
struct TestComponent {
    id: String,
    active: bool,
    subscriptions: SubscriptionSet,
    position: (f32, f32),
    health: u32,
    moved_count: u32,
    damaged_count: u32,
    last_position: Option<(f32, f32)>,
    last_damage: Option<u32>,
}

impl TestComponent {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            active: false,
            subscriptions: SubscriptionSet::new(),
            position: (0.0, 0.0),
            health: 100,
            moved_count: 0,
            damaged_count: 0,
            last_position: None,
            last_damage: None,
        }
    }
    
    #[event_handler(TestMoved)]
    fn handle_move(&mut self, event: &TestMoved) {
        if event.id == self.id {
            self.position = (event.x, event.y);
            self.last_position = Some((event.x, event.y));
            self.moved_count += 1;
        }
    }
    
    #[event_handler(TestDamaged)]
    fn handle_damage(&mut self, event: &TestDamaged) {
        if event.id == self.id {
            self.health = self.health.saturating_sub(event.amount);
            self.last_damage = Some(event.amount);
            self.damaged_count += 1;
        }
    }
}

#[test]
fn test_event_handler_registration() {
    let mut component = TestComponent::new("test1");
    component.active = true;
    
    let event_bus = SharedEventBus::new();
    component.register_event_handlers(&event_bus);
    
    // Should have registered two handlers
    assert_eq!(component.subscriptions().len(), 2);
}

#[test]
fn test_event_handler_receiving() {
    let mut component = TestComponent::new("test1");
    component.active = true;
    
    let event_bus = SharedEventBus::new();
    component.register_event_handlers(&event_bus);
    
    // Publish events
    event_bus.publish(TestMoved { 
        id: "test1".to_string(),
        x: 10.5, 
        y: 20.8 
    }).expect("Failed to publish event");
    
    event_bus.publish(TestDamaged { 
        id: "test1".to_string(), 
        amount: 30 
    }).expect("Failed to publish event");
    
    // Verify the handlers were called
    assert_eq!(component.moved_count, 1);
    assert_eq!(component.damaged_count, 1);
    assert_eq!(component.position, (10.5, 20.8));
    assert_eq!(component.health, 70);
    assert_eq!(component.last_position, Some((10.5, 20.8)));
    assert_eq!(component.last_damage, Some(30));
}

#[test]
fn test_multiple_components() {
    let mut component1 = TestComponent::new("test1");
    let mut component2 = TestComponent::new("test2");
    
    component1.active = true;
    component2.active = true;
    
    let event_bus = SharedEventBus::new();
    component1.register_event_handlers(&event_bus);
    component2.register_event_handlers(&event_bus);
    
    // Publish events for component1
    event_bus.publish(TestMoved { 
        id: "test1".to_string(),
        x: 10.0, 
        y: 20.0 
    }).expect("Failed to publish event");
    
    // Publish events for component2
    event_bus.publish(TestMoved { 
        id: "test2".to_string(),
        x: 30.0, 
        y: 40.0 
    }).expect("Failed to publish event");
    
    // Verify each component only handled their own events
    assert_eq!(component1.position, (10.0, 20.0));
    assert_eq!(component2.position, (30.0, 40.0));
}

#[test]
fn test_event_handler_deactivation() {
    let mut component = TestComponent::new("test1");
    component.active = true;
    
    let event_bus = SharedEventBus::new();
    component.register_event_handlers(&event_bus);
    
    // Deactivate the component
    component.clear_subscriptions();
    
    // Publish events
    event_bus.publish(TestMoved { 
        id: "test1".to_string(),
        x: 10.0, 
        y: 20.0 
    }).expect("Failed to publish event");
    
    // Verify the handlers were not called
    assert_eq!(component.moved_count, 0);
    assert_eq!(component.position, (0.0, 0.0));
}