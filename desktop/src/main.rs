use bevy::prelude::*;
use spooky_core::systems;
use spooky_core::resources;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
    ))
    .add_systems(Startup, systems::setup::setup)
    .add_systems(Update, systems::player_input::handle_input)
    .add_systems(Update, systems::game_logic::update_game)
    .run();
}
