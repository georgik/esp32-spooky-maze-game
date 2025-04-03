use crate::components::CoinComponent;
use crate::events::coin::CoinCollisionEvent;
use crate::maze::Coin;
use crate::resources::{MazeResource, PlayerPosition};
use bevy::prelude::*;
use crate::systems::hud::HudState;

/// This system checks the player's current position against all coin positions in the maze.
/// If the player is on the same tile as a coin, it dispatches a `CoinCollisionEvent`.
pub fn detect_coin_collision(
    player_pos: Res<PlayerPosition>,
    maze_res: Res<MazeResource>,
    mut event_writer: EventWriter<CoinCollisionEvent>,
) {
    // Assuming the player moves in tile increments, cast the logical position to i32.
    let player_tile_x = player_pos.x as i32;
    let player_tile_y = player_pos.y as i32;

    for coin in maze_res.maze.coins.iter() {
        if coin.x == player_tile_x && coin.y == player_tile_y {
            event_writer.send(CoinCollisionEvent {
                coin_x: coin.x,
                coin_y: coin.y,
            });
        }
    }
}

/// This system listens for `CoinCollisionEvent` and removes the collided coin from the maze.
pub fn remove_coin_on_collision(
    mut events: EventReader<CoinCollisionEvent>,
    mut maze_res: ResMut<MazeResource>,
    mut hud_state: ResMut<HudState>,
    mut commands: Commands,
    query: Query<(Entity, &CoinComponent)>,
) {
    for event in events.read() {
        // Remove the coin at the collision coordinates.
        maze_res.maze.remove_coin(Coin {
            x: event.coin_x,
            y: event.coin_y,
        });

        hud_state.coins_left = maze_res.maze.coin_counter;

        // Despawn coin entity with matching coordinates.
        for (entity, coin_comp) in query.iter() {
            if coin_comp.x == event.coin_x && coin_comp.y == event.coin_y {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
