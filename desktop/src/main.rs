mod desktop_systems;

use bevy::prelude::*;
use spooky_core::systems;
use spooky_core::events::player_events::PlayerInputEvent;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
    ))
    .add_systems(Startup, systems::setup::setup)
    .add_event::<PlayerInputEvent>()
    .add_systems(Update, crate::desktop_systems::player_input::dispatch_keyboard_input)
    .add_systems(Update, systems::process_player_input::process_player_input)
    .add_systems(Update, systems::game_logic::update_game)
    .run();
}
