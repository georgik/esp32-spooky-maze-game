use crate::resources::MazeResource;
use bevy::prelude::*;

/// This system updates the positions of all NPCs by calling Maze::move_npcs.
pub fn update_npc_movement(mut maze_res: ResMut<MazeResource>) {
    maze_res.maze.move_npcs();
}
