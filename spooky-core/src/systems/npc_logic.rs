use crate::components::NpcComponent;
use crate::resources::MazeResource;
use crate::transform::UnifiedTransform;
use bevy::prelude::*;

/// This system updates the positions of all NPCs by calling Maze::move_npcs.
pub fn update_npc_movement(
    mut maze_res: ResMut<MazeResource>,
    mut query: Query<(&mut UnifiedTransform, &mut NpcComponent)>,
) {
    // Update positions in the Maze resource.
    maze_res.maze.move_npcs();

    // For each NPC entity, update its component and transform using its index.
    for (mut transform, mut npc_comp) in query.iter_mut() {
        // Use the stored index to look up the new position in the Maze.
        let updated_npc = maze_res.maze.npcs[npc_comp.index];
        npc_comp.x = updated_npc.x;
        npc_comp.y = updated_npc.y;

        #[cfg(feature = "std")]
        {
            transform.translation.x = npc_comp.x as f32;
            transform.translation.y = npc_comp.y as f32;
        }
        #[cfg(not(feature = "std"))]
        {
            transform.0.translation.x = npc_comp.x as f32;
            transform.0.translation.y = npc_comp.y as f32;
        }
    }
}
