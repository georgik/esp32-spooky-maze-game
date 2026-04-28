use bevy::prelude::Event;

/// An event indicating that the player collided with a walker.
/// The collision is reported in tile‐coordinates.
#[derive(Debug, Event)]
pub struct WalkerCollisionMessage {
    pub walker_x: i32,
    pub walker_y: i32,
}
