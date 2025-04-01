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

/// Render the maze, coins, and player ghost onto the off‑screen framebuffer and then flush
/// the result to the display. This version calculates the visible world area based on the
/// camera offset (computed from the player’s position) so that only the visible tiles are drawn.
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
    // Get the maze playable bounds (world coordinates of the maze’s bottom‐left corner).
    let (left, bottom, _right, _top) = maze.playable_bounds();

    // Our “camera” centers on the player.
    let display_width = crate::LCD_H_RES as i32;
    let display_height = crate::LCD_V_RES as i32;
    let display_center_x = display_width / 2;
    let display_center_y = display_height / 2;

    // Calculate the offset: the translation we apply so that the player (in world coordinates)
    // appears at the center of the display.
    let offset_x = display_center_x - player_pos.x as i32;
    let offset_y = display_center_y - player_pos.y as i32;

    // Determine tile size (in pixels).
    // Assume these values are already computed as i32:
    let tile_w = maze.tile_width as i32;
    let tile_h = maze.tile_height as i32;

    // The visible world coordinate range based on the camera offset:
    let min_world_x = 0 - offset_x;
    let max_world_x = display_width - offset_x;
    let min_world_y = 0 - offset_y;
    let max_world_y = display_height - offset_y;

    // Use integer arithmetic to calculate the tile indices (avoiding floats)
    let min_tx = (min_world_x - left) / tile_w;
    let max_tx = (max_world_x - left + tile_w - 1) / tile_w;
    let min_ty = (min_world_y - bottom) / tile_h;
    let max_ty = (max_world_y - bottom + tile_h - 1) / tile_h;

    // Clamp indices to maze dimensions.
    let start_tx = min_tx.max(0);
    let end_tx = max_tx.min(maze.width as i32);
    let start_ty = min_ty.max(0);
    let end_ty = max_ty.min(maze.height as i32);

    // Now iterate only over the visible tiles.
    for ty in start_ty..end_ty {
        for tx in start_tx..end_tx {
            // In maze data, row 0 is at the top, so flip the ty index.
            let maze_row = (maze.height as i32 - 1) - ty;
            let index = (maze_row * maze.width as i32 + tx) as usize;
            // Select the texture based on maze.data[index].
            let bmp_opt = match maze.data[index] {
                1 => texture_assets.wall.as_ref(),
                0 => texture_assets.ground.as_ref(),
                2 => texture_assets.scorched.as_ref(),
                _ => texture_assets.ground.as_ref(),
            };

            if let Some(bmp) = bmp_opt {
                // Compute the world position of the tile.
                let world_x = left + tx * tile_w;
                let world_y = bottom + ty * tile_h;
                // Convert to screen coordinates:
                let screen_x = world_x + offset_x;
                let screen_y = world_y + offset_y;
                let pos = Point::new(screen_x, screen_y);
                Image::new(bmp, pos)
                    .draw(&mut fb_res.frame_buf)
                    .unwrap();
            }
        }
    }

    // --- Render coins ---
    for coin in &maze.coins {
        if coin.x != -1 && coin.y != -1 {
            if let Some(bmp) = texture_assets.coin.as_ref() {
                // Assume coin.x and coin.y are in world coordinates.
                let screen_x = coin.x + offset_x;
                let screen_y = coin.y + offset_y;
                let pos = Point::new(screen_x, screen_y);
                Image::new(bmp, pos).draw(&mut fb_res.frame_buf).unwrap();
            }
        }
    }

    // --- Render the player ghost ---
    if let Some(bmp) = texture_assets.ghost.as_ref() {
        // Draw the player at the center of the display.
        let pos = Point::new(display_center_x, display_center_y);
        Image::new(bmp, pos).draw(&mut fb_res.frame_buf).unwrap();
    }

    // Flush the completed framebuffer to the physical display.
    let area = Rectangle::new(Point::zero(), fb_res.frame_buf.size());
    display_res.display.fill_contiguous(&area, fb_res.frame_buf.data.iter().copied()).unwrap();
}
