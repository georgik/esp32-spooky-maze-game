#![cfg_attr(not(feature = "std"), no_std)]

// Expose modules for the core logic.
mod camera;
pub mod components;
pub mod events;
pub mod maze;
pub mod resources;
pub mod systems;
mod transform;

#[cfg(feature = "static_maze")]
mod static_maze_data;
