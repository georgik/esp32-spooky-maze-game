use bevy::prelude::*;

use crate::maze::Maze;

/// Wraps the maze so that it can be used as a Bevy resource.
#[derive(Resource)]
pub struct MazeResource {
    pub maze: Maze,
}

#[derive(Resource, Debug)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
}