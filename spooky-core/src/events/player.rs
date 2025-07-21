use bevy::prelude::Event;
/// An event carrying a player movement request.
/// A positive dx moves the player right; a positive dy moves up.
/// In our game the movement step is one tile.
#[derive(Debug, Event)]
pub struct PlayerInputEvent {
    pub dx: f32,
    pub dy: f32,
}

/// An event triggered when the player requests teleportation.
#[derive(Debug, Event)]
pub struct PlayerTeleportEvent;
