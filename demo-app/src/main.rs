#![allow(dead_code)]

use nexus_events::prelude::*;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{Rng, thread_rng};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction as Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{
    io::self,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
    collections::HashMap,
};

////////////////////////////////////////////////////////
// 1) Event Definitions
////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
struct GameTick {
    dt: f32,
    frame_number: u64,
}

#[derive(Debug, Clone)]
struct EnemyAttack {
    attacker_name: String,
    damage: u32,
    critical: bool,
}

#[derive(Debug, Clone)]
struct TargetedAttack {
    target_name: String,
    damage: u32,
    attacker_name: String,
}

#[derive(Debug, Clone)]
struct PlayerMovement {
    direction: MoveDirection,
    speed: f32,
}

#[derive(Debug, Clone)]
struct WorldUpdate {
    elapsed_time: f32,
    active_entities: usize,
}

#[derive(Debug, Clone, Copy)]
enum MoveDirection {
    North,
    South, 
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

////////////////////////////////////////////////////////
// 2) Performance Metrics
////////////////////////////////////////////////////////
#[derive(Debug)]
struct MetricsTracker {
    logs: Vec<String>,
    events_processed: usize,
    events_by_type: HashMap<String, usize>,
    handler_calls: usize,
    frame_times: Vec<Duration>,
    max_frame_time: Duration,
    min_frame_time: Duration,
    start_time: Instant,
    event_components: usize,
}

impl MetricsTracker {
    fn new() -> Self {
        Self {
            logs: Vec::with_capacity(20),
            events_processed: 0,
            events_by_type: HashMap::new(),
            handler_calls: 0,
            frame_times: Vec::with_capacity(120),
            max_frame_time: Duration::from_nanos(0),
            min_frame_time: Duration::from_secs(1),
            start_time: Instant::now(),
            event_components: 0,
        }
    }
    
    fn record_event(&mut self, event_type: &str) {
        self.events_processed += 1;
        *self.events_by_type.entry(event_type.to_string()).or_insert(0) += 1;
    }
    
    fn record_handler_call(&mut self) {
        self.handler_calls += 1;
    }
    
    fn add_frame_time(&mut self, dt: Duration) {
        self.frame_times.push(dt);
        if self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }
        
        if dt > self.max_frame_time {
            self.max_frame_time = dt;
        }
        if dt < self.min_frame_time {
            self.min_frame_time = dt;
        }
    }
    
    fn increment_components(&mut self, count: usize) {
        self.event_components = count;
    }

    fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::from_nanos(0);
        }
        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }
    
    fn push_log(&mut self, line: String) {
        if self.logs.len() > 15 {
            self.logs.remove(0);
        }
        self.logs.push(line);
    }
}

////////////////////////////////////////////////////////
// 3) UI Model
////////////////////////////////////////////////////////
#[derive(Debug)]
struct UIModel {
    game_logs: Vec<String>,
    enemy_logs: Vec<String>,
    event_logs: Vec<String>,
    metrics: Arc<Mutex<MetricsTracker>>,
}

impl UIModel {
    fn new(metrics: Arc<Mutex<MetricsTracker>>) -> Self {
        Self {
            game_logs: Vec::new(),
            enemy_logs: Vec::new(),
            event_logs: Vec::new(),
            metrics,
        }
    }

