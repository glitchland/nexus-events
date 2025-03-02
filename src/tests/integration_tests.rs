use nexus_events::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// Define events
define_event!(EntityMoved { entity_id: String, x: f32, y: f32 });
define_event!(EntityDamaged { entity_id: String, amount: u32 });

// Component that both receives and sends events
#[derive(Debug, EventSubscriber)]
struct Entity {
    id: String,
    active: bool,
    subscriptions: SubscriptionSet,
    position: (f32, f32),
    health: u32,
    sender: EventSender,
}

impl EventEmitter for Entity {
    fn sender(&self) -> &EventSender {
        &self.sender
    }
}

impl Entity {
    fn new(id: &str, event_bus: &SharedEventBus) -> Self {
        Self {
            id: id.to_string(),
            active: false,
            subscriptions: SubscriptionSet::new(),
            position: (0.0, 0.0),
            health: 100,
            sender: EventSender::new(event_bus.clone()),
        }
    }
    
    fn activate(&mut self, event_bus: &SharedEventBus) {
        self.active = true;
        self.register_event_handlers(event_bus);
    }
    
    fn deactivate(&mut self) {
        self.active = false;
        self.clear_subscriptions();
    }
    
    #[event_handler(EntityMoved)]
    fn on_move(&mut self, event: &EntityMoved) {
        if event.entity_id == self.id {
            self.position = (event.x, event.y);
        }
    }
    
    #[event_handler(EntityDamaged)]
    fn on_damage(&mut self, event: &EntityDamaged) {
        if event.entity_id == self.id {
            self.health = self.health.saturating_sub(event.amount);
        }
    }
    
    #[event_sender(EntityMoved)]
    fn move_to(&self, entity_id: String, x: f32, y: f32) {
        // Pre-processing
        println!("Moving entity {} to ({}, {})", entity_id, x, y);
    }
    
    #[event_sender(EntityDamaged)]
    fn damage(&self, entity_id: String, amount: u32) {
        // Pre-processing
        println!("Damaging entity {} by {}", entity_id, amount);
    }
}

#[test]
fn test_complete_event_flow() {
    let event_bus = SharedEventBus::new();
    
    // Create entities
    let mut entity1 = Entity::new("entity1", &event_bus);
    let mut entity2 = Entity::new("entity2", &event_bus);
    
    // Activate entities
    entity1.activate(&event_bus);
    entity2.activate(&event_bus);
    
    // Entity 1 attacks entity 2
    entity1.damage("entity2".to_string(), 30).expect("Failed to send damage event");
    
    // Entity 2 moves
    entity2.move_to("entity2".to_string(), 10.0, 15.0).expect("Failed to send move event");
    
    // Verify the results
    assert_eq!(entity1.health, 100); // Entity 1 wasn't damaged
    assert_eq!(entity2.health, 70);  // Entity 2 was damaged by 30
    
    assert_eq!(entity1.position, (0.0, 0.0)); // Entity 1 didn't move
    assert_eq!(entity2.position, (10.0, 15.0)); // Entity 2 moved
    
    // Test event filtering
    entity1.move_to("entity1".to_string(), 5.0, 5.0).expect("Failed to send move event");
    entity1.move_to("entity2".to_string(), 20.0, 20.0).expect("Failed to send move event");
    
    // Verify only the proper events were handled
    assert_eq!(entity1.position, (5.0, 5.0));   // Entity 1 only handled its own event
    assert_eq!(entity2.position, (20.0, 20.0)); // Entity 2 only handled its own event
    
    // Test deactivation
    entity2.deactivate();
    entity1.damage("entity2".to_string(), 20).expect("Failed to send damage event");
    
    // Verify entity2 didn't handle the event
    assert_eq!(entity2.health, 70); // Health didn't change
}

#[test]
fn test_multiple_handlers_same_event() {
    let event_bus = SharedEventBus::new();
    
    // Define a component with multiple handlers for the same event type
    #[derive(Debug, EventSubscriber)]
    struct MultiHandler {
        id: String,
        active: bool,
        subscriptions: SubscriptionSet,
        move_count: u32,
        total_moved: f32,
    }
    
    impl MultiHandler {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                active: false,
                subscriptions: SubscriptionSet::new(),
                move_count: 0,
                total_moved: 0.0,
            }
        }
        
        #[event_handler(EntityMoved)]
        fn count_moves(&mut self, _event: &EntityMoved) {
            self.move_count += 1;
        }
        
        #[event_handler(EntityMoved)]
        fn track_distance(&mut self, event: &EntityMoved) {
            self.total_moved += event.x + event.y;
        }
    }
    
    // Create and activate the handler
    let mut handler = MultiHandler::new("tracker");
    handler.active = true;
    handler.register_event_handlers(&event_bus);
    
    // Send some events
    event_bus.publish(EntityMoved {
        entity_id: "test".to_string(),
        x: 10.0,
        y: 20.0,
    }).expect("Failed to publish");
    
    event_bus.publish(EntityMoved {
        entity_id: "test".to_string(),
        x: 5.0,
        y: 15.0,
    }).expect("Failed to publish");
    
    // Verify both handlers were called for each event
    assert_eq!(handler.move_count, 2);
    assert_eq!(handler.total_moved, 50.0); // 10+20+5+15
}