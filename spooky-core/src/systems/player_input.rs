use bevy::prelude::*;
use bevy_input::prelude::*;
use bevy_transform::prelude::*;
use log::info;
use crate::resources::{MazeResource, PlayerPosition};
use crate::components::Player;

pub fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut player_pos: ResMut<PlayerPosition>,
    maze_res: Res<MazeResource>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let step: f32 = 16.0;
    let mut moved = false;
    // Compute candidate position starting from current player_pos.
    let mut candidate_x = player_pos.x;
    let mut candidate_y = player_pos.y;

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        candidate_y += step;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        candidate_y -= step;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        candidate_x -= step;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        candidate_x += step;
        moved = true;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }

    if moved {
        // Check collision with wall or boundary
        if maze_res.maze.check_wall_collision(candidate_x as i32, candidate_y as i32) {
            info!("Collision detected at candidate position: ({}, {})", candidate_x, candidate_y);
        } else {
            // No collision: update player position.
            player_pos.x = candidate_x;
            player_pos.y = candidate_y;
            // Update the player's transform.
            if let Ok(mut transform) = player_query.get_single_mut() {
                transform.translation.x = player_pos.x;
                transform.translation.y = player_pos.y;
            }
            // Update the camera's transform to follow the player.
            for mut transform in camera_query.iter_mut() {
                transform.translation.x = player_pos.x;
                transform.translation.y = player_pos.y;
            }
            info!("New player position: ({}, {})", player_pos.x, player_pos.y);
        }
    }
}
