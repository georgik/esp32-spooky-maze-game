use bevy::prelude::Event;

/// An event indicating that the player has collided with a dynamite.
/// The coordinates refer to the tile where the collision occurred.
#[derive(Debug, Event)]
pub struct DynamiteCollisionEvent {
    pub x: i32,
    pub y: i32,
}
