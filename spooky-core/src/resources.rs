use bevy::prelude::*;

use crate::maze::Maze;

/// Wraps the maze so that it can be used as a Bevy resource.
#[derive(Resource)]
pub struct MazeResource {
    pub maze: Maze,
}

#[derive(Resource)]
pub struct MazeSeed(pub Option<[u8; 32]>);

impl Default for MazeSeed {
    fn default() -> Self {
        // In the default case, you can leave it as None (or use a fixed seed).
        MazeSeed(None)
    }
}

#[derive(Resource, Debug)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
