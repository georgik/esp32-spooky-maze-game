use bevy::prelude::Event;

/// This event is fired when the player collides with a coin.
/// The event carries the coin's pixel coordinates.
#[derive(Debug, Event)]
pub struct CoinCollisionEvent {
    pub coin_x: i32,
    pub coin_y: i32,
}
