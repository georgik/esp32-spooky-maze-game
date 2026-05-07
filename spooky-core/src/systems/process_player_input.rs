#[cfg(not(feature = "std"))]
use crate::components::MainCamera;
use crate::components::Player;
use crate::events::player::PlayerInputMessage;
use crate::resources::{MazeResource, PlayerPosition};
use bevy::prelude::*;

// Use our unified transform type alias.
use crate::transform::UnifiedTransform;
use log::info;

/// Process player input events: update the logical player position and adjust
/// both the player's and camera's transform so that the player remains centered.
/// Movement is only applied if the new coordinates do not collide with a wall.
pub fn process_player_input(
    mut input_events: MessageReader<PlayerInputMessage>,
    mut player_pos: ResMut<PlayerPosition>,
    maze_res: Res<MazeResource>,
    mut player_query: Query<&mut UnifiedTransform, With<Player>>,
    #[cfg(feature = "std")] mut camera_query: Query<
        &mut UnifiedTransform,
        (With<Camera2d>, Without<Player>),
    >,
    #[cfg(not(feature = "std"))] mut camera_query: Query<
        &mut UnifiedTransform,
        (With<MainCamera>, Without<Player>),
    >,
) {
    for event in input_events.read() {
        // Check each axis independently - allow movement in clear directions
        let mut new_x = player_pos.x;
        let mut new_y = player_pos.y;

        // Try X movement
        if event.dx != 0.0 {
            let candidate_x = player_pos.x + event.dx;
            if !maze_res
                .maze
                .check_wall_collision(candidate_x as i32, player_pos.y as i32)
            {
                new_x = candidate_x;
            }
        }

        // Try Y movement
        if event.dy != 0.0 {
            let candidate_y = player_pos.y + event.dy;
            if !maze_res
                .maze
                .check_wall_collision(player_pos.x as i32, candidate_y as i32)
            {
                new_y = candidate_y;
            }
        }

        // Skip if no movement possible
        if (new_x - player_pos.x).abs() < f32::EPSILON && (new_y - player_pos.y).abs() < f32::EPSILON {
            continue;
        }

        // Update the logical player position.
        player_pos.x = new_x;
        player_pos.y = new_y;

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
