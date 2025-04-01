// spooky_core/src/systems/player_input.rs

use bevy::prelude::*;
use crate::components::{Player, MainCamera};
use crate::resources::PlayerPosition;

/// An event carrying a player movement request.
/// A positive dx moves the player right; a positive dy moves up.
/// In our game the movement step is one tile.
#[derive(Debug, Event)]
pub struct PlayerInputEvent {
    pub dx: f32,
    pub dy: f32,
}

// Use our unified transform type alias.
use crate::transform::SpookyTransform;

// Create a type alias so that in std builds SpookyTransform is just Transform,
// and in no_std builds it is our wrapper.
#[cfg(feature = "std")]
type SpTransform = SpookyTransform;

#[cfg(not(feature = "std"))]
type SpTransform = SpookyTransform;

/// Process player input events: update the logical player position and adjust
/// both the player's and camera's transform so that the player remains centered.
pub fn process_player_input(
    mut events: EventReader<PlayerInputEvent>,
    mut player_pos: ResMut<PlayerPosition>,
    mut player_query: Query<&mut SpookyTransform, With<Player>>,
    #[cfg(feature = "std")]
    mut camera_query: Query<&mut SpookyTransform, (With<Camera2d>, Without<Player>)>,
    #[cfg(not(feature = "std"))]
    mut camera_query: Query<&mut SpookyTransform, (With<MainCamera>, Without<Player>)>,
) {
    // Use `read()` to iterate over events.
    for event in events.read() {
        // Update the logical position.
        player_pos.x += event.dx;
        player_pos.y += event.dy;

        // Update the player's transform.
        if let Ok(mut transform) = player_query.get_single_mut() {
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
        // Update the camera transform so that the player stays centered.
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
