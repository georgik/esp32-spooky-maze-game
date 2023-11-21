pub const LCD_H_RES: u16 = 320;
pub const LCD_V_RES: u16 = 240;
pub const COLOR_DEPTH_BYTES: usize = core::mem::size_of::<embedded_graphics::pixelcolor::Rgb565>(); // = 2

pub const LCD_PIXELS: usize = (LCD_H_RES as usize) * (LCD_V_RES as usize);
pub const LCD_MEMORY_SIZE: usize = LCD_PIXELS * COLOR_DEPTH_BYTES;
