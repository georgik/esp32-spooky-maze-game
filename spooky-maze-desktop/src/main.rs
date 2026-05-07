mod desktop_systems;

use crate::desktop_systems::hud::{setup_hud, update_hud};
use crate::desktop_systems::player_input;
use bevy::prelude::*;
use rand::RngCore;
use spooky_core::events::npc::NpcCollisionMessage;
use spooky_core::events::player::PlayerInputMessage;
use spooky_core::events::walker::WalkerCollisionMessage;
use spooky_core::events::{coin::CoinCollisionMessage, dynamite::DynamiteCollisionMessage};
use spooky_core::resources::MazeSeed;
use spooky_core::systems::hud::HudState;
use spooky_core::{systems, systems::collisions};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            // Use CARGO_MANIFEST_DIR to find assets relative to crate root
            file_path: env!("CARGO_MANIFEST_DIR").to_string() + "/assets",
            ..default()
        }),
    ))
        .insert_resource(MazeSeed(Some({
            let mut seed = [0u8; 32];
            rand::rng().fill_bytes(seed.as_mut());
            seed
        })))
        .add_systems(Startup, (systems::setup::setup, setup_hud))
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        .add_event::<PlayerInputMessage>()
        .add_event::<CoinCollisionMessage>()
        .add_event::<DynamiteCollisionMessage>()
        .add_event::<WalkerCollisionMessage>()
        .add_event::<NpcCollisionMessage>()
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
