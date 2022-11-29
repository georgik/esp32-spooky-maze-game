// Based on https://github.com/bernii/embedded-graphics-framebuf

use embedded_graphics::{
    prelude::{RgbColor, PixelIteratorExt},
    prelude::{PixelColor, Point, DrawTarget, Size},
    geometry::OriginDimensions,
    Pixel,
};
use embedded_graphics::{image::Image, pixelcolor::Rgb565};
use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};

pub struct SpriteBuf<B: FrameBufferBackend<Color = Rgb565>> {
    pub data: B,
    width: usize,
    height: usize,
}

impl<B: FrameBufferBackend<Color = Rgb565>> OriginDimensions for SpriteBuf<B> {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl<B: FrameBufferBackend<Color = Rgb565>> SpriteBuf<B> {
    pub fn new(data: B, width: usize, height: usize) -> Self {
        assert_eq!(
            data.nr_elements(),
            width * height,
            "FrameBuf underlying data size does not match width ({}) * height ({}) = {} but is {}",
            width,
            height,
            width * height,
            data.nr_elements(),
        );
        Self {
            data,
            width,
            height,
        }
    }

    /// Get the framebuffers width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the framebuffers height.
    pub fn height(&self) -> usize {
        self.height
    }

    fn point_to_index(&self, p: Point) -> usize {
        self.width * p.y as usize + p.x as usize
    }

    /// Set a pixel's color.
    pub fn set_color_at(&mut self, p: Point, color: Rgb565) {
        self.data.set(self.point_to_index(p), color)
    }

    /// Get a pixel's color.
    pub fn get_color_at(&self, p: Point) -> Rgb565 {
        self.data.get(self.point_to_index(p))
    }
}


/// An iterator for all [Pixels](Pixel) in the framebuffer.
pub struct PixelIterator<'a, B: FrameBufferBackend<Color = Rgb565>> {
    fbuf: &'a SpriteBuf<B>,
    index: usize,
}


impl<'a, B: FrameBufferBackend<Color = Rgb565>> IntoIterator for &'a SpriteBuf<B> {
    type Item = Pixel<Rgb565>;
    type IntoIter = PixelIterator<'a, B>;

    /// Creates an iterator over all [Pixels](Pixel) in the frame buffer. Can be
    /// used for rendering the framebuffer to the physical display.
    ///
    /// # Example
    /// ```rust
    /// use embedded_graphics::{
    ///     draw_target::DrawTarget,
    ///     mock_display::MockDisplay,
    ///     pixelcolor::BinaryColor,
    ///     prelude::{Point, Primitive},
    ///     primitives::{Line, PrimitiveStyle},
    ///     Drawable,
    /// };
    /// use embedded_graphics_framebuf::FrameBuf;
    /// let mut data = [BinaryColor::Off; 12 * 11];
    /// let mut fbuf = FrameBuf::new(&mut data, 12, 11);
    /// let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
    /// Line::new(Point::new(2, 2), Point::new(10, 2))
    ///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
    ///     .draw(&mut fbuf)
    ///     .unwrap();
    /// display.draw_iter(fbuf.into_iter()).unwrap();
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        PixelIterator {
            fbuf: self,
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
                && coord.x < self.width as i32
                && coord.y >= 0
                && coord.y < self.height as i32
            {
                self.set_color_at(coord, color);
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_color_at(Point::new(x as i32, y as i32), color);
            }
        }
        Ok(())
    }
}

impl<'a, B: FrameBufferBackend<Color = Rgb565>> Iterator for PixelIterator<'a, B> {
    type Item = Pixel<Rgb565>;
    fn next(&mut self) -> Option<Pixel<Rgb565>> {
        let y = self.index / self.fbuf.width;
        let x = self.index - y * self.fbuf.width;

        if self.index >= self.fbuf.width * self.fbuf.height {
            return None;
        }
        self.index += 1;
        let p = Point::new(x as i32, y as i32);
        Some(Pixel(p, self.fbuf.get_color_at(p)))
    }
}

