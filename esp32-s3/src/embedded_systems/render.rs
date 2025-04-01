#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;

// Import embedded_graphics traits.
use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
};

use bevy_ecs::prelude::*;
use spooky_core::resources::{MazeResource, PlayerPosition};
// In no_std mode our TextureAssets were defined in spooky_core's setup module.
#[cfg(not(feature = "std"))]
use spooky_core::systems::setup::TextureAssets;

/// Render the maze tile map, coins, and the player ghost into the off‑screen framebuffer,
/// then flush it to the physical display.
///
/// In no_std mode this uses our embedded TextureAssets (loaded via tinybmp) to draw each tile.
/// The drawing order is:
/// 1. The full maze tile map (background)
/// 2. The coins (from maze.coins)
/// 3. The player ghost (from PlayerPosition)
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

    // --- Render the maze tile map (background) ---
    for ty in 0..maze.height as i32 {
        for tx in 0..maze.width as i32 {
            // Maze data: row 0 is at the top so flip the row index.
            let maze_row = (maze.height as i32 - 1) - ty;
            let index = (maze_row * maze.width as i32 + tx) as usize;
            // Choose the texture based on maze data.
            let bmp_opt = match maze.data[index] {
                1 => texture_assets.wall.as_ref(),
                0 => texture_assets.ground.as_ref(),
                2 => texture_assets.scorched.as_ref(),
                _ => texture_assets.ground.as_ref(),
            };

            if let Some(bmp) = bmp_opt {
                // Compute the pixel position for this tile.
                let x = left + tx * maze.tile_width as i32;
                let y = bottom + ty * maze.tile_height as i32;
                let pos = Point::new(x, y);
                Image::new(bmp, pos)
                    .draw(&mut fb_res.frame_buf)
                    .unwrap();
            }
        }
    }

    // --- Render coins on top of the maze ---
    for coin in &maze.coins {
        if coin.x != -1 && coin.y != -1 {
            if let Some(bmp) = texture_assets.coin.as_ref() {
                // Assume coin.x, coin.y are already in pixel coordinates.
                let pos = Point::new(coin.x, coin.y);
                Image::new(bmp, pos)
                    .draw(&mut fb_res.frame_buf)
                    .unwrap();
            }
        }
    }

    // --- Render the player ghost ---
    if let Some(bmp) = texture_assets.ghost.as_ref() {
        let pos = Point::new(player_pos.x as i32, player_pos.y as i32);
        Image::new(bmp, pos)
            .draw(&mut fb_res.frame_buf)
            .unwrap();
    }

    // Flush the completed framebuffer to the physical display.
    let area = Rectangle::new(Point::zero(), fb_res.frame_buf.size());
    display_res
        .display
        .fill_contiguous(&area, fb_res.frame_buf.data.iter().copied())
        .unwrap();
}
