use bevy::prelude::*;

mod components;
mod maze;
mod resources;
mod systems;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
    ))
    .add_systems(Startup, systems::setup::setup)
    // .add_systems(Update, systems::player_input::handle_input)
    .add_systems(Update, systems::game_logic::update_game)
    .run();
}
