#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;

use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::{Rectangle},
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
/// Tiles that fall outside the visible area are not drawn.
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
    let (maze_left, maze_bottom, _maze_right, _maze_top) = maze.playable_bounds();
    let tile_w = maze.tile_width as i32;
    let tile_h = maze.tile_height as i32;

    // Display dimensions.
    let display_width = crate::LCD_H_RES as i32;
    let display_height = crate::LCD_V_RES as i32;
    let display_center_x = display_width / 2;
    let display_center_y = display_height / 2;

    // Compute camera offset so that the player is centered.
    let offset_x = player_pos.x as i32 - display_center_x;
    let offset_y = player_pos.y as i32 - display_center_y;

    // Determine visible region in world coordinates.
    // (Since we use top‑left of tile as the anchor, visible region is [offset, offset+display_dim].)
    let visible_left = offset_x;
    let visible_right = offset_x + display_width;
    let visible_bottom = offset_y;
    let visible_top = offset_y + display_height;

    // Convert visible world coordinates to tile indices.
    // Clamp the indices to the maze dimensions.
    let min_tx = ((visible_left - maze_left) / tile_w).max(0);
    let max_tx = ((visible_right - maze_left) / tile_w).min(maze.width as i32 - 1);
    let min_ty = ((visible_bottom - maze_bottom) / tile_h).max(0);
    let max_ty = ((visible_top - maze_bottom) / tile_h).min(maze.height as i32 - 1);

    // --- Render the maze tile map (background) ---
    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            // Compute the tile's world coordinate (using top‑left as anchor).
            let world_x = maze_left + tx * tile_w;
            let world_y = maze_bottom + ty * tile_h;
            // Convert to screen coordinates by subtracting the camera offset.
            let screen_x = world_x - offset_x;
            let screen_y = world_y - offset_y;
            let pos = Point::new(screen_x, screen_y);

            // The data array is assumed to be laid out in row‑major order with row 0 at the top.
            // Since our maze tiles are placed with row 0 at the bottom, we need to flip the y index.
            let maze_row = (maze.height as i32 - 1) - ty;
            let tile_index = (maze_row * maze.width as i32 + tx) as usize;

            let bmp_opt = match maze.data[tile_index] {
                1 => texture_assets.wall.as_ref(),
                0 => texture_assets.ground.as_ref(),
                2 => texture_assets.scorched.as_ref(),
                _ => texture_assets.ground.as_ref(),
            };

            if let Some(bmp) = bmp_opt {
                Image::new(bmp, pos)
                    .draw(&mut fb_res.frame_buf)
                    .unwrap();
            }
        }
    }

    // --- Render coins (using top‑left as anchor) ---
    for coin in &maze.coins {
        if coin.x != -1 && coin.y != -1 {
            if let Some(bmp) = texture_assets.coin.as_ref() {
                let screen_x = coin.x - offset_x;
                let screen_y = coin.y - offset_y;
                let pos = Point::new(screen_x, screen_y);
                Image::new(bmp, pos)
                    .draw(&mut fb_res.frame_buf)
                    .unwrap();
            }
        }
    }

    // --- Render the player ghost (using top‑left as anchor) ---
    if let Some(bmp) = texture_assets.ghost.as_ref() {
        let screen_x = player_pos.x as i32 - offset_x;
        let screen_y = player_pos.y as i32 - offset_y;
        let pos = Point::new(screen_x, screen_y);
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
