use crate::components::WalkerComponent;
use crate::events::walker::WalkerCollisionEvent;
use crate::maze::Coin;
use crate::resources::{MazeResource, PlayerPosition};
use bevy::prelude::*; // Assumes you have a WalkerComponent

/// This system checks the player's current tile against all walker tiles in the maze.
/// When a collision is detected, a `WalkerCollisionEvent` is sent.
pub fn detect_walker_collision(
    player_pos: Res<PlayerPosition>,
    maze_res: Res<MazeResource>,
    mut event_writer: EventWriter<WalkerCollisionEvent>,
) {
    // Assume the player moves in tile increments.
    let player_tile_x = player_pos.x as i32;
    let player_tile_y = player_pos.y as i32;

    // Iterate over all walker positions stored in the maze.
    for walker in maze_res.maze.walkers.iter() {
        if walker.x == player_tile_x && walker.y == player_tile_y {
            event_writer.send(WalkerCollisionEvent {
                walker_x: walker.x,
                walker_y: walker.y,
            });
        }
    }
}

/// This system handles `WalkerCollisionEvent`s by relocating the walker in the maze
/// (so that the player can collect it again later) and updating the visual component.
pub fn handle_walker_collision(
    mut events: EventReader<WalkerCollisionEvent>,
    mut maze_res: ResMut<MazeResource>,
    mut query: Query<&mut WalkerComponent>,
) {
    for event in events.read() {
        // Get a new random coordinate for the walker.
        let (new_x, new_y) = maze_res.maze.get_random_coordinates();
        // Update the maze's walker array.
        for walker in maze_res.maze.walkers.iter_mut() {
            if walker.x == event.walker_x && walker.y == event.walker_y {
                walker.x = new_x;
                walker.y = new_y;
            }
        }
        // Update the corresponding entity (walker component) so its visual position is updated.
        for mut walker_comp in query.iter_mut() {
            if walker_comp.x == event.walker_x && walker_comp.y == event.walker_y {
                walker_comp.x = new_x;
                walker_comp.y = new_y;
            }
        }
    }
}
