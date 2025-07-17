use bevy::prelude::*;
use bevy::window::{WindowPlugin, WindowResolution};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use spooky_core::events::npc::NpcCollisionEvent;
use spooky_core::events::player::PlayerInputEvent;
use spooky_core::events::walker::WalkerCollisionEvent;
use spooky_core::events::{coin::CoinCollisionEvent, dynamite::DynamiteCollisionEvent};
use spooky_core::resources::MazeSeed;
use spooky_core::systems::hud::HudState;
use spooky_core::{systems, systems::collisions};
use wasm_bindgen::prelude::*;
use web_sys::console;

mod wasm_input;
use wasm_input::WasmInputPlugin;

// Input queue for buffering input events
#[derive(Resource, Clone, Default)]
pub struct InputQueue {
    queue: Arc<Mutex<VecDeque<PlayerInputEvent>>>,
}

#[wasm_bindgen]
pub struct SpookyMazeWasm {
    app: App,
    input_queue: Arc<Mutex<VecDeque<PlayerInputEvent>>>,
}

#[wasm_bindgen]
impl SpookyMazeWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize console logging for debugging
        console_error_panic_hook::set_once();
        
        console::log_1(&"Initializing Spooky Maze WASM".into());
        
        let input_queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut app = App::new();
        
        // Add plugins needed for WASM
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Spooky Maze Game".into(),
                    resolution: WindowResolution::new(320.0, 240.0),
                    canvas: Some("#gameCanvas".into()),
                    ..default()
                }),
                ..default()
            }),
            WasmInputPlugin, // Custom input handling for WASM
        ));
        
        app.insert_resource(MazeSeed(Some({
            let mut seed = [0u8; 32];
            getrandom::getrandom(&mut seed).unwrap();
            seed
        })))
        .add_systems(Startup, systems::setup::setup)
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        .add_event::<PlayerInputEvent>()
        .add_event::<CoinCollisionEvent>()
        .add_event::<DynamiteCollisionEvent>()
        .add_event::<WalkerCollisionEvent>()
        .add_event::<NpcCollisionEvent>()
        .insert_resource(HudState::default())
        .insert_resource(InputQueue { queue: input_queue.clone() })
        .add_systems(
            FixedUpdate,
            (
                process_input_queue,
                systems::process_player_input::process_player_input,
                collisions::coin::detect_coin_collision,
                collisions::coin::remove_coin_on_collision,
                collisions::dynamite::handle_dynamite_collision,
                collisions::walker::detect_walker_collision,
                collisions::walker::handle_walker_collision,
                collisions::npc::detect_npc_collision,
                collisions::npc::handle_npc_collision,
                systems::dynamite_logic::handle_dynamite_collision,
                systems::npc_logic::update_npc_movement,
                systems::game_logic::update_game,
            ),
        );
        
        console::log_1(&"Spooky Maze WASM initialized".into());
        
        Self { app, input_queue }
    }
    
    #[wasm_bindgen]
    pub fn run(&mut self) {
        console::log_1(&"Running Spooky Maze WASM".into());
        self.app.run();
    }
    
    #[wasm_bindgen]
    pub fn move_up(&mut self) {
        self.send_input(0.0, 16.0);
    }
    
    #[wasm_bindgen]
    pub fn move_down(&mut self) {
        self.send_input(0.0, -16.0);
    }
    
    #[wasm_bindgen]
    pub fn move_left(&mut self) {
        self.send_input(-16.0, 0.0);
    }
    
    #[wasm_bindgen]
    pub fn move_right(&mut self) {
        self.send_input(16.0, 0.0);
    }
    
    #[wasm_bindgen]
    pub fn teleport(&mut self) {
        // Teleport functionality - this would need to be implemented
        // as a separate event or action in the core game logic
        console::log_1(&"Teleport requested".into());
    }
    
    #[wasm_bindgen]
    pub fn place_dynamite(&mut self) {
        // Place dynamite functionality - this would need to be implemented
        // as a separate event or action in the core game logic
        console::log_1(&"Place dynamite requested".into());
    }
    
    fn send_input(&mut self, dx: f32, dy: f32) {
        if let Ok(mut queue) = self.input_queue.lock() {
            queue.push_back(PlayerInputEvent { dx, dy });
            console::log_1(&format!("Input queued: dx={}, dy={}", dx, dy).into());
        } else {
            console::log_1(&"Failed to lock input queue".into());
        }
    }
}

// System to process input events from the queue
fn process_input_queue(
    input_queue: Res<InputQueue>,
    mut player_input_events: EventWriter<PlayerInputEvent>,
) {
    if let Ok(mut queue) = input_queue.queue.lock() {
        while let Some(event) = queue.pop_front() {
            player_input_events.write(event);
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"WASM module loaded".into());
}
