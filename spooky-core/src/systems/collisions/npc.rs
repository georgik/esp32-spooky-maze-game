use crate::components::Player;
use crate::events::npc::NpcCollisionEvent;
use crate::maze::Npc;
use crate::resources::{MazeResource, PlayerPosition};
use crate::transform::SpookyTransform;
use bevy::prelude::*; // Our unified transform alias

/// This system checks the player's current tile against all NPC positions in the maze.
/// If the player is on the same tile as an NPC, it dispatches a `NPCCollisionEvent`.
pub fn detect_npc_collision(
    player_pos: Res<PlayerPosition>,
    maze_res: Res<MazeResource>,
    mut event_writer: EventWriter<NpcCollisionEvent>,
) {
    let player_tile_x = player_pos.x as i32;
    let player_tile_y = player_pos.y as i32;

    for npc in maze_res.maze.npcs.iter() {
        if npc.x == player_tile_x && npc.y == player_tile_y {
            event_writer.send(NpcCollisionEvent {
                npc_x: npc.x,
                npc_y: npc.y,
            });
        }
    }
}

/// This system handles `NPCCollisionEvent`s by relocating the player
/// to a new valid position. (NPC positions remain unchanged.)
pub fn handle_npc_collision(
    mut events: EventReader<NpcCollisionEvent>,
    mut player_pos: ResMut<PlayerPosition>,
    mut maze_res: ResMut<MazeResource>,
    mut player_query: Query<&mut SpookyTransform, With<Player>>,
) {
    for _event in events.read() {
        // When collision happens, relocate the player.
        let (new_x, new_y) = maze_res.maze.get_random_coordinates();
        player_pos.x = new_x as f32;
        player_pos.y = new_y as f32;

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
    }
}
