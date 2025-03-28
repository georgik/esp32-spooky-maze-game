use bevy::prelude::*;
use crate::resources::PlayerPosition;

pub fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_pos: ResMut<PlayerPosition>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let step: f32 = 16.0; // Adjust this step to match your maze tile size.
    let mut moved = false;

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        player_pos.y += step;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        player_pos.y -= step;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        player_pos.x -= step;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        player_pos.x += step;
        moved = true;
    }

    if moved {
        // Update the camera's transform so that it always centers on the player.
        for mut transform in camera_query.iter_mut() {
            transform.translation.x = player_pos.x;
            transform.translation.y = player_pos.y;
        }
        info!("New player position: ({}, {})", player_pos.x, player_pos.y);
    }
}
