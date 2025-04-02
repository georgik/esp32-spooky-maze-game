#![cfg_attr(not(feature = "std"), no_std)]

// Expose modules for the core logic.
pub mod maze;
pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
mod transform;
mod camera;

#[cfg(feature = "static_maze")]
mod static_maze_data;
