use bevy::prelude::Message;

/// A message indicating that the player collided with a walker.
/// The collision is reported in tile‐coordinates.
#[derive(Debug, Message)]
pub struct WalkerCollisionMessage {
    pub walker_x: i32,
    pub walker_y: i32,
}
