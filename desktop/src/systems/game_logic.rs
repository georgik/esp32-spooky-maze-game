use bevy::prelude::*;
use crate::resources::MazeResource;

/// Updates the maze’s dynamic state—for example, moving NPCs.
/// A local timer controls how frequently the maze state is updated.
pub fn update_game(
    time: Res<Time>,
    mut maze_resource: ResMut<MazeResource>,
    mut update_timer: Local<Timer>,
) {
    // Initialize the timer on the first run to tick every 0.5 seconds.
    if update_timer.elapsed_secs() == 0.0 {
        *update_timer = Timer::from_seconds(0.5, TimerMode::Repeating);
    }
    update_timer.tick(time.delta());

    if update_timer.just_finished() {
        // Update dynamic elements of the maze:
        // For example, move NPCs (and walkers can be updated inside move_npcs too).
        maze_resource.maze.move_npcs();

        // Future additions:
        // - Check for coin collection collisions and remove coins.
        // - Trigger walker timers to allow player to walk through walls.
        // - Relocate walkers or dynamites if needed.
        info!("Maze updated: NPCs have moved.");
    }
}