    fn push_game_log(&mut self, line: String) {
        if self.game_logs.len() > 8 {
            self.game_logs.remove(0);
        }
        self.game_logs.push(line.clone());
        
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push_log(format!("[Game] {}", line));
        }
    }
    
    fn push_enemy_log(&mut self, line: String) {
        if self.enemy_logs.len() > 8 {
            self.enemy_logs.remove(0);
        }
        self.enemy_logs.push(line.clone());
        
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push_log(format!("[Enemy] {}", line));
        }
    }
    
    fn push_event_log(&mut self, line: String) {
        if self.event_logs.len() > 8 {
            self.event_logs.remove(0);
        }
        self.event_logs.push(line.clone()); 
        
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push_log(format!("[Event] {}", line));
        }
    }

    fn draw<B: Backend>(&self, frame: &mut tui::Frame<B>) {
        // Get performance metrics
        let runtime;
        let fps;
        let avg_frame_ms;
        let min_frame_ms;
        let max_frame_ms;
        let events_per_sec;
        let handlers_per_sec;
        let event_total;
        let handler_total;
        let evented_components;
        
        if let Ok(metrics) = self.metrics.lock() {
            runtime = metrics.start_time.elapsed().as_secs_f32();
            fps = 1.0 / metrics.average_frame_time().as_secs_f32().max(0.000001);
            avg_frame_ms = metrics.average_frame_time().as_micros() as f32 / 1000.0;
            min_frame_ms = metrics.min_frame_time.as_micros() as f32 / 1000.0;
            max_frame_ms = metrics.max_frame_time.as_micros() as f32 / 1000.0;
            events_per_sec = metrics.events_processed as f32 / runtime.max(0.001);
            handlers_per_sec = metrics.handler_calls as f32 / runtime.max(0.001);
            event_total = metrics.events_processed;
            handler_total = metrics.handler_calls;
            evented_components = metrics.event_components;
        } else {
            // Defaults if can't get lock
            runtime = 0.0;
            fps = 0.0;
            avg_frame_ms = 0.0;
            min_frame_ms = 0.0;
            max_frame_ms = 0.0;
            events_per_sec = 0.0;
            handlers_per_sec = 0.0;
            event_total = 0;
            handler_total = 0;
            evented_components = 0;
        }
        
        // Create main layout
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5),  // Stats header
            Constraint::Min(10),    // Logs area
            Constraint::Length(3),  // Controls
        ].as_ref())
        .split(frame.size());
            
        // Stats header
        let stats_text = vec![
            Spans::from(vec![
                Span::styled("NEXUS EVENTS PERFORMANCE DEMO", Style::default().fg(Color::Yellow))
            ]),
            Spans::from(vec![
                Span::raw(format!("Runtime: {:.1}s | FPS: {:.1} | Frame: {:.2}ms (min: {:.2} max: {:.2})",
                    runtime, fps, avg_frame_ms, min_frame_ms, max_frame_ms))
            ]),
            Spans::from(vec![
                Span::raw(format!("Events: {} ({:.1}/sec) | Handler calls: {} ({:.1}/sec) | EventedComponents: {}",
                    event_total, events_per_sec, handler_total, handlers_per_sec, evented_components))
            ]),
        ];
        
        let stats_para = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title("Performance Stats"));
        frame.render_widget(stats_para, chunks[0]);
        
        // Logs area - split horizontally
        let logs_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // Game and Event logs
                Constraint::Percentage(50),  // Enemy logs
            ].as_ref())
            .split(chunks[1]);
            
        // Split left side (Game + Event logs) vertically
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),  // Game logs
                Constraint::Percentage(50),  // Event logs
            ].as_ref())
            .split(logs_chunks[0]);
        
        // Game logs
        let game_log_items: Vec<ListItem> = self.game_logs
            .iter()
            .map(|s| ListItem::new(s.clone()))
            .collect();
            
        let game_logs = List::new(game_log_items)
            .block(Block::default().borders(Borders::ALL).title("Game Logs"));
        frame.render_widget(game_logs, left_chunks[0]);
        
        // Event logs
        let event_log_items: Vec<ListItem> = self.event_logs
            .iter()
            .map(|s| ListItem::new(s.clone()))
            .collect();
            
        let event_logs = List::new(event_log_items)
            .block(Block::default().borders(Borders::ALL).title("Event Logs"));
        frame.render_widget(event_logs, left_chunks[1]);
        
        // Enemy logs
        let enemy_log_items: Vec<ListItem> = self.enemy_logs
            .iter()
            .map(|s| ListItem::new(s.clone()))
            .collect();
            
        let enemy_logs = List::new(enemy_log_items)
            .block(Block::default().borders(Borders::ALL).title("Enemy Logs"));
        frame.render_widget(enemy_logs, logs_chunks[1]);
        
        // Controls section
        let controls_text = vec![
            Spans::from(vec![
                Span::raw("Q: Quit | Space: Attack | WASD: Move | F: Targeted Attack | E: Add Enemy | R: Remove Enemy | T: Toggle Auto-Events")
            ]),
        ];
        
        let controls = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(controls, chunks[2]);
    }
}

////////////////////////////////////////////////////////
// 4) Game World
////////////////////////////////////////////////////////
#[event_component]
struct World {
    elapsed_time: f32,
    entities_count: usize,
    auto_events: bool,
    ui: Arc<Mutex<UIModel>>,
    metrics: Arc<Mutex<MetricsTracker>>,
}

impl World {
    fn new(ui: Arc<Mutex<UIModel>>, metrics: Arc<Mutex<MetricsTracker>>) -> Self {
        Self {
            elapsed_time: 0.0,
            entities_count: 0,
            auto_events: true,
            ui,
            metrics,
        }
    }

