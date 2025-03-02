//! Example demonstrating the attribute-based event system API
//! Shows both event handling and sending using attributes

use std::fmt::Debug;
use nexus_events::{
    define_event, EventSubscriber, EventEmitter, SharedEventBus, 
    SubscriptionSet, EventSender, EventResult, event_handler, event_sender
};

// Define events
define_event!(PlayerMoved { player_id: String, x: f32, y: f32 });
define_event!(PlayerDamaged { player_id: String, amount: u32 });
define_event!(ItemCollected { player_id: String, item_id: String });

// Define our player component
#[derive(Debug, EventSubscriber)]
struct Player {
    id: String,
    active: bool,
    subscriptions: SubscriptionSet,
    position: (f32, f32),
    health: u32,
    inventory: Vec<String>,
    sender: EventSender,
}

impl EventEmitter for Player {
    fn sender(&self) -> &EventSender {
        &self.sender
    }
}

impl Player {
    fn new(id: impl Into<String>, event_bus: &SharedEventBus) -> Self {
        Self {
            id: id.into(),
            active: false,
            subscriptions: SubscriptionSet::new(),
            position: (0.0, 0.0),
            health: 100,
            inventory: Vec::new(),
            sender: EventSender::new(event_bus.clone()),
        }
    }
    
    // Activate the player - subscribes to events
    fn activate(&mut self, event_bus: &SharedEventBus) {
        if self.active { return; }
        self.active = true;
        
        // This automatically registers ALL methods marked with #[event_handler]
        self.register_event_handlers(event_bus);
    }
    
    // Deactivate the player - unsubscribes from events
    fn deactivate(&mut self) {
        if !self.active { return; }
        self.active = false;
        self.clear_subscriptions();
    }
    
    // === EVENT HANDLERS ===
    // Just add the attribute to handle events!
    
    #[event_handler(PlayerMoved)]
    fn handle_movement(&mut self, event: &PlayerMoved) {
        // Only handle our own movement events
        if event.player_id == self.id {
            println!("Player {} moved to position ({}, {})", 
                     self.id, event.x, event.y);
            self.position = (event.x, event.y);
        }
    }
    
    #[event_handler(PlayerDamaged)]
    fn handle_damage(&mut self, event: &PlayerDamaged) {
        // Only handle our own damage events
        if event.player_id == self.id {
            if event.amount >= self.health {
                self.health = 0;
                println!("Player {} died!", self.id);
            } else {
                self.health -= event.amount;
                println!("Player {} took {} damage. Health: {}", 
                         self.id, event.amount, self.health);
            }
        }
    }
    
    #[event_handler(ItemCollected)]
    fn handle_item_collection(&mut self, event: &ItemCollected) {
        // Only handle our own item collection events
        if event.player_id == self.id {
            println!("Player {} collected item: {}", 
                     self.id, event.item_id);
            self.inventory.push(event.item_id.clone());
        }
    }
    
    // === EVENT SENDERS ===
    // Just add the attribute to send events!
    
    #[event_sender(PlayerMoved)]
    fn move_to(&self, player_id: String, x: f32, y: f32) {
        // Logic before sending the event
        println!("Moving player {} to ({}, {})", player_id, x, y);
        
        // Event is automatically sent after this method runs
    }
    
    #[event_sender(ItemCollected)]
    fn collect_item(&self, player_id: String, item_id: String) {
        println!("Collecting item {} for player {}", item_id, player_id);
    }
    
    // Regular method that uses event senders
    fn explore_area(&self, x: f32, y: f32) -> EventResult<()> {
        // Move to the location
        self.move_to(self.id.clone(), x, y)?;
        
        // Simulate finding an item
        let found_item = format!("treasure_{}{}", x as i32, y as i32);
        self.collect_item(self.id.clone(), found_item)?;
        
        Ok(())
    }
}

// Game controller - only emits events
#[derive(Debug)]
struct GameController {
    sender: EventSender
}

impl EventEmitter for GameController {
    fn sender(&self) -> &EventSender {
        &self.sender
    }
}

impl GameController {
    fn new(event_bus: &SharedEventBus) -> Self {
        Self {
            sender: EventSender::new(event_bus.clone()),
        }
    }
    
    // Event sender methods
    #[event_sender(PlayerDamaged)]
    fn damage_player(&self, player_id: String, amount: u32) {
        println!("Controller: Applying {} damage to player {}", amount, player_id);
    }
    
    // Run a simple game simulation
    fn run_simulation(&self, player: &Player) -> EventResult<()> {
        println!("\n=== STARTING SIMULATION ===\n");
        
        // Have the player explore some areas
        player.explore_area(10.0, 20.0)?;
        player.explore_area(30.0, 15.0)?;
        
        // Deal damage to the player
        self.damage_player(player.id.clone(), 30)?;
        
        // Move again
        player.move_to(player.id.clone(), 5.0, 5.0)?;
        
        println!("\n=== SIMULATION COMPLETE ===");
        Ok(())
    }
}

fn main() -> EventResult<()> {
    // Create the event bus
    let event_bus = SharedEventBus::new();
    
    // Create a player
    let mut player = Player::new("player1", &event_bus);
    player.activate(&event_bus);
    
    // Create the game controller
    let controller = GameController::new(&event_bus);
    
    // Run the simulation
    controller.run_simulation(&player)?;
    
    // Display player state
    println!("\nFinal player state:");
    println!("Position: ({}, {})", player.position.0, player.position.1);
    println!("Health: {}", player.health);
    println!("Inventory: {:?}", player.inventory);
    
    Ok(())
}