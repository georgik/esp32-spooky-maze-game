#![cfg_attr(not(feature = "std"), no_std)]

// Expose modules for the core logic.
pub mod components;
pub mod events;
pub mod maze;
pub mod resources;
pub mod systems;
mod transform;

pub mod sprite_buf;
#[cfg(feature = "static_maze")]
mod static_maze_data;
