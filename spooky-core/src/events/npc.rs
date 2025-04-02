use bevy::prelude::*;

/// An event indicating that the player collided with an NPC.
/// The collision is reported using tile coordinates.
#[derive(Debug, Event)]
pub struct NpcCollisionEvent {
    pub npc_x: i32,
    pub npc_y: i32,
}
