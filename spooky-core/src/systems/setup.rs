// spooky_core/src/systems/setup.rs

// Import common parts from Bevy.
use bevy::prelude::*;
use crate::maze::Maze;
use crate::resources::{MazeResource, PlayerPosition};
use crate::components::Player;

//
// Asset loading types
//

// When compiling for desktop (std enabled), use Bevy's AssetServer and bevy::image::Image.
#[cfg(feature = "std")]
use bevy::image::Image;

#[cfg(feature = "std")]
pub struct TextureAssets {
    pub wall: Handle<Image>,
    pub ground: Handle<Image>,
    pub empty: Handle<Image>,
    pub scorched: Handle<Image>,
    pub ghost: Handle<Image>,
    pub coin: Handle<Image>,
    pub walker: Handle<Image>,
    pub dynamite: Handle<Image>,
}

#[cfg(feature = "std")]
impl TextureAssets {
    pub fn load(asset_server: &Res<AssetServer>) -> Self {
        Self {
            wall: asset_server.load("textures/wall.png"),
            ground: asset_server.load("textures/ground.png"),
            empty: asset_server.load("textures/empty.png"),
            scorched: asset_server.load("textures/scorched.png"),
            ghost: asset_server.load("textures/ghost.png"),
            coin: asset_server.load("textures/coin.png"),
            walker: asset_server.load("textures/walker.png"),
            dynamite: asset_server.load("textures/dynamite.png"),
        }
    }
}

// For no_std builds, use tinybmp to load embedded BMP images.
// (Adjust the file paths as needed.)
#[cfg(not(feature = "std"))]
use tinybmp::Bmp;
#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;

#[cfg(not(feature = "std"))]
pub struct TextureAssets<'a> {
    pub wall: Option<Bmp<'a, Rgb565>>,
    pub ground: Option<Bmp<'a, Rgb565>>,
    pub empty: Option<Bmp<'a, Rgb565>>,
    pub scorched: Option<Bmp<'a, Rgb565>>,
    pub ghost: Option<Bmp<'a, Rgb565>>,
    pub coin: Option<Bmp<'a, Rgb565>>,
    pub walker: Option<Bmp<'a, Rgb565>>,
    pub dynamite: Option<Bmp<'a, Rgb565>>,
}

#[cfg(not(feature = "std"))]
impl<'a> TextureAssets<'a> {
    pub fn load() -> Self {
        Self {
            wall: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/wall.bmp")).unwrap()),
            ground: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/ground.bmp")).unwrap()),
            empty: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/empty.bmp")).unwrap()),
            scorched: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/scorched.bmp")).unwrap()),
            ghost: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/ghost1.bmp")).unwrap()),
            coin: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/coin.bmp")).unwrap()),
            walker: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/walker.bmp")).unwrap()),
            dynamite: Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/dynamite.bmp")).unwrap()),
        }
    }
}

