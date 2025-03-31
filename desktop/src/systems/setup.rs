use bevy::prelude::*;
use bevy::image::Image;
use crate::maze::Maze;
use crate::resources::{MazeResource, PlayerPosition};
use crate::components::Player;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load textures.
    let wall_texture: Handle<Image> = asset_server.load("textures/wall.png");
    let ground_texture: Handle<Image> = asset_server.load("textures/ground.png");
    let empty_texture: Handle<Image> = asset_server.load("textures/empty.png");
    let scorched_texture: Handle<Image> = asset_server.load("textures/scorched.png");
    let ghost_texture: Handle<Image> = asset_server.load("textures/ghost.png");
    let coin_texture: Handle<Image> = asset_server.load("textures/coin.png");
    let walker_texture: Handle<Image> = asset_server.load("textures/walker.png");
    let dynamite_texture: Handle<Image> = asset_server.load("textures/dynamite.png");

    // Create the maze.
    let mut maze = Maze::new(64, 64, None);
    maze.generate_coins();
    maze.generate_walkers();
    maze.generate_dynamites();
    maze.generate_npcs();

    // Compute playable bounds.
    let (left, bottom, _right, _top) = maze.playable_bounds();
    let tile_width = maze.tile_width as f32;
    let tile_height = maze.tile_height as f32;
    // For reference, place the player at the lower-left playable tile's center.
    let initial_x = left as f32;
    let initial_y = bottom as f32;
    let player_start = Vec3::new(initial_x, initial_y, 2.0);

    // Insert initial player position resource.
    commands.insert_resource(PlayerPosition { x: initial_x, y: initial_y });

    // Clone maze for spawning entities, and insert original into resource.
    let maze_for_entities = maze.clone();
    commands.insert_resource(MazeResource { maze });

    // Spawn the player (ghost) with a marker component.
    commands.spawn((
        Sprite::from_image(ghost_texture),
        Transform::from_translation(player_start),
        GlobalTransform::default(),
        Player,
    ));

    // Spawn coins.
    for coin in &maze_for_entities.coins {
        if coin.x != -1 && coin.y != -1 {
            commands.spawn((
                Sprite::from_image(coin_texture.clone()),
                Transform {
                    translation: Vec3::new(coin.x as f32, coin.y as f32, 1.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
            ));
        }
    }

    // Spawn walkers.
    for walker in &maze_for_entities.walkers {
        if walker.x != -1 && walker.y != -1 {
            commands.spawn((
                Sprite::from_image(walker_texture.clone()),
                Transform {
                    translation: Vec3::new(walker.x as f32, walker.y as f32, 1.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
            ));
        }
    }

    // Spawn dynamites.
    for dynamite in &maze_for_entities.dynamites {
        if dynamite.x != -1 && dynamite.y != -1 {
            commands.spawn((
                Sprite::from_image(dynamite_texture.clone()),
                Transform {
                    translation: Vec3::new(dynamite.x as f32, dynamite.y as f32, 1.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
            ));
        }
    }

    // Spawn full tile map (background) covering maze plus margin.
    let margin: i32 = Maze::MARGIN;
    let total_width = maze_for_entities.width as i32 + 2 * margin;
    let total_height = maze_for_entities.height as i32 + 2 * margin;
    for ty in 0..total_height {
        for tx in 0..total_width {
            let mx = tx - margin;
            let my = ty - margin;
            // Only if the tile is inside the maze's dimensions.
            let texture: Handle<Image> = if mx >= 0 && my >= 0 &&
                mx < maze_for_entities.width as i32 && my < maze_for_entities.height as i32
            {
                // Because maze data row 0 is at the top, flip my:
                let maze_row = (maze_for_entities.height as i32 - 1) - my;
                let index = (maze_row * maze_for_entities.width as i32 + mx) as usize;
                match maze_for_entities.data[index] {
                    1 => wall_texture.clone(),
                    0 => ground_texture.clone(),
                    2 => scorched_texture.clone(),
                    _ => ground_texture.clone(),
                }
            } else {
                empty_texture.clone()
            };

            let translation = Vec3::new(tx as f32 * maze_for_entities.tile_width as f32, ty as f32 * maze_for_entities.tile_height as f32, 0.0);
            commands.spawn((
                Sprite::from_image(texture),
                Transform { translation, ..Default::default() },
                GlobalTransform::default(),
            ));
        }
    }

    // Spawn the camera.
    // Here we set the camera's initial transform to follow the player start.
    commands.spawn((
        Camera2d::default(),
        Transform::from_translation(Vec3::new(initial_x, initial_y, 100.0)),
        GlobalTransform::default(),
    ));
}
