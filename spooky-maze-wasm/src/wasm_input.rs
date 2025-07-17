use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use spooky_core::events::player::PlayerInputEvent;
use web_sys::console;

pub struct WasmInputPlugin;

impl Plugin for WasmInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dispatch_keyboard_input);
    }
}

fn dispatch_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_input_events: EventWriter<PlayerInputEvent>,
) {
    let mut dx = 0.0;
    let mut dy = 0.0;
    let step = 16.0; // adjust to your tile size
    
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        dy += step;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        dy -= step;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        dx -= step;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        dx += step;
    }
    
    if dx != 0.0 || dy != 0.0 {
        console::log_1(&format!("Input: dx={}, dy={}", dx, dy).into());
        player_input_events.write(PlayerInputEvent { dx, dy });
    }
    
    // Handle special actions
    if keyboard_input.just_pressed(KeyCode::Space) {
        console::log_1(&"Space pressed - teleport".into());
        // TODO: Send teleport event
    }
    
    if keyboard_input.just_pressed(KeyCode::Enter) {
        console::log_1(&"Enter pressed - place dynamite".into());
        // TODO: Send place dynamite event
    }
}
