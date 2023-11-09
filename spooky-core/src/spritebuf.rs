// Based on https://github.com/bernii/embedded-graphics-framebuf

use embedded_graphics::{
    prelude::{RgbColor},
    prelude::{Point, DrawTarget, Size},
    geometry::OriginDimensions,
    Pixel,
};
use embedded_graphics::{pixelcolor::Rgb565};
use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend, PixelIterator};

pub struct SpriteBuf<B: FrameBufferBackend<Color = Rgb565>> {
    pub fbuf: FrameBuf<Rgb565, B>,
}

impl<B: FrameBufferBackend<Color = Rgb565>> OriginDimensions for SpriteBuf<B> {
    fn size(&self) -> Size {
        self.fbuf.size()
    }
}

impl<B: FrameBufferBackend<Color = Rgb565>> SpriteBuf<B> {
    pub fn new(fbuf:FrameBuf<Rgb565, B>) -> Self {
        Self {
            fbuf,
        }
    }

    /// Get the framebuffers width.
    pub fn width(&self) -> usize {
        self.fbuf.width()
    }

    /// Get the framebuffers height.
    pub fn height(&self) -> usize {
        self.fbuf.height()
    }

    /// Set a pixel's color.
    pub fn set_color_at(&mut self, p: Point, color: Rgb565) {
        self.fbuf.set_color_at(p, color)
    }

    /// Get a pixel's color.
    pub fn get_color_at(&self, p: Point) -> Rgb565 {
        self.fbuf.get_color_at(p)
    }
}

impl<'a, B: FrameBufferBackend<Color = Rgb565>> IntoIterator for &'a SpriteBuf<B> {
    type Item = Pixel<Rgb565>;
    type IntoIter = PixelIterator<'a, Rgb565,  B>;

    fn into_iter(self) -> Self::IntoIter {
        self.fbuf.into_iter()
    }
}

impl<B: FrameBufferBackend<Color = Rgb565>> DrawTarget for SpriteBuf<B> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if color.g() == 0 && color.b() == 31 && color.r() == 31 {
                continue;
            }
            if coord.x >= 0
                && coord.x < self.width() as i32
                && coord.y >= 0
                && coord.y < self.height() as i32
            {
                self.fbuf.set_color_at(coord, color);
            }
        }
        Ok(())
    }
}


// impl<B> SpriteBuf<B>
// where
//     B: FrameBufferBackend<Color = Rgb565>,
// {
//     pub fn get_pixel_iter(&self) -> impl Iterator<Item = Pixel<Rgb565>> + '_ {
//         self.fbuf.into_iter()
//     }
// }

impl<B> SpriteBuf<B>
where
    B: FrameBufferBackend<Color = Rgb565>,
{
    pub fn get_pixel_iter(&self) -> impl Iterator<Item = Rgb565> + '_ {
        self.fbuf.into_iter().map(|pixel| pixel.1)
    }
}
