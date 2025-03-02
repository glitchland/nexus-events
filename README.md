# Nexus Events

A lightweight, attribute-based event system for Rust games.

## Features
- Type-safe event handling with Rust's type system
- Declarative event handling with attributes
- Automatic subscription management
- Thread-safe event dispatching
- Reasonably fast
- Unique run-time patching approach to allow for easier integration to existing codebase

## Usage

```rust
#[derive(Debug, EventSubscriber)]
struct Player {
    id: String,
    active: bool,
    subscriptions: SubscriptionSet,
    // ...fields...
}

impl Player {
    // Users just add this attribute and everything works!
    #[event_handler(PlayerMoved)]
    fn on_player_moved(&mut self, event: &PlayerMoved) {
        // Handle the event...
    }
    
    // Another handler - zero configuration needed
    #[event_handler(PlayerDamaged)]
    fn handle_damage(&mut self, event: &PlayerDamaged) {
        // Handle damage...
    }
}
```