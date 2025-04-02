use bevy::prelude::*;
use crate::maze::Coin;
use crate::events::dynamite::DynamiteCollisionEvent;
use crate::resources::MazeResource;

/// This system listens for `DynamiteCollisionEvent` and relocates the dynamite
/// instead of despawning it.
pub fn handle_dynamite_collision(
    mut events: EventReader<DynamiteCollisionEvent>,
    mut maze_res: ResMut<MazeResource>,
) {
    for event in events.read() {
        maze_res.maze.relocate_dynamite(Coin { x: event.x, y: event.y });
    }
}