//
// The main setup function
//
pub fn setup(mut commands: Commands, #[cfg(feature = "std")] asset_server: Res<AssetServer>) {
    // Load textures conditionally.
    #[cfg(feature = "std")]
    let textures = TextureAssets::load(&asset_server);

    #[cfg(not(feature = "std"))]
    let textures = TextureAssets::load();

    // Create the maze.
    let mut maze = Maze::new(64, 64, None);
    maze.generate_coins();
    maze.generate_walkers();
    maze.generate_dynamites();
    maze.generate_npcs();

    // Compute playable bounds.
    let (left, bottom, _right, _top) = maze.playable_bounds();
    let initial_x = left as f32;
    let initial_y = bottom as f32;
    let player_start = Vec3::new(initial_x, initial_y, 2.0);

    // Insert initial player position resource.
    commands.insert_resource(PlayerPosition { x: initial_x, y: initial_y });

    // Clone maze for spawning entities, then insert original into a resource.
    let maze_for_entities = maze.clone();
    commands.insert_resource(MazeResource { maze });

    // Spawn the player (ghost). In std mode we can use Sprite with loaded textures.
    #[cfg(feature = "std")]
    {
        commands.spawn((
            Sprite::from_image(textures.ghost),
            Transform::from_translation(player_start),
            GlobalTransform::default(),
            Player,
        ));
    }
    // In no_std mode, you may need to spawn your player using your own drawing logic.
    #[cfg(not(feature = "std"))]
    {
        // For example, simply spawn the player with its transform and marker.
        commands.spawn((
            Transform::from_translation(player_start),
            GlobalTransform::default(),
            Player,
        ));
    }

    // Spawn coins.
    for coin in &maze_for_entities.coins {
        if coin.x != -1 && coin.y != -1 {
            #[cfg(feature = "std")]
            {
                commands.spawn((
                    Sprite::from_image(textures.coin.clone()),
                    Transform {
                        translation: Vec3::new(coin.x as f32, coin.y as f32, 1.0),
                        ..Default::default()
                    },
                    GlobalTransform::default(),
                ));
            }
            #[cfg(not(feature = "std"))]
            {
                commands.spawn((
                    Transform {
                        translation: Vec3::new(coin.x as f32, coin.y as f32, 1.0),
                        ..Default::default()
                    },
                    GlobalTransform::default(),
                    // You might add a marker component for coin entities.
                ));
            }
        }
    }

    // Spawn walkers.
    for walker in &maze_for_entities.walkers {
        if walker.x != -1 && walker.y != -1 {
            #[cfg(feature = "std")]
            {
                commands.spawn((
                    Sprite::from_image(textures.walker.clone()),
                    Transform {
                        translation: Vec3::new(walker.x as f32, walker.y as f32, 1.0),
                        ..Default::default()
                    },
                    GlobalTransform::default(),
                ));
            }
            #[cfg(not(feature = "std"))]
            {
                commands.spawn((
                    Transform {
                        translation: Vec3::new(walker.x as f32, walker.y as f32, 1.0),
                        ..Default::default()
                    },
                    GlobalTransform::default(),
                ));
            }
        }
    }

    // Spawn dynamites.
    for dynamite in &maze_for_entities.dynamites {
        if dynamite.x != -1 && dynamite.y != -1 {
            #[cfg(feature = "std")]
            {
                commands.spawn((
                    Sprite::from_image(textures.dynamite.clone()),
                    Transform {
                        translation: Vec3::new(dynamite.x as f32, dynamite.y as f32, 1.0),
                        ..Default::default()
                    },
                    GlobalTransform::default(),
                ));
            }
            #[cfg(not(feature = "std"))]
            {
                commands.spawn((
                    Transform {
                        translation: Vec3::new(dynamite.x as f32, dynamite.y as f32, 1.0),
                        ..Default::default()
                    },
                    GlobalTransform::default(),
                ));
            }
        }
    }

    // Spawn the full tile map (background) covering the maze plus margin.
    let margin: i32 = Maze::MARGIN;
    let total_width = maze_for_entities.width as i32 + 2 * margin;
    let total_height = maze_for_entities.height as i32 + 2 * margin;
    for ty in 0..total_height {
        for tx in 0..total_width {
            let mx = tx - margin;
            let my = ty - margin;
            #[cfg(feature = "std")]
            let texture = if mx >= 0 && my >= 0 &&
                mx < maze_for_entities.width as i32 && my < maze_for_entities.height as i32
            {
                // Flip the row since maze data row 0 is at the top.
                let maze_row = (maze_for_entities.height as i32 - 1) - my;
                let index = (maze_row * maze_for_entities.width as i32 + mx) as usize;
                match maze_for_entities.data[index] {
                    1 => textures.wall.clone(),
                    0 => textures.ground.clone(),
                    2 => textures.scorched.clone(),
                    _ => textures.ground.clone(),
                }
            } else {
                textures.empty.clone()
            };

            #[cfg(feature = "std")]
            {
                let translation = Vec3::new(tx as f32 * maze_for_entities.tile_width as f32, ty as f32 * maze_for_entities.tile_height as f32, 0.0);
                commands.spawn((
                    Sprite::from_image(texture),
                    Transform { translation, ..Default::default() },
                    GlobalTransform::default(),
                ));
            }
            // In no_std mode, you might store tile info in a separate resource for your render system.
        }
    }

    // Spawn the camera.
    commands.spawn((
        Camera2d::default(),
        Transform::from_translation(Vec3::new(initial_x, initial_y, 100.0)),
        GlobalTransform::default(),
    ));
}
