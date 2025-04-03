// spooky_core/src/systems/setup.rs

// Common Bevy imports.
use crate::components::{CoinComponent, NpcComponent, Player};
use crate::maze::Maze;
use crate::resources::{MazeResource, PlayerPosition};
use bevy::prelude::*;
use bevy_math::Vec3;
use bevy_transform::prelude::{GlobalTransform, Transform};

// When compiling for desktop (std enabled), use Bevy's AssetServer and its Image type.
#[cfg(feature = "std")]
use bevy::image::Image;
#[cfg(feature = "std")]
use bevy::prelude::*;

// --- TextureAssets for asset loading ---
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
    pub npc: Handle<Image>,
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
            npc: asset_server.load("textures/npc.png"),
        }
    }
}

// Embedded (no_std) mode: load embedded BMP images via tinybmp.
#[cfg(not(feature = "std"))]
use bevy::prelude::Resource;
#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;
#[cfg(not(feature = "std"))]
use tinybmp::Bmp;

#[cfg(not(feature = "std"))]
#[derive(Resource)]
pub struct TextureAssets {
    pub wall: Option<Bmp<'static, Rgb565>>,
    pub ground: Option<Bmp<'static, Rgb565>>,
    pub empty: Option<Bmp<'static, Rgb565>>,
    pub scorched: Option<Bmp<'static, Rgb565>>,
    pub ghost: Option<Bmp<'static, Rgb565>>,
    pub coin: Option<Bmp<'static, Rgb565>>,
    pub walker: Option<Bmp<'static, Rgb565>>,
    pub dynamite: Option<Bmp<'static, Rgb565>>,
}

#[cfg(not(feature = "std"))]
impl TextureAssets {
    pub fn load() -> Self {
        Self {
            wall: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/wall.bmp")).unwrap(),
            ),
            ground: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/ground.bmp"))
                    .unwrap(),
            ),
            empty: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/empty.bmp")).unwrap(),
            ),
            scorched: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/scorched.bmp"))
                    .unwrap(),
            ),
            ghost: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/ghost1.bmp"))
                    .unwrap(),
            ),
            coin: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/coin.bmp")).unwrap(),
            ),
            walker: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/walker.bmp"))
                    .unwrap(),
            ),
            dynamite: Some(
                Bmp::<Rgb565>::from_slice(include_bytes!("../../../assets/img/dynamite.bmp"))
                    .unwrap(),
            ),
        }
    }
}

#[cfg(not(feature = "std"))]
#[derive(Clone, Copy, Debug)]
pub enum TextureId {
    Ghost,
    Coin,
    Walker,
    Dynamite,
    // Add more as needed…
}

#[cfg(not(feature = "std"))]
#[derive(Component)]
pub struct NoStdSprite {
    pub texture: TextureId,
}
// --- Main Setup Function ---
#[cfg(not(feature = "std"))]
// use bevy_transform::prelude::Transform;
#[cfg(not(feature = "std"))]
#[derive(Component)]
pub struct NoStdTransform(pub Transform);

// --- Main Setup Function ---
pub fn setup(mut commands: Commands, #[cfg(feature = "std")] asset_server: Res<AssetServer>) {
    // Load textures conditionally.
    #[cfg(feature = "std")]
    let textures = TextureAssets::load(&asset_server);
    #[cfg(not(feature = "std"))]
    commands.insert_resource(TextureAssets::load());

    // Create the maze.
    let mut maze = Maze::new(64, 64, None);
    maze.generate_maze(32,32);
    maze.generate_coins();
    maze.generate_walkers();
    maze.generate_dynamites();
    maze.generate_npcs();

    // Compute playable bounds.
    let (left, bottom, _right, _top) = maze.playable_bounds();
    let initial_x = left as f32 + 11.0 * 16.0;
    let initial_y = bottom as f32 + 10.0 * 16.0;
    let player_start = Vec3::new(initial_x, initial_y, 2.0);

    // Insert the initial player position resource.
    commands.insert_resource(PlayerPosition {
        x: initial_x,
        y: initial_y,
        z: 10.0,
    });

    // Store the maze as a resource.
    let maze_for_entities = maze.clone();
    commands.insert_resource(MazeResource { maze });

    // Spawn the player (ghost).
    #[cfg(feature = "std")]
    {
        commands.spawn((
            Sprite::from_image(textures.ghost.clone()),
            Transform::from_translation(player_start),
            Player,
        ));
    }
    #[cfg(not(feature = "std"))]
    {
        commands.spawn((
            NoStdTransform(Transform::from_translation(player_start)),
            NoStdSprite {
                texture: TextureId::Ghost,
            },
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
                    Transform::from_translation(Vec3::new(coin.x as f32, coin.y as f32, 2.0)),
                    // (Assuming you have a CoinComponent for collision detection)
                    crate::components::CoinComponent {
                        x: coin.x,
                        y: coin.y,
                    },
                ));
            }
            #[cfg(not(feature = "std"))]
            {
                commands.spawn_empty();
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
                    Transform::from_translation(Vec3::new(walker.x as f32, walker.y as f32, 3.0)),
                    crate::components::WalkerComponent {
                        x: walker.x,
                        y: walker.y,
                    },
                ));
            }
            #[cfg(not(feature = "std"))]
            {
                // (No action or custom handling)
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
                    Transform::from_translation(Vec3::new(
                        dynamite.x as f32,
                        dynamite.y as f32,
                        4.0,
                    )),
                    crate::components::DynamiteComponent {
                        x: dynamite.x,
                        y: dynamite.y,
                    },
                ));
            }
            #[cfg(not(feature = "std"))]
            {
                // (No action or custom handling)
            }
        }
    }

    // <-- NEW: Spawn NPCs.
    for (i, npc) in maze_for_entities.npcs.iter().enumerate() {
        if npc.x != -1 && npc.y != -1 {
            #[cfg(feature = "std")]
            {
                // Choose an appropriate z-coordinate (e.g., 5.0) so that NPCs are drawn in front of coins
                // but behind the player if that’s your design.
                commands.spawn((
                    Sprite::from_image(textures.npc.clone()),
                    Transform::from_translation(Vec3::new(npc.x as f32, npc.y as f32, 5.0)),
                    NpcComponent {
                        index: i,
                        x: npc.x,
                        y: npc.y,
                    },
                ));
            }
            #[cfg(not(feature = "std"))]
            {
                // (No action or custom handling for no_std)
            }
        }
    }

    // Spawn the full tile map (background) covering the maze.
    let margin: i32 = Maze::MARGIN;
    let total_width = maze_for_entities.width as i32 + 2 * margin;
    let total_height = maze_for_entities.height as i32 + 2 * margin;
    for ty in 0..total_height {
        for tx in 0..total_width {
            let mx = tx - margin;
            let my = ty - margin;
            #[cfg(feature = "std")]
            let texture = if mx >= 0
                && my >= 0
                && mx < maze_for_entities.width as i32
                && my < maze_for_entities.height as i32
            {
                // No flipping here – use top‑left as anchor.
                let index = (my * maze_for_entities.width as i32 + mx) as usize;
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
                let translation = Vec3::new(
                    tx as f32 * maze_for_entities.tile_width as f32,
                    ty as f32 * maze_for_entities.tile_height as f32,
                    0.0,
                );
                commands.spawn((
                    Sprite::from_image(texture),
                    Transform::from_translation(translation),
                ));
            }
        }
    }

    // Spawn the camera.
    #[cfg(feature = "std")]
    {
        commands.spawn((
            Camera2d::default(),
            Transform::from_translation(Vec3::new(initial_x, initial_y, 100.0)),
        ));
    }
}
