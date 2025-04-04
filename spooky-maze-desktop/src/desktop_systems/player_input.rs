use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use spooky_core::events::player::PlayerInputEvent;

/// Reads keyboard input (arrow keys) and sends a PlayerInputEvent continuously
/// while keys are held down. A positive dx moves right; a positive dy moves up.
/// The step is defined as one tile.
pub fn dispatch_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<PlayerInputEvent>,
) {
    let mut dx = 0.0;
    let mut dy = 0.0;
    let step = 16.0; // adjust to your tile size

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        dy += step;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        dy -= step;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        dx += step;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        dx -= step;
    }

    if dx != 0.0 || dy != 0.0 {
        event_writer.send(PlayerInputEvent { dx, dy });
    }
}
