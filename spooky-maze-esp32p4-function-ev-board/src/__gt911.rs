use core::fmt::Debug;

use embedded_hal::i2c::I2c;

pub const GT911_ADDRESS_1: u8 = 0x5D;
pub const GT911_ADDRESS_2: u8 = 0x14;

#[derive(Debug)]
pub enum Error<E> {
    Bus(E),
    InvalidProductId([u8; 4]),
    NoDataAvailable,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum Register {
    ProductId = 0x8140,
    PointInfo = 0x814E,
    Point1 = 0x814F,
}

#[derive(Copy, Clone, Debug)]
pub enum Orientation {
    Landscape,
    Portrait,
    InvertedLandscape,
    InvertedPortrait,
}

#[derive(Copy, Clone, Debug)]
pub struct Dimensions {
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TouchEvent {
    Pressed(TouchPoint),
    Released,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TouchPoint {
    pub id: u8,
    pub x: u16,
    pub y: u16,
    pub size: u16,
}

pub struct Gt911<I2C> {
    address: u8,
    i2c: I2C,
    orientation: Orientation,
    dimensions: Dimensions,
}

impl<I2C, E> Gt911<I2C>
where
    I2C: I2c<Error = E>,
    E: Debug,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            address,
            i2c,
            orientation: Orientation::InvertedLandscape,
            dimensions: Dimensions {
                width: 1024,
                height: 600,
            },
        }
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        self.dimensions = Dimensions { width, height };
    }

    /// Return ownership of the I²C peripheral.
    pub fn release(self) -> I2C {
        self.i2c
    }

    fn register_bytes(register: Register) -> [u8; 2] {
        let register = register as u16;

        [
            (register >> 8) as u8,
            (register & 0xFF) as u8,
        ]
    }

    fn clear_status(&mut self) -> Result<(), Error<E>> {
        let register = Register::PointInfo as u16;

        let data = [
            (register >> 8) as u8,
            (register & 0xFF) as u8,
            0,
        ];

        self.i2c
            .write(self.address, &data)
            .map_err(Error::Bus)
    }

    /// Read and return the GT911 product ID.
    pub fn read_product_id(
        &mut self,
    ) -> Result<[u8; 4], Error<E>> {
        let register = Self::register_bytes(Register::ProductId);
        let mut product_id = [0u8; 4];

        self.i2c
            .write_read(
                self.address,
                &register,
                &mut product_id,
            )
            .map_err(Error::Bus)?;

        Ok(product_id)
    }

    /// Verify that the device responds as a GT911 and clear old events.
    pub fn init(&mut self) -> Result<(), Error<E>> {
        let product_id = self.read_product_id()?;

        // Most GT911 devices return b"911\0".
        if &product_id[0..3] != b"911" {
            return Err(Error::InvalidProductId(product_id));
        }

        self.clear_status()?;

        Ok(())
    }

    /// Poll the GT911 for a new touch report.
    ///
    /// Returns:
    /// - `Pressed(point)` when a finger is present;
    /// - `Released` when the controller reports a release;
    /// - `NoDataAvailable` when there is no new report.
    pub fn read_touch(
        &mut self,
    ) -> Result<TouchEvent, Error<E>> {
        let register = Self::register_bytes(Register::PointInfo);
        let mut status = [0u8; 1];

        self.i2c
            .write_read(
                self.address,
                &register,
                &mut status,
            )
            .map_err(Error::Bus)?;

        let status = status[0];

        let data_ready = status & 0x80 != 0;
        let touch_count = status & 0x0F;

        if !data_ready {
            return Err(Error::NoDataAvailable);
        }

        let event = if touch_count > 0 {
            let point = self.read_touch_point()?;
            TouchEvent::Pressed(point)
        } else {
            TouchEvent::Released
        };

        // The GT911 will not report another event until this is cleared.
        self.clear_status()?;

        Ok(event)
    }

    fn read_touch_point(
        &mut self,
    ) -> Result<TouchPoint, Error<E>> {
        let register = Self::register_bytes(Register::Point1);
        let mut data = [0u8; 7];

        self.i2c
            .write_read(
                self.address,
                &register,
                &mut data,
            )
            .map_err(Error::Bus)?;

        let id = data[0];

        let raw_x =
            u16::from(data[1]) |
            (u16::from(data[2]) << 8);

        let raw_y =
            u16::from(data[3]) |
            (u16::from(data[4]) << 8);

        let size =
            u16::from(data[5]) |
            (u16::from(data[6]) << 8);

        let (x, y) = self.transform_coordinates(raw_x, raw_y);

        Ok(TouchPoint {
            id,
            x,
            y,
            size,
        })
    }

    fn transform_coordinates(
        &self,
        raw_x: u16,
        raw_y: u16,
    ) -> (u16, u16) {
        let max_x = self.dimensions.width.saturating_sub(1);
        let max_y = self.dimensions.height.saturating_sub(1);

        let raw_x = raw_x.min(max_x);
        let raw_y = raw_y.min(max_y);

        match self.orientation {
            Orientation::Landscape => {
                (raw_x, raw_y)
            }

            Orientation::InvertedLandscape => {
                (
                    max_x.saturating_sub(raw_x),
                    max_y.saturating_sub(raw_y),
                )
            }

            Orientation::Portrait => {
                (
                    raw_y.min(max_x),
                    max_y.saturating_sub(raw_x.min(max_y)),
                )
            }

            Orientation::InvertedPortrait => {
                (
                    max_x.saturating_sub(raw_y.min(max_x)),
                    raw_x.min(max_y),
                )
            }
        }
    }
}