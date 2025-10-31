use crate::components::DynamiteComponent;
use crate::events::dynamite::DynamiteCollisionMessage;
use crate::maze::Coin;
use crate::resources::{MazeResource, PlayerPosition};
use bevy::prelude::*;

/// This system checks the player's current tile against the dynamite tile(s)
/// in the maze. If the player's tile matches a dynamite tile, it dispatches a
/// `DynamiteCollisionEvent`.
pub fn detect_dynamite_collision(
    player_pos: Res<PlayerPosition>,
    maze_res: Res<MazeResource>,
    mut event_writer: MessageWriter<DynamiteCollisionMessage>,
) {
    // We assume the player moves in tile increments.
    let player_tile_x = player_pos.x as i32;
    let player_tile_y = player_pos.y as i32;

    // Dynamites are stored in an array (e.g., [Coin; 1])
    for dynamite in maze_res.maze.dynamites.iter() {
        if dynamite.x == player_tile_x && dynamite.y == player_tile_y {
            event_writer.write(DynamiteCollisionMessage {
                x: dynamite.x,
                y: dynamite.y,
            });
        }
    }
}

/// This system listens for `DynamiteCollisionEvent` events and handles them by
/// relocating the dynamite in the maze (so that the player can pick up another one)
/// and updating the associated entity's component so the visual position is corrected.
pub fn handle_dynamite_collision(
    mut events: MessageReader<DynamiteCollisionMessage>,
    mut maze_res: ResMut<MazeResource>,
    mut query: Query<&mut DynamiteComponent>,
) {
    for event in events.read() {
        // Relocate the dynamite in the maze.
        maze_res.maze.relocate_dynamite(Coin {
            x: event.x,
            y: event.y,
        });
        // Now update the dynamite entity: we assume the component stores its tile coordinates.
        for mut dyn_comp in query.iter_mut() {
            if dyn_comp.x == event.x && dyn_comp.y == event.y {
                // Since there is only one dynamite in our maze (array length 1), we take its new coordinates.
                let new_dyn = maze_res.maze.dynamites[0];
                dyn_comp.x = new_dyn.x;
                dyn_comp.y = new_dyn.y;
            }
        }
    }
}
