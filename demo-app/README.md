# Nexus Events Demo Application

This application demonstrates the Nexus Events system, a lightweight, thread-safe event management system for games built in Rust.

## Key Concepts Demonstrated

### Event Components

Event components are structs marked with the `#[event_component]` attribute. This annotation automatically registers event handlers at runtime.

```rust
#[event_component]
struct Player {
    name: String,
    hp: i32,
    // Other fields...
}
```

### Event Definitions
Events are simple structs with Clone and Debug implementations. They carry data across components.

```rust
#[derive(Debug, Clone)]
struct EnemyAttack {
    attacker_name: String,
    damage: u32,
    critical: bool,
}
```

### Event Handlers
Methods marked with #[event_handler(EventType)] automatically register to receive events of that type.

```rust
#[event_handler(EnemyAttack)]
fn on_attacked(&mut self, evt: &EnemyAttack) {
    self.hp -= evt.damage as i32;
    // Handle event data
}
```

### Event Senders
Methods marked with #[event_sender(EventType)] automatically dispatch events using their parameters.

```rust
#[event_sender(EnemyAttack)]
fn send_attack(&self, attacker_name: String, damage: u32, critical: bool) {
    // Pre-processing logic
    // Event is automatically dispatched after return
}
```

### Event Processing
Events are queued and processed in batch using the global event bus:

```rust
// Queue an event for later processing
dispatch(event);

// Process all queued events
process_events();
```

### Targeted Events
Events can target specific entities by including identifying information:

```rust
#[derive(Debug, Clone)]
struct TargetedAttack {
    target_name: String,
    damage: u32,
    attacker_name: String,
}

#[event_handler(TargetedAttack)]
fn on_targeted_attack(&mut self, evt: &TargetedAttack) {
    // Only process if we're the target
    if evt.target_name != self.name {
        return;
    }
    
    // We're the target! Handle event
```

# Demo App Controls
```
Q: Quit
Space: Random attack
WASD: Move player
F: Targeted attack
E: Add enemy
R: Remove enemy
T: Toggle automatic world updates
```

## Implementation Tips
- Thread Safety: Use Arc<Mutex<>> for shared state
- Avoid Boilerplate: Let the macros handle registration
- Structured Data: Pass clean, well-defined event structs
- Batch Processing: Queue events and process once per frame
- Targeted vs. Broadcast: Use both patterns as appropriate
