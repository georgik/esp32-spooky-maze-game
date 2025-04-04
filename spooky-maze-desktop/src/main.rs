mod desktop_systems;

use crate::desktop_systems::hud::{setup_hud, update_hud};
use crate::desktop_systems::player_input;
use bevy::prelude::*;
use rand::RngCore;
use spooky_core::events::npc::NpcCollisionEvent;
use spooky_core::events::player::PlayerInputEvent;
use spooky_core::events::walker::WalkerCollisionEvent;
use spooky_core::events::{coin::CoinCollisionEvent, dynamite::DynamiteCollisionEvent};
use spooky_core::resources::MazeSeed;
use spooky_core::systems::hud::HudState;
use spooky_core::{systems, systems::collisions};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,))
        .insert_resource(MazeSeed(Some({
            let mut seed = [0u8; 32];
            rand::rng().fill_bytes(seed.as_mut());
            seed
        })))
        .add_systems(Startup, (systems::setup::setup, setup_hud))
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        .add_event::<PlayerInputEvent>()
        .add_event::<CoinCollisionEvent>()
        .add_event::<DynamiteCollisionEvent>()
        .add_event::<WalkerCollisionEvent>()
        .add_event::<NpcCollisionEvent>()
        .insert_resource(HudState::default())
        .add_systems(
            FixedUpdate,
            (
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
                player_input::dispatch_keyboard_input,
            ),
        )
        .add_systems(Update, (update_hud,))
        .run();
}
