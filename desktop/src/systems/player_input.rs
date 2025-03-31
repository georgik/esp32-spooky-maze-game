use bevy::prelude::*;
use crate::resources::PlayerPosition;
use crate::components::Player;

pub fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut player_pos: ResMut<PlayerPosition>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let step: f32 = 16.0;
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

    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }


    if moved {
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
