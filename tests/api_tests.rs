// Language: Rust
// File: tests/api_tests.rs

use nexus_events::prelude::*;
use nexus_events::core::shared::SharedEventBus;

// Use the define_event! macro to create test event types.
define_event!(TestMoved { id: String, x: f32, y: f32 });
define_event!(TestDamaged { id: String, amount: u32 });

// Define a component with event handlers using the derive macro.
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
    
    // Should have registered two handlers.
    assert_eq!(component.subscriptions().len(), 2);
}

#[test]
fn test_event_handler_receiving() {
    let mut component = TestComponent::new("test1");
    component.active = true;
    
    let event_bus = SharedEventBus::new();
    component.register_event_handlers(&event_bus);
    
    // Create and publish a TestMoved event.
    let moved = TestMoved { id: "test1".to_string(), x: 10.5, y: 20.8 };
    event_bus.publish(moved).expect("Failed to publish TestMoved event");
    
    // Create and publish a TestDamaged event.
    let damaged = TestDamaged { id: "test1".to_string(), amount: 30 };
    event_bus.publish(damaged).expect("Failed to publish TestDamaged event");
    
    // Verify that the handlers were properly called.
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
    
    // Create and publish a TestMoved event for component1.
    let moved1 = TestMoved { id: "test1".to_string(), x: 10.0, y: 20.0 };
    event_bus.publish(moved1).expect("Failed to publish TestMoved event for component1");
    
    // Create and publish a TestMoved event for component2.
    let moved2 = TestMoved { id: "test2".to_string(), x: 30.0, y: 40.0 };
    event_bus.publish(moved2).expect("Failed to publish TestMoved event for component2");
    
    // Verify that each component only handles its own events.
    assert_eq!(component1.position, (10.0, 20.0));
    assert_eq!(component2.position, (30.0, 40.0));
}

#[test]
fn test_event_handler_deactivation() {
    let mut component = TestComponent::new("test1");
    component.active = true;
    
    let event_bus = SharedEventBus::new();
    component.register_event_handlers(&event_bus);
    
    // Deactivate the component so that its subscriptions are cleared.
    component.clear_subscriptions();
    
    // Create and publish a TestMoved event; the handler should not be triggered.
    let moved = TestMoved { id: "test1".to_string(), x: 10.0, y: 20.0 };
    event_bus.publish(moved).expect("Failed to publish TestMoved event after deactivation");
    
    // Verify that no changes occurred because the subscriptions were cleared.
    assert_eq!(component.moved_count, 0);
    assert_eq!(component.position, (0.0, 0.0));
}