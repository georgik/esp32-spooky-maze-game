#[cfg(not(feature = "std"))]
use embedded_graphics::pixelcolor::Rgb565;
#[cfg(not(feature = "std"))]
use embedded_graphics::{
    image::Image,
    prelude::*,
    primitives::Rectangle,
};

use bevy_ecs::prelude::*;
use spooky_core::resources::{MazeResource, PlayerPosition};

#[cfg(not(feature = "std"))]
use spooky_core::systems::setup::TextureAssets;

/// A borrowed sprite buffer wrapper that implements a DrawTarget filtering out “magic pink”.
/// In our case, we treat any pixel with R=31, G=0, B=31 as transparent.
pub struct SpriteBuf<'a, B: embedded_graphics_framebuf::backends::FrameBufferBackend<Color = Rgb565>> {
    pub fbuf: &'a mut embedded_graphics_framebuf::FrameBuf<Rgb565, B>,
}

impl<'a, B: embedded_graphics_framebuf::backends::FrameBufferBackend<Color=Rgb565>> Dimensions for SpriteBuf<'a, B> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::zero(), self.fbuf.size())
    }
}

impl<'a, B: embedded_graphics_framebuf::backends::FrameBufferBackend<Color = Rgb565>> DrawTarget
for SpriteBuf<'a, B>
{
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // If the pixel is "magic pink" (R=31, G=0, B=31), skip it.
            if color.r() == 31 && color.g() == 0 && color.b() == 31 {
                continue;
            }
            // Only draw pixels within bounds.
            if coord.x >= 0
                && coord.x < self.fbuf.width() as i32
                && coord.y >= 0
                && coord.y < self.fbuf.height() as i32
            {
                self.fbuf.set_color_at(coord, color);
            }
        }
        Ok(())
    }
}

/// Render the scene. First, the maze background is drawn directly to the framebuffer.
/// Then a temporary SpriteBuf wraps the framebuffer to draw sprites (coins and player ghost)
/// with pink filtering. Finally, the complete framebuffer is flushed to the display.
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

    // Compute camera offset so that the player's world position is centered.
    let offset_x = player_pos.x as i32 - display_center_x;
    let offset_y = player_pos.y as i32 - display_center_y;

    // Determine the visible region in world space.
    let visible_left = offset_x;
    let visible_right = offset_x + display_width;
    let visible_bottom = offset_y;
    let visible_top = offset_y + display_height;

    // Compute visible tile indices (clamped to maze dimensions).
    let min_tx = ((visible_left - maze_left) / tile_w).max(0);
    let max_tx = ((visible_right - maze_left) / tile_w).min(maze.width as i32 - 1);
    let min_ty = ((visible_bottom - maze_bottom) / tile_h).max(0);
    let max_ty = ((visible_top - maze_bottom) / tile_h).min(maze.height as i32 - 1);

    // --- Draw the maze background directly ---
    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            // Compute the tile's world coordinate (top‑left of the tile).
            let world_x = maze_left + tx * tile_w;
            let world_y = maze_bottom + ty * tile_h;
            // Convert to screen coordinates.
            let screen_x = world_x - offset_x;
            let screen_y = world_y - offset_y;
            let pos = Point::new(screen_x, screen_y);
            // The maze data is stored in row‑major order (with row 0 at the top).
            let tile_index = (ty * maze.width as i32 + tx) as usize;
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

    // --- Draw sprites (coins and player ghost) with sprite filtering ---
    {
        // Wrap the framebuffer with our SpriteBuf so that drawing skips pink pixels.
        let mut sprite_buf = SpriteBuf {
            fbuf: &mut fb_res.frame_buf,
        };
        // Draw coins.
        for coin in &maze.coins {
            if coin.x != -1 && coin.y != -1 {
                if let Some(bmp) = texture_assets.coin.as_ref() {
                    let screen_x = coin.x - offset_x;
                    let screen_y = coin.y - offset_y;
                    let pos = Point::new(screen_x, screen_y);
                    Image::new(bmp, pos)
                        .draw(&mut sprite_buf)
                        .unwrap();
                }
            }
        }
        // Draw the player ghost.
        if let Some(bmp) = texture_assets.ghost.as_ref() {
            let screen_x = player_pos.x as i32 - offset_x;
            let screen_y = player_pos.y as i32 - offset_y;
            let pos = Point::new(screen_x, screen_y);
            Image::new(bmp, pos)
                .draw(&mut sprite_buf)
                .unwrap();
        }
    }

    // Finally, flush the framebuffer to the display.
    let area = Rectangle::new(Point::zero(), fb_res.frame_buf.size());
    display_res
        .display
        .fill_contiguous(&area, fb_res.frame_buf.data.iter().copied())
        .unwrap();
}