    fn update(&mut self, dt: f32) {
        self.elapsed_time += dt;
        
        if self.auto_events {
            self.send_world_update(self.elapsed_time, self.entities_count);
        }
        
        // Record metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_handler_call();
        }
    }

    fn add_entity(&mut self) {
        self.entities_count += 1;
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_game_log(format!("Entity added (total: {})", self.entities_count));
        }

        // Update the component count in metrics when adding an entity
        if let Ok(mut metrics) = self.metrics.lock() {
            // Base count is 2 (world + player) + current enemies count
            metrics.increment_components(2 + self.entities_count);
        }
    }
    
    fn remove_entity(&mut self) {
        if self.entities_count > 0 {
            self.entities_count -= 1;
            if let Ok(mut ui) = self.ui.lock() {
                ui.push_game_log(format!("Entity removed (total: {})", self.entities_count));
            }
        }

        // Update the component count in metrics when removing an entity
        if let Ok(mut metrics) = self.metrics.lock() {
            // Base count is 2 (world + player) + current enemies count
            metrics.increment_components(2 + self.entities_count);
        }
    }
    
    fn toggle_auto_events(&mut self) {
        self.auto_events = !self.auto_events;
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_game_log(format!("Auto events: {}", if self.auto_events { "ON" } else { "OFF" }));
        }
    }
    
    #[event_handler(GameTick)]
    fn on_tick(&mut self, evt: &GameTick) {
        if let Ok(mut ui) = self.ui.lock() {
            if evt.frame_number % 60 == 0 {  // Only log every 60th frame to reduce spam
                ui.push_game_log(format!("Tick frame #{} (dt={:.3}ms)", 
                    evt.frame_number, evt.dt * 1000.0));
            }
        }
        
        // Record the event and handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_event("GameTick");
            metrics.record_handler_call();
        }
    }
    
    #[event_handler(PlayerMovement)]
    fn on_player_movement(&mut self, evt: &PlayerMovement) {
        let dir_str = match evt.direction {
            MoveDirection::North => "North",
            MoveDirection::South => "South",
            MoveDirection::East => "East",
            MoveDirection::West => "West",
            MoveDirection::NorthEast => "North-East",
            MoveDirection::NorthWest => "North-West",
            MoveDirection::SouthEast => "South-East",
            MoveDirection::SouthWest => "South-West",
        };
        
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_game_log(format!("Player moved {} (speed: {:.1})", dir_str, evt.speed));
        }
        
        // Record the event and handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_event("PlayerMovement");
            metrics.record_handler_call();
        }
    }
    
    #[event_sender(WorldUpdate)]
    fn send_world_update(&self, elapsed_time: f32, active_entities: usize) {
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_game_log(format!("World update: t={:.1}s, entities={}", 
                elapsed_time, active_entities));
        }
        
        // The event_sender macro will dispatch the event after this method returns
        
        // Record metrics for event sender
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_handler_call();
        }
    }
}

////////////////////////////////////////////////////////
// 5) Enemy
////////////////////////////////////////////////////////
#[event_component]
struct Enemy {
    name: String,
    hp: i32,
    position: (f32, f32),
    tick_handler: Option<HandlerId>,
    unsub_time: Option<Instant>,
    ui: Arc<Mutex<UIModel>>,
    metrics: Arc<Mutex<MetricsTracker>>,
}

impl Enemy {
    fn new(name: &str, ui: Arc<Mutex<UIModel>>, metrics: Arc<Mutex<MetricsTracker>>) -> Self {
        Self {
            name: name.to_string(),
            hp: 100,
            position: (0.0, 0.0),
            tick_handler: None,
            unsub_time: None,
            ui,
            metrics,
        }
    }

