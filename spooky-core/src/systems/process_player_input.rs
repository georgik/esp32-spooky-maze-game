use bevy::prelude::*;
use crate::components::{Player, MainCamera};
use crate::resources::{PlayerPosition, MazeResource};
use crate::events::player::PlayerInputEvent;

// Use our unified transform type alias.
use crate::transform::SpookyTransform;

#[cfg(feature = "std")]
type SpTransform = SpookyTransform;
#[cfg(not(feature = "std"))]
type SpTransform = SpookyTransform;

/// Process player input events: update the logical player position and adjust
/// both the player's and camera's transform so that the player remains centered.
/// Movement is only applied if the new coordinates do not collide with a wall.
pub fn process_player_input(
    mut events: EventReader<PlayerInputEvent>,
    mut player_pos: ResMut<PlayerPosition>,
    maze_res: Res<MazeResource>,
    mut player_query: Query<&mut SpookyTransform, With<Player>>,
    #[cfg(feature = "std")]
    mut camera_query: Query<&mut SpookyTransform, (With<Camera2d>, Without<Player>)>,
    #[cfg(not(feature = "std"))]
    mut camera_query: Query<&mut SpookyTransform, (With<MainCamera>, Without<Player>)>,
) {
    for event in events.read() {
        // Calculate candidate new position.
        let candidate_x = player_pos.x + event.dx;
        let candidate_y = player_pos.y + event.dy;

        // Check for wall collision.
        if maze_res.maze.check_wall_collision(candidate_x as i32, candidate_y as i32) {
            // Optionally log the collision, then skip updating.
            info!("Collision detected at ({}, {})", candidate_x, candidate_y);
            continue;
        }

        // No collision: update the logical player position.
        player_pos.x = candidate_x;
        player_pos.y = candidate_y;

        // Update the player's transform.
        if let Ok(mut transform) = player_query.single_mut() {
            #[cfg(feature = "std")]
            {
                transform.translation.x = player_pos.x;
                transform.translation.y = player_pos.y;
            }
            #[cfg(not(feature = "std"))]
            {
                transform.0.translation.x = player_pos.x;
                transform.0.translation.y = player_pos.y;
            }
        }

        // Update the camera's transform so that the player remains centered.
        for mut transform in camera_query.iter_mut() {
            #[cfg(feature = "std")]
            {
                transform.translation.x = player_pos.x;
                transform.translation.y = player_pos.y;
            }
            #[cfg(not(feature = "std"))]
            {
                transform.0.translation.x = player_pos.x;
                transform.0.translation.y = player_pos.y;
            }
        }
    }
}
