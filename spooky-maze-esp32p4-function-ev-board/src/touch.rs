use bevy_ecs::prelude::*;
use embedded_hal::i2c::I2c;

use spooky_core::{
    events::player::PlayerInputMessage,
    resources::MazeResource,
};

use esp_println::println;

use esp_hal::time::{
    Duration as HalDuration,
    Instant as HalInstant,
};

// -----------------------------------------------------------------------------
// GT911 registers
// -----------------------------------------------------------------------------

const GT911_ADDRESS_PRIMARY: u8 = 0x5D;
const GT911_ADDRESS_SECONDARY: u8 = 0x14;

const GT911_PRODUCT_ID_REGISTER: u16 = 0x8140;
const GT911_STATUS_REGISTER: u16 = 0x814E;
const GT911_FIRST_POINT_REGISTER: u16 = 0x814F;

const GT911_DATA_READY: u8 = 0x80;
const GT911_TOUCH_COUNT_MASK: u8 = 0x0F;

// Espressif's BSP mirrors both axes for this display.
const TOUCH_MIRROR_X: bool = true;
const TOUCH_MIRROR_Y: bool = true;

const SCREEN_WIDTH: u16 = 1024;
const SCREEN_HEIGHT: u16 = 600;

// -----------------------------------------------------------------------------
// On-screen D-pad geometry
// -----------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ButtonRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ButtonRect {
    pub const fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width as i32
            && y < self.y + self.height as i32
    }
}

pub const UP_BUTTON: ButtonRect =
    ButtonRect::new(120, 310, 80, 80);

pub const LEFT_BUTTON: ButtonRect =
    ButtonRect::new(30, 400, 80, 80);

pub const RIGHT_BUTTON: ButtonRect =
    ButtonRect::new(210, 400, 80, 80);

pub const DOWN_BUTTON: ButtonRect =
    ButtonRect::new(120, 490, 80, 80);

// -----------------------------------------------------------------------------
// Touch and direction types
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum TouchEvent {
    /// The GT911 has no new information.
    NoUpdate,

    /// All fingers were released.
    Released,

    /// A new or updated touch position is available.
    Point(TouchPoint),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}



// Wait this long before movement begins repeating.
const INITIAL_REPEAT_DELAY_MS: u64 = 70;

// Time between repeated movements while held.
const REPEAT_INTERVAL_MS: u64 = 60;

#[derive(Resource, Default)]
pub struct TouchInputState {
    /// Direction currently held by the finger.
    pub pressed: Option<Direction>,

    /// An immediate movement waiting to be sent.
    pending: Option<Direction>,

    /// Time at which the next held movement should occur.
    next_repeat: Option<HalInstant>,
}

impl TouchInputState {
    pub fn update_pressed(
        &mut self,
        new_direction: Option<Direction>,
    ) {
        // Receiving another touch update for the same button must not
        // restart the repeat timer.
        if new_direction == self.pressed {
            return;
        }

        self.pressed = new_direction;

        match new_direction {
            Some(direction) => {
                // Move immediately on the initial press.
                self.pending = Some(direction);

                // Begin repeating after a short initial delay.
                self.next_repeat = Some(
                    HalInstant::now()
                        + HalDuration::from_millis(
                            INITIAL_REPEAT_DELAY_MS,
                        ),
                );
            }

            None => {
                // Stop immediately when the finger is released or moved
                // outside all direction buttons.
                self.pending = None;
                self.next_repeat = None;
            }
        }
    }

    fn next_direction(
        &mut self,
    ) -> Option<Direction> {
        // A new press or direction change moves immediately.
        if let Some(direction) = self.pending.take() {
            return Some(direction);
        }

        // Nothing is currently held.
        let direction = self.pressed?;

        // A held button should always have a repeat deadline.
        let next_repeat = self.next_repeat?;

        let now = HalInstant::now();

        if now < next_repeat {
            return None;
        }

        // Schedule the next repetition relative to the current time.
        //
        // Using the current time avoids producing many catch-up messages
        // after an unusually slow frame.
        self.next_repeat = Some(
            now + HalDuration::from_millis(
                REPEAT_INTERVAL_MS,
            ),
        );

        Some(direction)
    }
}

// -----------------------------------------------------------------------------
// GT911 driver
// -----------------------------------------------------------------------------

