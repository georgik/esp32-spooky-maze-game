mod desktop_systems;

use crate::desktop_systems::player_input;
use bevy::prelude::*;
use spooky_core::events::player::PlayerInputEvent;
use spooky_core::events::{coin::CoinCollisionEvent, dynamite::DynamiteCollisionEvent};
use spooky_core::{systems, systems::collisions};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,))
        .add_systems(Startup, systems::setup::setup)
        .add_event::<PlayerInputEvent>()
        .add_event::<CoinCollisionEvent>()
        .add_event::<DynamiteCollisionEvent>()
        .add_systems(Update, player_input::dispatch_keyboard_input)
        .add_systems(Update, systems::process_player_input::process_player_input)
        .add_systems(Update, collisions::coin::detect_coin_collision)
        .add_systems(Update, collisions::coin::remove_coin_on_collision)
        .add_systems(Update, collisions::dynamite::handle_dynamite_collision)
        .add_systems(Update, systems::dynamite_logic::handle_dynamite_collision)
        .add_systems(Update, systems::game_logic::update_game)
        .run();
}
