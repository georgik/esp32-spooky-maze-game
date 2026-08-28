use alloc::{boxed::Box, format, vec};

use bevy_ecs::prelude::*;

use embedded_graphics::{
    image::Image,
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10, FONT_10X20},
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

use tinybmp::Bmp;

use spooky_core::{
    resources::{MazeResource, PlayerPosition},
    systems::{hud::HudState, setup::TextureAssets},
};

use crate::touch::{
    ButtonRect, DOWN_BUTTON, Direction, LEFT_BUTTON, RIGHT_BUTTON, TouchInputState, UP_BUTTON,
};

// Set this to true if removing the full-screen clear causes visual artifacts.
//
// A full clear writes approximately 1.2 MB to PSRAM every frame and can
// therefore be very expensive.
const CLEAR_FRAMEBUFFER_EACH_FRAME: bool = true;

// RGB565 representation of R=31, G=0, B=31.
const MAGIC_PINK_RGB565: u16 = 0xF81F;

// -----------------------------------------------------------------------------
// Cached RGB565 textures
// -----------------------------------------------------------------------------

/// A BMP image decoded once into a contiguous RGB565 pixel buffer.
///
/// Maze tiles are opaque, so they can be copied directly into the framebuffer
/// one row at a time.
pub(crate) struct FastTexture {
    pixels: Box<[Rgb565]>,
    width: usize,
    height: usize,
}

impl FastTexture {
    fn from_bmp(bmp: &Bmp<'_, Rgb565>) -> Self {
        let size = bmp.size();

        let width = size.width as usize;
        let height = size.height as usize;

        let mut pixels = vec![Rgb565::BLACK; width * height];

        // Decode the BMP only once.
        for Pixel(position, color) in bmp.pixels() {
            if position.x < 0
                || position.y < 0
                || position.x >= width as i32
                || position.y >= height as i32
            {
                continue;
            }

            let index = position.y as usize * width + position.x as usize;

            pixels[index] = color;
        }

        Self {
            pixels: pixels.into_boxed_slice(),
            width,
            height,
        }
    }
}

/// Cached versions of the opaque maze textures.
pub(crate) struct FastMazeTextures {
    wall: Option<FastTexture>,
    ground: Option<FastTexture>,
    scorched: Option<FastTexture>,
}

impl FastMazeTextures {
    fn from_assets(assets: &TextureAssets) -> Self {
        Self {
            wall: assets.wall.as_ref().map(FastTexture::from_bmp),

            ground: assets.ground.as_ref().map(FastTexture::from_bmp),

            scorched: assets.scorched.as_ref().map(FastTexture::from_bmp),
        }
    }

    fn get(&self, tile_type: u8) -> Option<&FastTexture> {
        match tile_type {
            1 => self.wall.as_ref(),
            0 => self.ground.as_ref(),
            2 => self.scorched.as_ref(),
            _ => self.ground.as_ref(),
        }
    }
}

// -----------------------------------------------------------------------------
// Direct framebuffer blitting
// -----------------------------------------------------------------------------

/// Copies an opaque RGB565 texture directly into the framebuffer.
///
/// The texture is copied one contiguous row at a time instead of sending every
/// pixel through embedded-graphics' generic DrawTarget implementation.
fn blit_opaque(
    destination: &mut [Rgb565],
    destination_width: usize,
    destination_height: usize,
    texture: &FastTexture,
    destination_x: i32,
    destination_y: i32,
) {
    // Reject textures completely outside the display.
    if destination_x >= destination_width as i32
        || destination_y >= destination_height as i32
        || destination_x + texture.width as i32 <= 0
        || destination_y + texture.height as i32 <= 0
    {
        return;
    }

    // Clip the source against the left side of the framebuffer.
    let source_x_start = if destination_x < 0 {
        (-destination_x) as usize
    } else {
        0
    };

    // Clip the source against the top side of the framebuffer.
    let source_y_start = if destination_y < 0 {
        (-destination_y) as usize
    } else {
        0
    };

    let destination_x_start = destination_x.max(0) as usize;

    let destination_y_start = destination_y.max(0) as usize;

    if source_x_start >= texture.width
        || source_y_start >= texture.height
        || destination_x_start >= destination_width
        || destination_y_start >= destination_height
    {
        return;
    }

    // Clip against the right side of the framebuffer.
    let copy_width = (texture.width - source_x_start).min(destination_width - destination_x_start);

    // Clip against the bottom side of the framebuffer.
    let copy_height =
        (texture.height - source_y_start).min(destination_height - destination_y_start);

    if copy_width == 0 || copy_height == 0 {
        return;
    }

    for row in 0..copy_height {
        let source_start = (source_y_start + row) * texture.width + source_x_start;

        let destination_start =
            (destination_y_start + row) * destination_width + destination_x_start;

        let source_end = source_start + copy_width;

        let destination_end = destination_start + copy_width;

        destination[destination_start..destination_end]
            .copy_from_slice(&texture.pixels[source_start..source_end]);
    }
}