pub struct Gt911<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Gt911<I2C>
where
    I2C: I2c,
{
    pub fn new(
        mut i2c: I2C,
    ) -> Result<(Self, [u8; 4]), I2C::Error> {
        let mut product_id = [0u8; 4];

        if Self::read_register_from(
            &mut i2c,
            GT911_ADDRESS_PRIMARY,
            GT911_PRODUCT_ID_REGISTER,
            &mut product_id,
        )
        .is_ok()
        {
            return Ok((
                Self {
                    i2c,
                    address: GT911_ADDRESS_PRIMARY,
                },
                product_id,
            ));
        }

        Self::read_register_from(
            &mut i2c,
            GT911_ADDRESS_SECONDARY,
            GT911_PRODUCT_ID_REGISTER,
            &mut product_id,
        )?;

        Ok((
            Self {
                i2c,
                address: GT911_ADDRESS_SECONDARY,
            },
            product_id,
        ))
    }

    pub fn address(&self) -> u8 {
        self.address
    }

    pub fn poll_event(
        &mut self,
    ) -> Result<TouchEvent, I2C::Error> {
        let mut status = [0u8; 1];

        self.read_register(
            GT911_STATUS_REGISTER,
            &mut status,
        )?;

        // If bit 7 is clear, the GT911 has no new information.
        if status[0] & GT911_DATA_READY == 0 {
            return Ok(TouchEvent::NoUpdate);
        }

        let touch_count =
            status[0] & GT911_TOUCH_COUNT_MASK;

        if touch_count == 0 {
            self.clear_status()?;
            return Ok(TouchEvent::Released);
        }

        // Each GT911 touch record is eight bytes:
        //
        // byte 0       tracking ID
        // bytes 1..=2  X coordinate
        // bytes 3..=4  Y coordinate
        // bytes 5..=6  touch size
        // byte 7       reserved
        let mut point_data = [0u8; 8];

        self.read_register(
            GT911_FIRST_POINT_REGISTER,
            &mut point_data,
        )?;

        // The controller expects the status register to be cleared
        // after the point data has been consumed.
        self.clear_status()?;

        let x = u16::from_le_bytes([
            point_data[1],
            point_data[2],
        ]);

        let y = u16::from_le_bytes([
            point_data[3],
            point_data[4],
        ]);

        let size = u16::from_le_bytes([
            point_data[5],
            point_data[6],
        ]);

        Ok(TouchEvent::Point(TouchPoint {
            x,
            y,
            size,
        }))
    }

    fn clear_status(
        &mut self,
    ) -> Result<(), I2C::Error> {
        self.write_u8(GT911_STATUS_REGISTER, 0)
    }

    fn read_register(
        &mut self,
        register: u16,
        output: &mut [u8],
    ) -> Result<(), I2C::Error> {
        Self::read_register_from(
            &mut self.i2c,
            self.address,
            register,
            output,
        )
    }

    fn read_register_from(
        i2c: &mut I2C,
        address: u8,
        register: u16,
        output: &mut [u8],
    ) -> Result<(), I2C::Error> {
        let register_bytes = register.to_be_bytes();

        i2c.write_read(
            address,
            &register_bytes,
            output,
        )
    }

    fn write_u8(
        &mut self,
        register: u16,
        value: u8,
    ) -> Result<(), I2C::Error> {
        let register_bytes = register.to_be_bytes();

        let data = [
            register_bytes[0],
            register_bytes[1],
            value,
        ];

        self.i2c.write(self.address, &data)
    }
}

// -----------------------------------------------------------------------------
// Coordinate conversion and button hit testing
// -----------------------------------------------------------------------------

pub fn transform_touch_point(
    point: TouchPoint,
) -> (i32, i32) {
    let raw_x = point.x.min(SCREEN_WIDTH - 1);
    let raw_y = point.y.min(SCREEN_HEIGHT - 1);

    let x = if TOUCH_MIRROR_X {
        SCREEN_WIDTH - 1 - raw_x
    } else {
        raw_x
    };

    let y = if TOUCH_MIRROR_Y {
        SCREEN_HEIGHT - 1 - raw_y
    } else {
        raw_y
    };

    (x as i32, y as i32)
}

pub fn direction_at(
    x: i32,
    y: i32,
) -> Option<Direction> {
    if UP_BUTTON.contains(x, y) {
        Some(Direction::Up)
    } else if DOWN_BUTTON.contains(x, y) {
        Some(Direction::Down)
    } else if LEFT_BUTTON.contains(x, y) {
        Some(Direction::Left)
    } else if RIGHT_BUTTON.contains(x, y) {
        Some(Direction::Right)
    } else {
        None
    }
}

// -----------------------------------------------------------------------------
// Bevy input system
// -----------------------------------------------------------------------------

/// Converts a newly pressed on-screen button into one tile of movement.
///
/// Holding a button does not emit movement every frame. The player must
/// release it and press it again. Dragging directly to another direction
/// also produces one movement in the new direction.
/// Converts touchscreen direction input into player movement.
///
/// A new press moves immediately. Holding the button produces repeated
/// movement after the configured initial delay.
pub fn dispatch_touch_input(
    mut touch_state: ResMut<TouchInputState>,
    maze_res: Res<MazeResource>,
    mut message_writer: MessageWriter<PlayerInputMessage>,
) {
    let Some(direction) =
        touch_state.next_direction()
    else {
        return;
    };

    let step_x =
        maze_res.maze.tile_width as f32;

    let step_y =
        maze_res.maze.tile_height as f32;

    let (dx, dy) = match direction {
        Direction::Up => {
            (0.0, -step_y)
        }

        Direction::Down => {
            (0.0, step_y)
        }

        Direction::Left => {
            (-step_x, 0.0)
        }

        Direction::Right => {
            (step_x, 0.0)
        }
    };

    message_writer.write(
        PlayerInputMessage {
            dx,
            dy,
        },
    );
}