    // handle GameTick
    #[event_handler(GameTick)]
    fn on_tick(&mut self, evt: &GameTick) {
        // Only log occasionally to reduce spam
        if evt.frame_number % 60 == 0 {
            if let Ok(mut ui) = self.ui.lock() {
                ui.push_enemy_log(format!("({}) active at frame #{}", self.name, evt.frame_number));
            }
        }

        // Random chance to send Attack (10% per second, scaled by dt)
        let mut rng = thread_rng();
        let attack_chance = 0.1 * evt.dt;  // 10% chance per second, scaled by dt
        
        if rng.gen::<f32>() < attack_chance {
            let is_critical = rng.gen_bool(0.2); // 20% chance for critical hit
            let base_damage = rng.gen_range(5..=20);
            let damage = if is_critical { base_damage * 2 } else { base_damage };
            
            // Call event sender method
            self.send_attack(self.name.clone(), damage, is_critical);
        }
        
        // Random chance to move (30% per second, scaled by dt)
        let move_chance = 0.3 * evt.dt;
        if rng.gen::<f32>() < move_chance {
            // Update position
            let dx = rng.gen_range(-10.0..10.0);
            let dy = rng.gen_range(-10.0..10.0);
            self.position.0 += dx;
            self.position.1 += dy;
            
            if let Ok(mut ui) = self.ui.lock() {
                ui.push_enemy_log(format!("({}) moved to ({:.1},{:.1})", 
                    self.name, self.position.0, self.position.1));
            }
        }

        // Maybe schedule unsub in 2-5 seconds (5% chance per second)
        let unsub_chance = 0.05 * evt.dt;
        if self.unsub_time.is_none() && rng.gen::<f32>() < unsub_chance {
            let unsub_delay = rng.gen_range(2..=5);
            self.unsub_time = Some(Instant::now() + Duration::from_secs(unsub_delay));
            
            if let Ok(mut ui) = self.ui.lock() {
                ui.push_enemy_log(format!("({}) will unsub in {}s", self.name, unsub_delay));
            }
        }

        // Check if it's time to unsub/resub
        if let Some(t) = self.unsub_time {
            if Instant::now() >= t {
                // unsub if we had a handler
                if let Some(hid) = self.tick_handler.take() {
                    let _ = unsubscribe::<GameTick>(hid);
                    if let Ok(mut ui) = self.ui.lock() {
                        ui.push_enemy_log(format!("({}) unsubscribed from Tick", self.name));
                    }
                }
                // Schedule resub in 1-3 seconds
                let resub_delay = rng.gen_range(1..=3);
                self.unsub_time = Some(Instant::now() + Duration::from_secs(resub_delay));
                
                if let Ok(mut ui) = self.ui.lock() {
                    ui.push_enemy_log(format!("({}) will re-sub in {}s", self.name, resub_delay));
                }
            } 
            else if self.tick_handler.is_none() && Instant::now() + Duration::from_millis(500) >= t {
                // Resubscribe with a closure that invokes our actual handler
                let name = self.name.clone();
                let ui = self.ui.clone();
                
                let hid = subscribe::<GameTick, _>(move |_evt| {
                    // Just a placeholder - our on_tick handler does the real work
                    if let Ok(mut ui_lock) = ui.lock() {
                        ui_lock.push_event_log(format!("({}) received tick via closure", name));
                    }
                });
                
                self.tick_handler = Some(hid);
                if let Ok(mut ui) = self.ui.lock() {
                    ui.push_enemy_log(format!("({}) re-subscribed to Tick", self.name));
                }
                self.unsub_time = None;
            }
        }
        
        // Record the event and handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_handler_call();
        }
    }

    #[event_handler(TargetedAttack)]
    fn on_targeted_attack(&mut self, evt: &TargetedAttack) {
        // Only process if we're the target
        if evt.target_name != self.name {
            return;
        }
        
        // We're the target! Take damage
        self.hp -= evt.damage as i32;
        
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_enemy_log(format!("({}) was TARGETED by {} for {} damage! HP={}",
                self.name, evt.attacker_name, evt.damage, self.hp));
        }
        
        // Record the event
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_event("TargetedAttack");
            metrics.record_handler_call();
        }
    }

    // handle an EnemyAttack
    #[event_handler(EnemyAttack)]
    fn on_attacked(&mut self, evt: &EnemyAttack) {
        // Skip self-attacks
        if evt.attacker_name == self.name {
            return;
        }
        
        eprintln!("Enemy {} attacked by {}", self.name, evt.attacker_name);

        self.hp -= evt.damage as i32;
        
        let attack_type = if evt.critical { "CRITICAL" } else { "normal" };
        
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_enemy_log(format!("({}) hit by {} for {} dmg ({}) → HP={}",
                self.name, evt.attacker_name, evt.damage, attack_type, self.hp));
        }
        
        // Check if defeated
        if self.hp <= 0 {
            if let Ok(mut ui) = self.ui.lock() {
                ui.push_enemy_log(format!("({}) was defeated!", self.name));
            }
            
            // In a real game, we'd remove the enemy here
            // For demo purposes, just reset HP
            self.hp = 100;
        }
        
        // Record the event and handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_event("EnemyAttack");
            metrics.record_handler_call();
        }
    }

    // Handler for world updates
    #[event_handler(WorldUpdate)]
    fn on_world_update(&mut self, evt: &WorldUpdate) {
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_enemy_log(format!("({}) received world update: t={:.1}s", 
                self.name, evt.elapsed_time));
        }
        
        // Record the event and handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_event("WorldUpdate");
            metrics.record_handler_call();
        }
    }
    
    // sending an Attack
    #[event_sender(EnemyAttack)]
    fn send_attack(&self, attacker_name: String, damage: u32, critical: bool) {
        if let Ok(mut ui) = self.ui.lock() {
            let attack_type = if critical { "CRITICAL" } else { "normal" };
            ui.push_enemy_log(format!("({}) sending {} attack: {} dmg", 
                self.name, attack_type, damage));
        }
        
        // Record handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_handler_call();
        }
    }
}

