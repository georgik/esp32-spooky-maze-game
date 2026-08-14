use alloc::format;
use embedded_graphics::{
    image::Image,
    mono_font::{
        MonoTextStyle,
        ascii::{
            FONT_6X10,
            FONT_10X20,
        },
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        PrimitiveStyleBuilder,
        Rectangle,
    },
    text::Text,
};

use crate::touch::{
    ButtonRect,
    Direction,
    DOWN_BUTTON,
    LEFT_BUTTON,
    RIGHT_BUTTON,
    TouchInputState,
    UP_BUTTON,
};

use bevy_ecs::prelude::*;


    use esp_hal::time::Instant as HalInstant;///////////////////////////////////
    use esp_println::println;/////////////////////////////////////////////////


use spooky_core::resources::{MazeResource, PlayerPosition};
use spooky_core::systems::hud::HudState;
use spooky_core::systems::setup::TextureAssets;



/// A borrowed sprite buffer wrapper that implements a DrawTarget filtering out “magic pink”.
/// In our case, we treat any pixel with R=31, G=0, B=31 as transparent.
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
/// 

pub fn render_system(
    mut fb_res: ResMut<crate::FrameBufferResource>,
    maze_res: Res<MazeResource>,
    texture_assets: Res<TextureAssets>,
    player_pos: Res<PlayerPosition>,
    hud_state: Res<HudState>,
    touch_state: Res<TouchInputState>,
) {




    
    
    
    let test_start = HalInstant::now();/////////////////////////////////////////////
    
    
    // Clear the framebuffer.
    //fb_res.frame_buf.data.fill(Rgb565::BLACK);
    
    unsafe {
    core::ptr::write_bytes(
        fb_res.frame_buf.data.as_mut_ptr(),
        0,
        crate::LCD_BUFFER_SIZE,
    );
    }





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
                Image::new(bmp, pos).draw(&mut fb_res.frame_buf).unwrap();
            }
        }
    }
    let test_ms = test_start.elapsed().as_millis();///////////////////




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
                    Image::new(bmp, pos).draw(&mut sprite_buf).unwrap();
                }
            }
        }
        // Draw the player ghost.
        if let Some(bmp) = texture_assets.ghost.as_ref() {
            let screen_x = player_pos.x as i32 - offset_x;
            let screen_y = player_pos.y as i32 - offset_y;
            let pos = Point::new(screen_x, screen_y);
            Image::new(bmp, pos).draw(&mut sprite_buf).unwrap();
        }
    }

    // --- Render HUD overlay ---
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let hud_start_x = 5;
    let mut hud_start_y = 12;
    let line_height = 12;

    let coins_line = format!("Coins: {}", hud_state.coins_left);
    let teleport_line = format!("Teleport: {}", hud_state.teleport_countdown);
    let walker_line = format!("Walker: {}", hud_state.walker_timer);
    let dynamite_line = format!("Dynamite: {}", hud_state.dynamites);

    // Draw each HUD line.
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

    draw_touch_controls(
        &mut fb_res.frame_buf,
        &touch_state,
    );







    println!("{test_ms} ms");  ///////////////////////////////////////////////////////////////






}

fn draw_touch_button<B>(
    frame_buf: &mut embedded_graphics_framebuf::FrameBuf<Rgb565, B>,
    rect: ButtonRect,
    label: &str,
    pressed: bool,
) where
    B: embedded_graphics_framebuf::backends::FrameBufferBackend<
        Color = Rgb565,
    >,
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

    rectangle
        .into_styled(style)
        .draw(frame_buf)
        .unwrap();

    let text_style =
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

    // A single character is ten pixels wide in FONT_10X20.
    let label_x =
        rect.x + (rect.width as i32 - 10) / 2;

    let label_y =
        rect.y + (rect.height as i32 + 20) / 2;

    Text::new(
        label,
        Point::new(label_x, label_y),
        text_style,
    )
    .draw(frame_buf)
    .unwrap();
}

fn draw_touch_controls<B>(
    frame_buf: &mut embedded_graphics_framebuf::FrameBuf<Rgb565, B>,
    touch_state: &TouchInputState,
) where
    B: embedded_graphics_framebuf::backends::FrameBufferBackend<
        Color = Rgb565,
    >,
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