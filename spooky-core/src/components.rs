use bevy::prelude::*;

/// Marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// Marker component for wall entities.
#[derive(Component)]
pub struct Wall;

/// Marker component for ghost entities.
#[derive(Component)]
pub struct Ghost;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CoinComponent {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct DynamiteComponent {
    pub x: i32,
    pub y: i32,
}



#[derive(Component, Debug)]
pub struct WalkerComponent {
    pub x: i32,
    pub y: i32,
}

