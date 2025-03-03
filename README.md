# Nexus Events

A lightweight, barebones, simple broadcast event system, using an attribute-based annotation system. Created
to make game events easier to manage.

## Features
- Type-safe event handling with Rust's type system
- Declarative event handling with attribute annotation approach
- Thread-safe event dispatching
- Fast


cargo install cargo-expand  # Useful for debugging macros

# Cargo install dev dependencies
# For code formatting
cargo install rustfmt

# For linting
cargo install clippy

# For benchmarking (if not using the built-in bench)
cargo install criterion

# For documentation generation
cargo install cargo-docs

# For visualization of crate dependencies
cargo install cargo-deps

# Expand macros to see how they're processed
cargo expand --bin stress_test

# Run benchmarks
cargo bench

# Run specific tests
cargo test nexus_events::test_basic_subscribe_publish -- --exact

# Build everything in debug mode
cargo build

# Build in release mode for better performance
cargo build --release

# Run directly
cargo run --bin stress_test
cargo build --bin stress_test

# Or with release optimizations
cargo run --release --bin stress_test
cargo build --lib


# Run all tests
cargo test

# Create a simple test file to see how macros expand
echo 'use nexus_events::*; #[event_handler(MyEvent)] fn handle_event(event: &MyEvent) {}' > macro_test.rs

# Expand the macros
cargo expand --bin stress_test

# Run benchmarks
cargo bench


## Usage

```rust
use nexus_events::prelude::*;
use macros::event_handler;

#[event_component]
struct Player {
    id: String,
    // ...
}

impl Player {
    #[event_handler(PlayerMoved)]
    fn on_player_moved(&mut self, event: &PlayerMoved) {
        println!("Player {} moves to {},{}!", self.id, event.x, event.y);
    }

    #[event_handler(PlayerDamaged)]
    fn handle_damage(&mut self, event: &PlayerDamaged) {
        println!("Player {} took {} damage!", self.id, event.amount);
    }
}

// Then, at runtime:
fn main() {
    let mut player = Player { id: String::from("Alice") };
    let movement = PlayerMoved { x: 10.0, y: 20.0 };    
    nexus_events::registry::dispatch_event(&mut player, &movement);
}
```

# Testing

cargo test nexus_events::test_basic_subscribe_publish -- --exact