////////////////////////////////////////////////////////
// 6) Player
////////////////////////////////////////////////////////

#[event_component]
struct Player {
    name: String,
    hp: i32,
    position: (f32, f32),
    ui: Arc<Mutex<UIModel>>,
    metrics: Arc<Mutex<MetricsTracker>>,
}

impl Player {
    fn new(name: &str, ui: Arc<Mutex<UIModel>>, metrics: Arc<Mutex<MetricsTracker>>) -> Self {
        Self {
            name: name.to_string(),
            hp: 100,
            position: (0.0, 0.0),
            ui,
            metrics,
        }
    }
    
    // Handle being attacked
    #[event_handler(EnemyAttack)]
    fn on_attacked(&mut self, evt: &EnemyAttack) {
        self.hp -= evt.damage as i32;
        
        let attack_type = if evt.critical { "CRITICAL" } else { "normal" };
        
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_event_log(format!("Player hit by {} for {} dmg ({}) → HP={}",
                evt.attacker_name, evt.damage, attack_type, self.hp));
        }
        
        // Reset HP if defeated
        if self.hp <= 0 {
            if let Ok(mut ui) = self.ui.lock() {
                ui.push_event_log(format!("Player was defeated! HP reset."));
            }
            self.hp = 100;
        }
        
        // Record the event and handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_event("EnemyAttack");
            metrics.record_handler_call();
        }
    }
    
    #[event_sender(TargetedAttack)]
    fn target_attack(&self, target_name: String, damage: u32, attacker_name: String) {
        if let Ok(mut ui) = self.ui.lock() {
            // Add more descriptive log message
            ui.push_event_log(format!("Player targeting {} with {} damage - dispatching TargetedAttack", 
                target_name, damage));
        }
        
        // Record handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_event("TargetedAttack");
            metrics.record_handler_call();
        }
    }

    // Send player movement event
    #[event_sender(PlayerMovement)]
    fn move_player(&self, direction: MoveDirection, speed: f32) {
        if let Ok(mut ui) = self.ui.lock() {
            ui.push_event_log(format!("Player sending movement event ({:?})", 
                direction));
        }
        
        // Record handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_handler_call();
        }
    }
    
    // Send attack event
    #[event_sender(EnemyAttack)]
    fn attack(&self, attacker_name: String, damage: u32, critical: bool) {
        if let Ok(mut ui) = self.ui.lock() {
            let attack_type = if critical { "CRITICAL" } else { "normal" };
            ui.push_event_log(format!("Player sending {} attack: {} dmg", 
                attack_type, damage));
        }
        
        // Record handler call in metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_handler_call();
        }
    }
}

