mod desktop_systems;

use crate::desktop_systems::player_input;
use bevy::prelude::*;
use spooky_core::events::player::PlayerInputEvent;
use spooky_core::events::{coin::CoinCollisionEvent, dynamite::DynamiteCollisionEvent};
use spooky_core::{systems, systems::collisions};
use spooky_core::events::walker::WalkerCollisionEvent;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,))
        .add_systems(Startup, systems::setup::setup)
        .add_event::<PlayerInputEvent>()
        .add_event::<CoinCollisionEvent>()
        .add_event::<DynamiteCollisionEvent>()
        .add_event::<WalkerCollisionEvent>()
        .add_systems(
            Update,
            (
                player_input::dispatch_keyboard_input,
                systems::process_player_input::process_player_input,
                collisions::coin::detect_coin_collision,
                collisions::coin::remove_coin_on_collision,
                collisions::dynamite::handle_dynamite_collision,
                collisions::walker::detect_walker_collision,
                collisions::walker::handle_walker_collision,
                systems::dynamite_logic::handle_dynamite_collision,
                systems::game_logic::update_game,
            ),
        )
        .run();
}