// -----------------------------------------------------------------------------
// Transparent sprite target
// -----------------------------------------------------------------------------

/// A borrowed sprite buffer wrapper that filters out magic pink.
///
/// Pixels with RGB565 value 0xF81F are treated as transparent.
pub struct SpriteBuf<
    'a,
    B: embedded_graphics_framebuf::backends::FrameBufferBackend<Color = Rgb565>,
> {
    pub fbuf: &'a mut embedded_graphics_framebuf::FrameBuf<Rgb565, B>,
}

impl<'a, B: embedded_graphics_framebuf::backends::FrameBufferBackend<Color = Rgb565>> Dimensions
    for SpriteBuf<'a, B>
{
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
        let width = self.fbuf.width() as i32;

        let height = self.fbuf.height() as i32;

        for Pixel(coord, color) in pixels {
            // Use a single RGB565 comparison instead of extracting all
            // three color components separately.
            if color.into_storage() == MAGIC_PINK_RGB565 {
                continue;
            }

            if coord.x < 0 || coord.y < 0 || coord.x >= width || coord.y >= height {
                continue;
            }

            self.fbuf.set_color_at(coord, color);
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Main rendering system
// -----------------------------------------------------------------------------

pub fn render_system(
    mut fb_res: ResMut<crate::FrameBufferResource>,
    maze_res: Res<MazeResource>,
    texture_assets: Res<TextureAssets>,
    player_pos: Res<PlayerPosition>,
    hud_state: Res<HudState>,
    touch_state: Res<TouchInputState>,
    mut fast_textures: Local<Option<FastMazeTextures>>,
) {
    // Decode the three opaque maze textures only once.
    //
    // Startup systems run before the first Update schedule, so TextureAssets
    // should already contain the loaded BMP images here.
    let fast_textures =
        fast_textures.get_or_insert_with(|| FastMazeTextures::from_assets(&texture_assets));

    // -------------------------------------------------------------------------
    // Optional framebuffer clear
    // -------------------------------------------------------------------------

    if CLEAR_FRAMEBUFFER_EACH_FRAME {
        fb_res.frame_buf.data.fill(Rgb565::BLACK);
    }

    // -------------------------------------------------------------------------
    // Camera and visible maze calculations
    // -------------------------------------------------------------------------

    let maze = &maze_res.maze;

    let (maze_left, maze_bottom, _maze_right, _maze_top) = maze.playable_bounds();

    let tile_w = maze.tile_width as i32;

    let tile_h = maze.tile_height as i32;

    let display_width = crate::LCD_H_RES as i32;

    let display_height = crate::LCD_V_RES as i32;

    let display_center_x = display_width / 2;

    let display_center_y = display_height / 2;

    let mut offset_x: i32 = player_pos.x as i32 - display_center_x;
    let mut offset_y = player_pos.y as i32 - display_center_y;

    // capping movement of the camera on the horizontal axis
    if offset_x < -287 {
        offset_x = -288;
    } else if offset_x > -16 {
        offset_x = -16;
    }

    // capping movement of the camera on the vertical axis
    if offset_y < 4 {
        offset_y = 4;
    } else if offset_y > 420 {
        offset_y = 436;
    }

    //println!("{0}, {1}", offset_y, player_pos.y);

    let visible_left = offset_x;

    let visible_right = offset_x + display_width;

    let visible_bottom = offset_y;

    let visible_top = offset_y + display_height;

    let min_tx = ((visible_left - maze_left) / tile_w).max(0);

    let max_tx = ((visible_right - maze_left) / tile_w).min(maze.width as i32 - 1);

    let min_ty = ((visible_bottom - maze_bottom) / tile_h).max(0);

    let max_ty = ((visible_top - maze_bottom) / tile_h).min(maze.height as i32 - 1);

    // -------------------------------------------------------------------------
    // Draw the maze using direct row copies
    // -------------------------------------------------------------------------

    if min_tx <= max_tx && min_ty <= max_ty {
        let first_screen_x = maze_left + min_tx * tile_w - offset_x;

        let first_screen_y = maze_bottom + min_ty * tile_h - offset_y;

        let framebuffer_pixels = &mut fb_res.frame_buf.data[..];

        for ty in min_ty..=max_ty {
            let screen_y = first_screen_y + (ty - min_ty) * tile_h;

            let maze_row_start = ty as usize * maze.width as usize;

            for tx in min_tx..=max_tx {
                let screen_x = first_screen_x + (tx - min_tx) * tile_w;

                let tile_index = maze_row_start + tx as usize;

                let tile_type = maze.data[tile_index];

                if let Some(texture) = fast_textures.get(tile_type) {
                    blit_opaque(
                        framebuffer_pixels,
                        crate::LCD_H_RES,
                        crate::LCD_V_RES,
                        texture,
                        screen_x,
                        screen_y,
                    );
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Draw transparent sprites
    // -------------------------------------------------------------------------

    {
        let mut sprite_buf = SpriteBuf {
            fbuf: &mut fb_res.frame_buf,
        };

        // Draw only coins that overlap the display.
        for coin in &maze.coins {
            if coin.x == -1 || coin.y == -1 {
                continue;
            }

            let screen_x = coin.x - offset_x;

            let screen_y = coin.y - offset_y;

            // Reject completely off-screen coins.
            if screen_x + tile_w <= 0
                || screen_y + tile_h <= 0
                || screen_x >= display_width
                || screen_y >= display_height
            {
                continue;
            }

            if let Some(bmp) = texture_assets.coin.as_ref() {
                Image::new(bmp, Point::new(screen_x, screen_y))
                    .draw(&mut sprite_buf)
                    .unwrap();
            }
        }

        // The player is centered by the camera, but calculate the
        // position normally to preserve the original behavior.
        if let Some(bmp) = texture_assets.ghost.as_ref() {
            let screen_x = player_pos.x as i32 - offset_x;

            let screen_y = player_pos.y as i32 - offset_y;

            Image::new(bmp, Point::new(screen_x, screen_y))
                .draw(&mut sprite_buf)
                .unwrap();
        }
    }

    // -------------------------------------------------------------------------
    // Draw HUD and touchscreen controls
    // -------------------------------------------------------------------------

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let hud_start_x = 5;
    let mut hud_start_y = 12;
    let line_height = 12;

    let coins_line = format!("Coins: {}", hud_state.coins_left,);

    let teleport_line = format!("Teleport: {}", hud_state.teleport_countdown,);

    let walker_line = format!("Walker: {}", hud_state.walker_timer,);

    let dynamite_line = format!("Dynamite: {}", hud_state.dynamites,);

    Text::new(
        &coins_line,
        Point::new(hud_start_x, hud_start_y),
        text_style,
    )
    .draw(&mut fb_res.frame_buf)
    .unwrap();

    hud_start_y += line_height;

    Text::new(
        &teleport_line,
        Point::new(hud_start_x, hud_start_y),
        text_style,
    )
    .draw(&mut fb_res.frame_buf)
    .unwrap();

    hud_start_y += line_height;

    Text::new(
        &walker_line,
        Point::new(hud_start_x, hud_start_y),
        text_style,
    )
    .draw(&mut fb_res.frame_buf)
    .unwrap();

    hud_start_y += line_height;

    Text::new(
        &dynamite_line,
        Point::new(hud_start_x, hud_start_y),
        text_style,
    )
    .draw(&mut fb_res.frame_buf)
    .unwrap();

    draw_touch_controls(&mut fb_res.frame_buf, &touch_state);
}

// -----------------------------------------------------------------------------
// Touchscreen control rendering
// -----------------------------------------------------------------------------

fn draw_touch_button<B>(
    frame_buf: &mut embedded_graphics_framebuf::FrameBuf<Rgb565, B>,
    rect: ButtonRect,
    label: &str,
    pressed: bool,
) where
    B: embedded_graphics_framebuf::backends::FrameBufferBackend<Color = Rgb565>,
{
    let fill_color = if pressed {
        Rgb565::new(0, 40, 4)
    } else {
        Rgb565::new(5, 10, 5)
    };

    let border_color = if pressed {
        Rgb565::GREEN
    } else {
        Rgb565::WHITE
    };

    let rectangle = Rectangle::new(
        Point::new(rect.x, rect.y),
        Size::new(rect.width, rect.height),
    );

    let style = PrimitiveStyleBuilder::new()
        .fill_color(fill_color)
        .stroke_color(border_color)
        .stroke_width(3)
        .build();

    rectangle.into_styled(style).draw(frame_buf).unwrap();

    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

    let label_x = rect.x + (rect.width as i32 - 10) / 2;

    let label_y = rect.y + (rect.height as i32 + 20) / 2;

    Text::new(label, Point::new(label_x, label_y), text_style)
        .draw(frame_buf)
        .unwrap();
}

fn draw_touch_controls<B>(
    frame_buf: &mut embedded_graphics_framebuf::FrameBuf<Rgb565, B>,
    touch_state: &TouchInputState,
) where
    B: embedded_graphics_framebuf::backends::FrameBufferBackend<Color = Rgb565>,
{
    draw_touch_button(
        frame_buf,
        UP_BUTTON,
        "U",
        touch_state.pressed == Some(Direction::Up),
    );

    draw_touch_button(
        frame_buf,
        DOWN_BUTTON,
        "D",
        touch_state.pressed == Some(Direction::Down),
    );

    draw_touch_button(
        frame_buf,
        LEFT_BUTTON,
        "L",
        touch_state.pressed == Some(Direction::Left),
    );

    draw_touch_button(
        frame_buf,
        RIGHT_BUTTON,
        "R",
        touch_state.pressed == Some(Direction::Right),
    );
}
