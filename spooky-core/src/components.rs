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