// Based on https://github.com/bernii/embedded-graphics-framebuf

use embedded_graphics::{
    prelude::{RgbColor},
    prelude::{Point, DrawTarget, Size},
    geometry::OriginDimensions,
    Pixel,
};
use embedded_graphics::{pixelcolor::Rgb565};
use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};

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


/// An iterator for all [Pixels](Pixel) in the framebuffer.
pub struct PixelIterator<'a, B: FrameBufferBackend<Color = Rgb565>> {
    fbuf: &'a FrameBuf<Rgb565, B>,
    index: usize,
}


impl<'a, B: FrameBufferBackend<Color = Rgb565>> IntoIterator for &'a SpriteBuf<B> {
    type Item = Pixel<Rgb565>;
    type IntoIter = PixelIterator<'a, B>;

    fn into_iter(self) -> Self::IntoIter {
        PixelIterator {
            fbuf: &self.fbuf,
            index: 0,
        }
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

impl<'a, B: FrameBufferBackend<Color = Rgb565>> Iterator for PixelIterator<'a, B> {
    type Item = Pixel<Rgb565>;
    fn next(&mut self) -> Option<Pixel<Rgb565>> {
        let y = self.index / self.fbuf.width();
        let x = self.index - y * self.fbuf.width();

        if self.index >= self.fbuf.width() * self.fbuf.height() {
            return None;
        }
        self.index += 1;
        let p = Point::new(x as i32, y as i32);
        Some(Pixel(p, self.fbuf.get_color_at(p)))
    }
}

