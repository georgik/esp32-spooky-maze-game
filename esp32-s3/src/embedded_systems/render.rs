#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;

use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};

use bevy_ecs::prelude::*;
use spooky_core::resources::{MazeResource, PlayerPosition};

#[cfg(not(feature = "std"))]
use spooky_core::systems::setup::TextureAssets;

/// Render the maze tile map, coins, and the player ghost into the off‑screen framebuffer,
/// then flush it to the physical display.
///
/// The camera is simulated by calculating an offset so that the player (whose world
/// coordinates are in PlayerPosition) is always centered on the display.
/// The drawing order is:
/// 1. Maze tile map (background)
/// 2. Coins (drawn using the tile’s top‑left as anchor)
/// 3. The player ghost (drawn using the tile’s top‑left as anchor)
pub fn render_system(
    mut display_res: NonSendMut<crate::DisplayResource>,
    mut fb_res: ResMut<crate::FrameBufferResource>,
    maze_res: Res<MazeResource>,
    #[cfg(not(feature = "std"))] texture_assets: Res<TextureAssets>,
    #[cfg(not(feature = "std"))] player_pos: Res<PlayerPosition>,
) {
    // Clear the framebuffer.
    fb_res.frame_buf.clear(Rgb565::BLACK).unwrap();

    let maze = &maze_res.maze;
    let (left, bottom, _right, _top) = maze.playable_bounds();
    let tile_w = maze.tile_width as i32;
    let tile_h = maze.tile_height as i32;

    // Compute the camera offset so that the player is centered.
    // For example, if the display is 320x240 then the center is (160,120).
    let display_center_x = (crate::LCD_H_RES as i32) / 2;
    let display_center_y = (crate::LCD_V_RES as i32) / 2;
    let offset_x = player_pos.x as i32 - display_center_x;
    let offset_y = player_pos.y as i32 - display_center_y;

    // --- Render the maze tile map (background) ---
    for ty in 0..maze.height as i32 {
        for tx in 0..maze.width as i32 {
            let tile_index = (ty * maze.width as i32 + tx) as usize;
            let bmp_opt = match maze.data[tile_index] {
                1 => texture_assets.wall.as_ref(),
                0 => texture_assets.ground.as_ref(),
                2 => texture_assets.scorched.as_ref(),
                _ => texture_assets.ground.as_ref(),
            };

            if let Some(bmp) = bmp_opt {
                // Compute the tile's world coordinate (top‑left of the tile)
                let world_x = left + tx * tile_w;
                let world_y = bottom + ty * tile_h;
                // Compute the screen coordinate by subtracting the camera offset.
                let screen_x = world_x - offset_x;
                let screen_y = world_y - offset_y;
                let pos = Point::new(screen_x, screen_y);
                Image::new(bmp, pos).draw(&mut fb_res.frame_buf).unwrap();
            }
        }
    }

    // --- Render coins on top of the maze ---
    for coin in &maze.coins {
        if coin.x != -1 && coin.y != -1 {
            if let Some(bmp) = texture_assets.coin.as_ref() {
                // Use the coin's coordinate as the top‑left corner.
                let screen_x = coin.x - offset_x;
                let screen_y = coin.y - offset_y;
                let pos = Point::new(screen_x, screen_y);
                Image::new(bmp, pos).draw(&mut fb_res.frame_buf).unwrap();
            }
        }
    }

    // --- Render the player ghost ---
    if let Some(bmp) = texture_assets.ghost.as_ref() {
        // Use the player position as the top‑left coordinate.
        let screen_x = player_pos.x as i32 - offset_x;
        let screen_y = player_pos.y as i32 - offset_y;
        let pos = Point::new(screen_x, screen_y);
        Image::new(bmp, pos).draw(&mut fb_res.frame_buf).unwrap();
    }

    // Flush the completed framebuffer to the physical display.
    let area = Rectangle::new(Point::zero(), fb_res.frame_buf.size());
    display_res
        .display
        .fill_contiguous(&area, fb_res.frame_buf.data.iter().copied())
        .unwrap();
}
