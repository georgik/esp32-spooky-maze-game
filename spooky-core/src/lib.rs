#![cfg_attr(not(feature = "std"), no_std)]

// Expose modules for the core logic.
pub mod maze;
pub mod components;
pub mod resources;
pub mod systems;
