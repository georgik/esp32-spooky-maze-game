
#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;

// Import embedded_graphics traits.
use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
};

use bevy_ecs::prelude::*;
use spooky_core::resources::MazeResource;
use spooky_core::systems::setup::TextureAssets;

/// Render the maze tile map into the off‑screen framebuffer and then flush it to the display.
///
/// In no_std mode, this function uses the embedded TextureAssets (loaded via tinybmp)
/// to draw each maze tile (wall, ground, scorched) at its proper position.
pub fn render_system(
    mut display_res: NonSendMut<crate::DisplayResource>,
    mut fb_res: ResMut<crate::FrameBufferResource>,
    maze_res: Res<MazeResource>,
    #[cfg(not(feature = "std"))] texture_assets: Res<TextureAssets>,
) {
    // Clear the framebuffer.
    fb_res.frame_buf.clear(Rgb565::BLACK).unwrap();

    let maze = &maze_res.maze;
    let (left, bottom, _right, _top) = maze.playable_bounds();

    // Iterate over each maze tile.
    for ty in 0..maze.height as i32 {
        for tx in 0..maze.width as i32 {
            // Maze data row 0 is at the top so flip the y coordinate.
            let maze_row = (maze.height as i32 - 1) - ty;
            let index = (maze_row * maze.width as i32 + tx) as usize;
            // Choose the texture based on the maze data.
            let bmp_opt = match maze.data[index] {
                1 => texture_assets.wall.as_ref(),
                0 => texture_assets.ground.as_ref(),
                2 => texture_assets.scorched.as_ref(),
                _ => texture_assets.ground.as_ref(),
            };

            if let Some(bmp) = bmp_opt {
                // Compute the pixel position for the tile.
                let x = left + tx * maze.tile_width as i32;
                let y = bottom + ty * maze.tile_height as i32;
                let pos = Point::new(x, y);
                // Draw the image (BMP) at the computed position.
                Image::new(bmp, pos)
                    .draw(&mut fb_res.frame_buf)
                    .unwrap();
            }
        }
    }
    // Flush the framebuffer to the physical display.
    let area = Rectangle::new(Point::zero(), fb_res.frame_buf.size());
    display_res
        .display
        .fill_contiguous(&area, fb_res.frame_buf.data.iter().copied())
        .unwrap();
}
