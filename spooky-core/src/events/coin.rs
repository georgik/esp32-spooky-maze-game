use bevy::prelude::Message;

/// This event is fired when the player collides with a coin.
/// The event carries the coin's pixel coordinates.
#[derive(Debug, Message)]
pub struct CoinCollisionMessage {
    pub coin_x: i32,
    pub coin_y: i32,
}
