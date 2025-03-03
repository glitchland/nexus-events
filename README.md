# Nexus Events - Integration Guide for Turbo.Computer Games

Welcome to **Nexus Events**, a lightweight, thread-safe, annotation-based event system designed to streamline how you handle in-game events.
I built this for a[turbo.computer](https://turbo.computer/) game jam, for my submission "Nexus Weaver". Its a fantastic 2D game engine, you
should check it out!

This README provides step-by-step instructions on integrating Nexus Events into your tiny game projects, with examples.

---

## Table of Contents

1. [Overview](#overview)  
2. [Repository Structure](#repository-structure)  
3. [Installing and Building](#installing-and-building)  
4. [Core Concepts](#core-concepts)  
    - [Events](#events)  
    - [Event Components](#event-components)  
    - [Event Handlers](#event-handlers)  
    - [Event Senders](#event-senders)  
    - [Event Processing](#event-processing)  
5. [Integration with Turbo.Computer Games](#integration-with-turbocomputer-games)  
    - [Step 1: Add Nexus Events to Your Cargo.toml](#step-1-add-nexus-events-to-your-cargotoml)  
    - [Step 2: Annotate Your Game Classes](#step-2-annotate-your-game-classes)  
    - [Step 3: Dispatch and Process Events](#step-3-dispatch-and-process-events)  
    - [Step 4: (Optional) Using the Demo App as a Reference](#step-4-optional-using-the-demo-app-as-a-reference)  
6. [Usage Examples](#usage-examples)  
    - [Creating and Handling Events](#creating-and-handling-events)  
    - [Sending Events in Your Gameplay Code](#sending-events-in-your-gameplay-code)  
    - [Batch Processing in the Main Loop](#batch-processing-in-the-main-loop)  
7. [Performance and Debugging Tips](#performance-and-debugging-tips)  
8. [License](#license)

---

## Overview

Nexus Events simplifies in-game communications between different parts of your Turbo.Computer project by:

- Minimizing boilerplate through **procedural macros**  
- Allowing **thread-safe**, **asynchronous** event dispatch  
- Letting you **batch process** events once per game loop iteration  
- Keeping your code **clean** and **modular** with an event-driven design  

Whether you have NPCs attacking players, GUI elements updating, or multiplayer updates streaming in, Nexus Events can handle it all with minimal fuss.

---

## Repository Structure

In the repository, you’ll see the following files:

```
nexus-events/
  src/
    core/mod.rs        # Core event bus and queue logic
    lib.rs             # Re-exports, prelude, etc.
  Cargo.toml           # The nexus-events crate metadata

nexus-events-macros/
  src/lib.rs           # The procedural macros: #[event_component], #[event_handler], #[event_sender]
  Cargo.toml           # Macros crate metadata

demo-app/
  src/main.rs          # A demo application showcasing usage (CLI + TUI interface)
  Cargo.toml           # Demo app crate metadata

LICENSE                # MIT license
README.md              # This file or the top-level readme for the repository
```

**Key Folders and Files**:

- **`nexus-events/`**: The core library for the event system.  
- **`nexus-events-macros/`**: Houses the procedural macros that power the automatic registration system.  
- **`demo-app/`**: An example game-like application demonstrating how to use Nexus Events.  

---

## Installing and Building

1. **Clone or copy** the `nexus-events` and `nexus-events-macros` crates into your workspace.  
2. In your main game project, **add the local dependencies** in your `Cargo.toml`:
   ```toml
   [dependencies]
   nexus-events = { path = "<path to nexus-events-repo>" } // e.g. crates/nexus-events/nexus-events
   ```

3. **Build** your project:
   ```bash
   cargo build
   ```

---

## Core Concepts

### Events
Events are simple data structures that contain information about something that happened in your game. For instance:

```rust
#[derive(Debug, Clone)]
struct PlayerMoved {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
struct EnemyAttack {
    pub attacker_name: String,
    pub damage: u32,
    pub critical: bool,
}
```

They must implement `Send + Sync + 'static` so that they can be safely passed between threads and stored in the event queue.

### Event Components
An **Event Component** is any struct marked with `#[event_component]`. This macro sets up internal bookkeeping so that the system can automatically register all of its event handlers.

```rust
use nexus_events::prelude::*;

#[event_component]
struct Player {
    name: String,
    hp: i32,
}
```

When you instantiate a `Player`, its event handlers (annotated with `#[event_handler(...)]`) will automatically register to receive events of specified types.

### Event Handlers
**Event Handlers** are methods that respond when a particular event type is dispatched. They must be annotated with `#[event_handler(MyEventType)]`. Example:

```rust
#[event_handler(PlayerMoved)]
fn on_player_moved(&mut self, evt: &PlayerMoved) {
    println!("Player {} moves to {},{}!", self.name, evt.x, evt.y);
    // ... handle movement logic
}
```

### Event Senders
Sometimes you want to *send* an event from within a method. Mark such a method with `#[event_sender(MyEventType)]`. When the method returns, the system automatically creates and dispatches an event of type `MyEventType` using the parameters you specify.

```rust
#[derive(Debug, Clone)]
struct TargetedAttack {
    target_name: String,
    damage: u32,
    attacker_name: String,
}

#[event_component]
struct Player {
    name: String,
    hp: i32,
}

impl Player {
    // Example of an event sender:
    #[event_sender(TargetedAttack)]
    fn target_enemy(&self, target_name: String, damage: u32, attacker_name: String) {
        // Do any pre-dispatch logic here
        println!("{} is targeting {} for {} damage!", attacker_name, target_name, damage);
        // Event is automatically constructed and dispatched after this method completes
    }

    #[event_handler(TargetedAttack)]
    fn on_targeted_attack(&mut self, evt: &TargetedAttack) {
        if evt.target_name == self.name {
            self.hp -= evt.damage as i32;
            println!("Player {} took {} damage from {}!", self.name, evt.damage, evt.attacker_name);
        }
    }
}
```

### Event Processing

Nexus Events collects all dispatched events in a **global queue**. Call `process_events()` (or the bus’s `.process()` method) **once per frame** or *game loop iteration* to execute all queued events in FIFO order:

```rust
use nexus_events::prelude::*;

fn main_loop() {
    // ...
    process_events(); 
    // All queued events are handled, calling their registered handlers.
    // ...
}
```

---

## Integration with Turbo.Computer Games

Integrating Nexus Events into your **Turbo.Computer** game project involves four main steps:

### Step 1: Add Nexus Events to Your Cargo.toml

If you have a standard Cargo-based game project, open your `Cargo.toml` and add:

```toml
[dependencies]
nexus-events = { path = "path/to/nexus-events" }
```

(Adjust `"path/to"` as needed for your project layout.)

### Step 2: Annotate Your Game Classes

- Mark structs that need to handle or send events with `#[event_component]`.  
- For each method that should respond to events, add `#[event_handler(EventType)]`.  
- For each method that should send an event, add `#[event_sender(EventType)]`.  

Example:
```rust
use nexus_events::prelude::*;

#[derive(Debug, Clone)]
struct PlayerSpawned {
    pub name: String,
}

#[event_component]
struct MyPlayer {
    name: String,
    health: i32,
}

impl MyPlayer {
    // Handler for PlayerSpawned event
    #[event_handler(PlayerSpawned)]
    fn on_player_spawned(&mut self, event: &PlayerSpawned) {
        if event.name == self.name {
            println!("{} has spawned with {} health.", self.name, self.health);
        }
    }

    // Method to spawn a player - automatically sends the PlayerSpawned event
    #[event_sender(PlayerSpawned)]
    fn spawn(&self, name: String) {
        println!("Spawning player: {}", name);
        // The macro will dispatch a PlayerSpawned { name } event 
    }
}
```

### Step 3: Dispatch and Process Events

Within your Turbo.Computer game’s main loop or initialization routine, ensure you do the following:

1. **Initialize** the global event bus once.
2. **Instantiate** your event components (e.g., `MyPlayer`).
3. **Dispatch** any events as needed (you can also dispatch them from within your components).
4. **Process** queued events *once* each frame or tick.

**Pseudocode**:

```rust
fn main() {
    // 1) (Optional) The system auto-initializes the global bus, but you can ensure it’s up:
    let _bus = nexus_events::prelude::subscribe::<PlayerSpawned, _>(|_| { /* ... */ });
    // Alternatively, just rely on the default global initialization.

    // 2) Create your components
    let player = MyPlayer { name: "Hero".to_string(), health: 100 };

    // 3) Dispatch an event
    // Option A: Direct dispatch:
    nexus_events::prelude::dispatch(PlayerSpawned { name: "Hero".to_string() });
    // Option B: Using an event-sender method:
    player.spawn("Hero".to_string());

    // 4) Process events
    nexus_events::prelude::process_events();

    // ... proceed with your game loop ...
}
```

### Step 4: (Optional) Using the Demo App as a Reference

The **demo-app** directory contains a **fully functional TUI (Text User Interface) game loop** showcasing:

- Automatic event handler registration  
- Event sending and handling  
- Real-time game loop with **keyboard input**, **metrics tracking**, and **UI updates**  

Check out [`demo-app/src/main.rs`](demo-app/src/main.rs in your repository) to see an end-to-end example you can adapt to your Turbo.Computer game’s structure.

---

## Usage Examples

### Creating and Handling Events

1. **Create** your event type:
   ```rust
   #[derive(Debug, Clone)]
   struct ScoreUpdated {
       pub player_name: String,
       pub new_score: u32,
   }
   ```
2. **Write** the handler in a component:
   ```rust
   #[event_component]
   struct Scoreboard;

   impl Scoreboard {
       #[event_handler(ScoreUpdated)]
       fn on_score_updated(&mut self, evt: &ScoreUpdated) {
           println!("Scoreboard sees {}'s score is now {}!", evt.player_name, evt.new_score);
       }
   }
   ```

### Sending Events in Your Gameplay Code

You can either call `dispatch(ScoreUpdated { ... })` directly, or **use the event-sender macro**:

```rust
#[event_sender(ScoreUpdated)]
fn update_score(&self, player_name: String, new_score: u32) {
    // This method will automatically construct and dispatch
    // a ScoreUpdated event with the arguments passed in.
    println!("Updating {}'s score to {}", player_name, new_score);
}
```

### Batch Processing in the Main Loop

Make sure you call `process_events()` once per **game loop** iteration to flush and handle all queued events:

```rust
fn game_loop() {
    loop {
        // ...
        // Update game logic, possibly calling event-sending methods
        // ...
        process_events(); // Handle all events
        // ...
        // Render game or proceed to next frame
    }
}
```

---

## Performance and Debugging Tips

1. **Process Once Per Tick**: Call `process_events()` only once per frame, rather than multiple times, to keep event handling deterministic.  
2. **Avoid Excessive Lock Contention**: If your Turbo.Computer game is highly parallelized, consider grouping event dispatch calls or using smaller, more focused events.  
3. **Use the Demo**: The `demo-app` in this repository showcases an event-driven TUI and includes **metrics tracking** (frames per second, event throughput, etc.) to illustrate how you can measure performance.

---

## License

This project is licensed under the terms of the **MIT License**. See [LICENSE](LICENSE) for details.
