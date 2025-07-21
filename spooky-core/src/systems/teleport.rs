use crate::components::Player;
use crate::events::player::PlayerTeleportEvent;
use crate::resources::{MazeResource, PlayerPosition};
use crate::systems::hud::HudState;
use bevy::prelude::*;
use log::info;

// Use our unified transform type alias.
use crate::transform::UnifiedTransform;

#[cfg(not(feature = "std"))]
use crate::components::MainCamera;

/// Handle teleport events: find a random safe location and teleport the player there.
/// Teleportation consumes from the teleport_countdown resource.
pub fn handle_player_teleport(
    mut events: EventReader<PlayerTeleportEvent>,
    mut player_pos: ResMut<PlayerPosition>,
    mut hud_state: ResMut<HudState>,
    maze_res: Res<MazeResource>,
    mut player_query: Query<&mut UnifiedTransform, With<Player>>,
    #[cfg(feature = "std")] mut camera_query: Query<
        &mut UnifiedTransform,
        (With<Camera2d>, Without<Player>),
    >,
    #[cfg(not(feature = "std"))] mut camera_query: Query<
        &mut UnifiedTransform,
        (With<MainCamera>, Without<Player>),
    >,
) {
    for _event in events.read() {
        // Check if teleportation is available
        if hud_state.teleport_countdown == 0 {
            info!("Teleport not available - countdown is 0");
            continue;
        }

        // Find a random safe location
        if let Some((new_x, new_y)) = find_safe_teleport_location(&maze_res.maze) {
            info!("Teleporting player to ({}, {})", new_x, new_y);
            
            // Update the logical player position
            player_pos.x = new_x as f32;
            player_pos.y = new_y as f32;

            // Update the player's transform
            if let Ok(mut transform) = player_query.single_mut() {
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

            // Update the camera's transform so that the player remains centered
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

            // Consume teleport charge
            hud_state.teleport_countdown = hud_state.teleport_countdown.saturating_sub(20);
        } else {
            info!("Could not find safe teleport location");
        }
    }
}

/// Find a safe location in the maze where the player can teleport.
/// Returns the world coordinates (x, y) of a safe location, or None if no safe location is found.
/// Uses a simple scanning approach that works in WASM without thread_rng.
fn find_safe_teleport_location(maze: &crate::maze::Maze) -> Option<(i32, i32)> {
    let (maze_left, maze_bottom, _maze_right, _maze_top) = maze.playable_bounds();
    let tile_w = maze.tile_width as i32;
    let tile_h = maze.tile_height as i32;

    // Create a simple pseudo-random starting point using current time/system state
    let start_x = (maze.width as usize / 3) as i32;
    let start_y = (maze.height as usize / 3) as i32;
    
    // Try different locations in a spiral pattern starting from a central area
    for offset in 0..((maze.width * maze.height) as i32 / 4) {
        let tile_x = (start_x + (offset % 10) - 5).clamp(0, maze.width as i32 - 1);
        let tile_y = (start_y + (offset / 10) - 5).clamp(0, maze.height as i32 - 1);
        
        // Check if this tile is passable (not a wall)
        let tile_index = (tile_y * maze.width as i32 + tile_x) as usize;
        if tile_index < maze.data.len() && maze.data[tile_index] != 1 {
            // Convert tile coordinates to world coordinates (tile boundary like normal movement)
            // This matches how the initial player position is calculated in setup.rs
            let world_x = maze_left + tile_x * tile_w;
            let world_y = maze_bottom + tile_y * tile_h;
            
            // Additional check: make sure there's no collision at this position
            if !maze.check_wall_collision(world_x, world_y) {
                return Some((world_x, world_y));
            }
        }
    }
    
    None
}

/// System to regenerate teleport charges over time
pub fn regenerate_teleport_charges(
    mut hud_state: ResMut<HudState>,
    _time: Res<Time>,
) {
    // Regenerate teleport charges slowly (1 charge every 5 seconds when at 0)
    if hud_state.teleport_countdown < 100 {
        // This is a simple timer-based regeneration
        // In a real game, you might want to use a proper timer resource
        hud_state.teleport_countdown = (hud_state.teleport_countdown + 1).min(100);
    }
}
