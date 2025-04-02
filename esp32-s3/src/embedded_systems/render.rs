#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;

use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
};

use bevy_ecs::prelude::*;
use spooky_core::resources::{MazeResource, PlayerPosition};

#[cfg(not(feature = "std"))]
use spooky_core::systems::setup::TextureAssets;

/// Render the maze tile map, coins, and the player ghost into the off‑screen framebuffer,
/// then flush it to the physical display.
///
/// Drawing order is:
/// 1. Maze tile map (background)
/// 2. Coins (drawn so that they’re centered on the tile)
/// 3. (Other elements, such as the player, might be drawn separately)
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

    // --- Render the maze tile map (background) ---
    for ty in 0..maze.height as i32 {
        for tx in 0..maze.width as i32 {
            // Since our maze data is stored bottom‑up, we can use the current (tx, ty)
            // directly if valid_coordinates are computed that way.
            let tile_index = (ty * maze.width as i32 + tx) as usize;
            let bmp_opt = match maze.data[tile_index] {
                1 => texture_assets.wall.as_ref(),
                0 => texture_assets.ground.as_ref(),
                2 => texture_assets.scorched.as_ref(),
                _ => texture_assets.ground.as_ref(),
            };

            if let Some(bmp) = bmp_opt {
                // Compute the top‑left pixel coordinate of this tile.
                let x = left + tx * tile_w;
                let y = bottom + ty * tile_h;
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
                // The valid coordinate is the center of the tile.
                // Subtract half the tile dimensions so the coin is centered.
                let pos = Point::new(coin.x - tile_w / 2, coin.y - tile_h / 2);
                Image::new(bmp, pos)
                    .draw(&mut fb_res.frame_buf)
                    .unwrap();
            }
        }
    }

    // (You can add additional drawing for the player ghost here if needed.)

    // Flush the completed framebuffer to the physical display.
    let area = Rectangle::new(Point::zero(), fb_res.frame_buf.size());
    display_res
        .display
        .fill_contiguous(&area, fb_res.frame_buf.data.iter().copied())
        .unwrap();
}
