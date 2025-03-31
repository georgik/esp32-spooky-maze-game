use bevy::prelude::*;
use bevy::image::Image; // Import Image from bevy::image
use crate::maze::Maze;
use crate::resources::{MazeResource, PlayerPosition};

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Insert initial player position.
    commands.insert_resource(PlayerPosition { x: 0.0, y: 0.0 });

    // Load textures with explicit type annotations.
    let wall_texture: Handle<Image> = asset_server.load("textures/wall.png");
    let ground_texture: Handle<Image> = asset_server.load("textures/ground.png");
    let empty_texture: Handle<Image> = asset_server.load("textures/empty.png");
    let scorched_texture: Handle<Image> = asset_server.load("textures/scorched.png");
    let ghost_texture: Handle<Image> = asset_server.load("textures/ghost.png");
    let coin_texture: Handle<Image> = asset_server.load("textures/coin.png");
    let walker_texture: Handle<Image> = asset_server.load("textures/walker.png");
    let dynamite_texture: Handle<Image> = asset_server.load("textures/dynamite.png");

    // Create a Maze instance (static mode: 64 x 64 tiles, no explicit seed).
    let mut maze = Maze::new(64, 64, None);
    maze.generate_coins();
    maze.generate_walkers();
    maze.generate_dynamites();
    maze.generate_npcs();
    // For static mode, maze.data is baked in.

    // Clone maze for spawning coins, walkers, and tile map,
    // then insert the original maze as a resource.
    let maze_for_entities = maze.clone();
    commands.insert_resource(MazeResource { maze });

    // Spawn the player's avatar (ghost) at the origin.
    // Set z = 2 so that it draws in front.
    commands.spawn((
        Sprite::from_image(ghost_texture),
        Transform {
            translation: Vec3::new(0.0, 0.0, 2.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        // (Optionally add a marker component like `Player`)
    ));

    // Spawn coin entities (z = 1 to appear above the background).
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

    // Spawn walker entities (z = 1).
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

    // Spawn dynamite entities (z = 1).
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

    // Spawn the full tile map covering the maze area plus extra margin.
    let margin: i32 = 10;
    let total_width = maze_for_entities.width as i32 + 2 * margin;
    let total_height = maze_for_entities.height as i32 + 2 * margin;
    let tile_width = maze_for_entities.tile_width as f32;
    let tile_height = maze_for_entities.tile_height as f32;

    // The maze tiles (background) are drawn at z = 0.
    for ty in 0..total_height {
        for tx in 0..total_width {
            let mx = tx - margin;
            let my = ty - margin;
            let texture: Handle<Image> = if mx >= 0 && my >= 0 &&
                mx < maze_for_entities.width as i32 && my < maze_for_entities.height as i32 {
                let index = (my * maze_for_entities.width as i32 + mx) as usize;
                match maze_for_entities.data[index] {
                    1 => wall_texture.clone(),     // Wall tile.
                    0 => ground_texture.clone(),   // Ground tile.
                    2 => scorched_texture.clone(), // Scorched tile.
                    _ => ground_texture.clone(),
                }
            } else {
                empty_texture.clone()
            };

            let translation = Vec3::new(tx as f32 * tile_width, ty as f32 * tile_height, 0.0);
            commands.spawn((
                Sprite::from_image(texture),
                Transform { translation, ..Default::default() },
                GlobalTransform::default(),
            ));
        }
    }

    // Spawn the 2D camera.
    commands.spawn(Camera2d::default());
}