////////////////////////////////////////////////////////
fn main() -> io::Result<()> {
    eprintln!("Starting demo app...");

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?; 
    
    // Set up metrics and UI
    let metrics = Arc::new(Mutex::new(MetricsTracker::new()));
    let ui = Arc::new(Mutex::new(UIModel::new(metrics.clone())));

    // Create game world and entities
    let mut world = World::new(ui.clone(), metrics.clone());
    let player = Player::new("Player1", ui.clone(), metrics.clone());
    let mut enemies = Vec::new();
    for i in 1..=3 {
        enemies.push(Enemy::new(&format!("Enemy{}", i), ui.clone(), metrics.clone()));
    }
    world.entities_count = enemies.len();
    
    // Game loop variables
    let mut last_frame = Instant::now();
    let mut running = true;
    let mut frame_number: u64 = 0;
    
    // Main game loop
    while running {
        // Calculate delta time
        let now = Instant::now();
        let dt = now.duration_since(last_frame);
        let dt_seconds = dt.as_secs_f32();
        last_frame = now;
        frame_number += 1;
        
        // Record frame time in metrics
        if let Ok(mut m) = metrics.lock() {
            m.add_frame_time(dt);
        }
        
        // Process input 
        if event::poll(std::time::Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        running = false;
                    },
                    KeyCode::Char(' ') => {
                        // Space: Player attack
                        let mut rng = thread_rng();
                        let damage = rng.gen_range(10..=30);
                        let critical = rng.gen_bool(0.2);
                        player.attack(player.name.clone(), damage, critical);
                    },
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        // F: Targeted attack on a random enemy
                        if !enemies.is_empty() {
                            let mut rng = thread_rng();
                            let target_idx = rng.gen_range(0..enemies.len());
                            let target = &enemies[target_idx];
                            let damage = rng.gen_range(15..35);
                            player.target_attack(target.name.clone(), damage, player.name.clone());
                        }
                    },
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        // W: Move North
                        player.move_player(MoveDirection::North, 3.0);
                    },
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        // A: Move West
                        player.move_player(MoveDirection::West, 3.0);
                    },
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        // S: Move South
                        player.move_player(MoveDirection::South, 3.0);
                    },
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        // D: Move East
                        player.move_player(MoveDirection::East, 3.0);
                    },
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        // E: Add Enemy
                        world.add_entity();
                    },
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // R: Remove Enemy
                        world.remove_entity();
                    },
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        // T: Toggle Auto-Events
                        world.toggle_auto_events();
                    },
                    _ => {}
                }
            }
        }
        
        // Update game state 
        world.update(dt_seconds);
        let tick = GameTick { dt: dt_seconds, frame_number };
        dispatch(tick);
        process_events();

        // Force enemies to do something visible occasionally
        if frame_number % 30 == 0 {
            for enemy in &enemies {
                // Trigger a random attack
                let mut rng = thread_rng();
                let damage = rng.gen_range(3..10);
                let critical = rng.gen_bool(0.1);
                enemy.send_attack(enemy.name.clone(), damage, critical);
            }
        }
        
        // Record events
        if let Ok(mut m) = metrics.lock() {
            m.record_event("GameTick");
            m.record_event("EnemyAttack");
            m.record_event("PlayerMovement");
            m.record_event("WorldUpdate");

            // Only count components once at startup, not every frame
            if frame_number == 1 {
                // Use the already acquired lock instead of trying to get it again
                m.increment_components(2 + enemies.len());
            }
        }

        // Draw UI using TUI
        terminal.draw(|f| {
            if let Ok(ui) = ui.lock() {
                ui.draw(f);
            }
        })?;
        
        // Frame rate limiting (same as before)
        let frame_time = now.elapsed();
        let target_frame_time = Duration::from_micros(16667); // ~60 FPS
        
        if frame_time < target_frame_time {
            thread::sleep(target_frame_time - frame_time);
        }
    }

    // Cleanup and exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    // Final stats remain the same
    println!("\nNEXUS EVENTS PERFORMANCE DEMO\n");
    println!("========== FINAL STATISTICS ==========");

    if let Ok(m) = metrics.lock() {
        let runtime = m.start_time.elapsed().as_secs_f32();
        println!("Runtime: {:.2} seconds", runtime);
        println!("Total frames: {}", frame_number);
        println!("Average FPS: {:.1}", frame_number as f32 / runtime);
        println!("Average frame time: {:.2}ms", m.average_frame_time().as_micros() as f32 / 1000.0);
        println!("Min frame time: {:.2}ms", m.min_frame_time.as_micros() as f32 / 1000.0);
        println!("Max frame time: {:.2}ms", m.max_frame_time.as_micros() as f32 / 1000.0);
        println!("\nEVENT STATISTICS:");
        println!("Total events processed: {}", m.events_processed);
        println!("Events per second: {:.1}", m.events_processed as f32 / runtime);
        println!("Total handler calls: {}", m.handler_calls);
        println!("Handler calls per second: {:.1}", m.handler_calls as f32 / runtime);
        
        println!("\nEVENTS BY TYPE:");
        for (event_type, count) in &m.events_by_type {
            println!("  {}: {} ({:.1}%)", 
                     event_type, 
                     count, 
                     *count as f32 / m.events_processed as f32 * 100.0);
        }
    }
    
    Ok(())
}