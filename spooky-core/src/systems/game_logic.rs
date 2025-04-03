use crate::resources::MazeResource;
use bevy::prelude::*;

pub fn update_game(
    mut maze_resource: ResMut<MazeResource>,
) {
    maze_resource.maze.move_npcs();
}
