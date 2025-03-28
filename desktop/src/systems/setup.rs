use bevy::prelude::*;
use bevy::image::Image; // Import Image from bevy::image
use crate::maze::Maze;
use crate::resources::MazeResource;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load textures with explicit type annotations.
    let wall_texture: Handle<Image> = asset_server.load("textures/wall.png");
    let ghost_texture: Handle<Image> = asset_server.load("textures/ghost.png");
    let coin_texture: Handle<Image> = asset_server.load("textures/coin.png");
    let walker_texture: Handle<Image> = asset_server.load("textures/walker.png");
    let dynamite_texture: Handle<Image> = asset_server.load("textures/dynamite.png");

    // Create a Maze instance (for example, a 64 x 64 maze with no explicit seed).
    let mut maze = Maze::new(64, 64, None);

    // Generate positions for coins, walkers, dynamites and NPCs.
    maze.generate_coins();
    maze.generate_walkers();
    maze.generate_dynamites();
    maze.generate_npcs();
    // (Optionally, if dynamic maze generation is enabled, call maze.generate_maze(...))

    // Clone the arrays that we need for spawning entities
    let coins = maze.coins;
    let walkers = maze.walkers;
    let dynamites = maze.dynamites;

    // Now move maze into a resource.
    commands.insert_resource(MazeResource { maze });

    // Spawn the player's avatar (ghost) at the origin.
    commands.spawn((
        Sprite::from_image(ghost_texture),
        Transform::from_translation(Vec3::ZERO),
        GlobalTransform::default(),
        // Optionally, add a marker component, e.g. Player
    ));

    // Spawn coin entities at each valid coin position.
    for coin in &coins {
        if coin.x != -1 && coin.y != -1 {
            commands.spawn((
                Sprite::from_image(coin_texture.clone()),
                Transform {
                    translation: Vec3::new(coin.x as f32, coin.y as f32, 0.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
                // Optionally, add a Coin marker component.
            ));
        }
    }

    // Spawn walker entities.
    for walker in &walkers {
        if walker.x != -1 && walker.y != -1 {
            commands.spawn((
                Sprite::from_image(walker_texture.clone()),
                Transform {
                    translation: Vec3::new(walker.x as f32, walker.y as f32, 0.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
                // Optionally, add a Walker marker component.
            ));
        }
    }

    // Spawn dynamite entities.
    for dynamite in &dynamites {
        if dynamite.x != -1 && dynamite.y != -1 {
            commands.spawn((
                Sprite::from_image(dynamite_texture.clone()),
                Transform {
                    translation: Vec3::new(dynamite.x as f32, dynamite.y as f32, 0.0),
                    ..Default::default()
                },
                GlobalTransform::default(),
                // Optionally, add a Dynamite marker component.
            ));
        }
    }

    // Spawn the 2D camera.
    commands.spawn(Camera2d::default());
}
