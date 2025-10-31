use bevy::prelude::Message;

/// An event indicating that the player has collided with a dynamite.
/// The coordinates refer to the tile where the collision occurred.
#[derive(Debug, Message)]
pub struct DynamiteCollisionMessage {
    pub x: i32,
    pub y: i32,
}
