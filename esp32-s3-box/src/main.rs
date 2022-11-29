#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    prelude::{RgbColor, PixelIteratorExt},
    mono_font::{
        ascii::{FONT_8X13, FONT_9X18_BOLD},
        MonoTextStyle,
    },
    prelude::{PixelColor, Point, DrawTarget, Size},
    geometry::OriginDimensions,
    text::Text,
    Drawable,
    Pixel
};

use esp_println::println;

#[cfg(feature="esp32")]
use esp32_hal as hal;
#[cfg(feature="esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature="esp32s3")]
use esp32s3_hal as hal;
#[cfg(feature="esp32c3")]
use esp32c3_hal as hal;

use hal::{
    clock::{ ClockControl, CpuClock },
    dma::{DmaPriority},
    gdma::Gdma,
    i2c,
    pac::Peripherals,
    // pac::Peripheral::I2C0,
    prelude::*,
    spi,
    spi::dma::WithDmaSpi2,
    timer::TimerGroup,
    Rng,
    Rtc,
    IO,
    Delay, gpio_types::Unknown,
};

// systimer was introduced in ESP32-S2, it's not available for ESP32
#[cfg(feature="system_timer")]
use hal::systimer::{SystemTimer};

// use panic_halt as _;
use esp_backtrace as _;

#[cfg(feature="xtensa-lx-rt")]
use xtensa_lx_rt::entry;
#[cfg(feature="riscv-rt")]
use riscv_rt::entry;

use embedded_graphics::{image::Image, pixelcolor::Rgb565};
use tinybmp::Bmp;
// use esp32s2_hal::Rng;

#[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg", feature = "esp32s3_box"))]
use mipidsi::{Display, Orientation};
#[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use spooky_core::assets::Assets;
use spooky_core::maze::Maze;

#[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital::v2::OutputPin;
use mipidsi::models::{Model, ILI9342CRgb565};

use heapless::String;
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

pub struct Universe<D, I> {
    pub start_time: u64,
    pub ghost_x: i32,
    pub ghost_y: i32,
    display: D,
    assets: Option<Assets<'static>>,
    step_size_x: u32,
    step_size_y: u32,
    maze: Maze,
    camera_x: i32,
    camera_y: i32,
    // #[cfg(any(feature = "imu_controls"))]
    icm: I,
    // icm: Option<Icm42670<shared_bus::I2cProxy<shared_bus::NullMutex<i2c::I2C<I2C0>>>>>
    // delay: Some(Delay),
}



impl <D:embedded_graphics::draw_target::DrawTarget<Color = Rgb565>, I:Accelerometer> Universe <D, I> {
    pub fn new(display:D, icm:I, seed: Option<[u8; 32]>) -> Universe<D, I> {
        Universe {
            start_time: 0,
            ghost_x: 9*16,
            ghost_y: 7*16,
            display,
            assets: None,
            step_size_x: 16,
            step_size_y: 16,
            maze: Maze::new(64, 64, seed),
            camera_x: 0,
            camera_y: 0,
            // #[cfg(any(feature = "imu_controls"))]
            icm,
            // delay: None,
        }
    }

    fn check_coin_collision(&mut self) {
        let x = self.camera_x + self.ghost_x;
        let y = self.camera_y + self.ghost_y;

        match self.maze.get_coin_at(x, y) {
            Some(coin) => {
                self.maze.remove_coin(coin);
            },
            None => {}
        }
    }

    fn relocate_avatar(&mut self) {
        let (new_camera_x, new_camera_y) = self.maze.get_random_coordinates();
        (self.camera_x, self.camera_y) = (new_camera_x - self.ghost_x, new_camera_y - self.ghost_y);
    }

    fn check_npc_collision(&mut self) {
        let x = self.camera_x + self.ghost_x;
        let y = self.camera_y + self.ghost_y;

        match self.maze.get_npc_at(x, y) {
            Some(_npc) => {
                self.relocate_avatar();
            },
            None => {}
        }
    }

    pub fn move_right(&mut self) {
        let new_camera_x = self.camera_x + self.step_size_x as i32;
        if !self.maze.check_wall_collision(new_camera_x + self.ghost_x, self.camera_y + self.ghost_y) {
            self.camera_x = new_camera_x;
            self.check_coin_collision();
        }
    }

    pub fn move_left(&mut self) {
        let new_camera_x = self.camera_x - self.step_size_x as i32;
        if !self.maze.check_wall_collision(new_camera_x + self.ghost_x, self.camera_y + self.ghost_y) {
            self.camera_x = new_camera_x;
            self.check_coin_collision();
        }
    }

    pub fn move_up(&mut self) {
        let new_camera_y = self.camera_y - self.step_size_y as i32;
        if !self.maze.check_wall_collision(self.camera_x + self.ghost_x, new_camera_y + self.ghost_y) {
            self.camera_y = new_camera_y;
            self.check_coin_collision();
        }
    }

    pub fn move_down(&mut self) {
        let new_camera_y = self.camera_y + self.step_size_y as i32;
        if !self.maze.check_wall_collision(self.camera_x + self.ghost_x, new_camera_y + self.ghost_y) {
            self.camera_y = new_camera_y;
            self.check_coin_collision();
        }
    }

    pub fn draw_maze(&mut self, camera_x: i32, camera_y: i32) {
        println!("Rendering the maze to display");
        #[cfg(feature = "system_timer")]
        let start_timestamp = SystemTimer::now();


                let assets = self.assets.as_ref().unwrap();
                let ground = assets.ground.as_ref().unwrap();
                let wall = assets.wall.as_ref().unwrap();
                let empty = assets.empty.as_ref().unwrap();

                let camera_tile_x = camera_x / self.maze.tile_width as i32;
                let camera_tile_y = camera_y / self.maze.tile_height as i32;
                for x in camera_tile_x..(camera_tile_x + (self.maze.visible_width as i32)-1) {
                    for y in camera_tile_y..(camera_tile_y + (self.maze.visible_height as i32)-1) {
                        let position_x = (x as i32 * self.maze.tile_width as i32) - camera_x;
                        let position_y = (y as i32 * self.maze.tile_height as i32) - camera_y;
                        let position = Point::new(position_x, position_y);

                        if x < 0 || y < 0 || x > (self.maze.width-1) as i32 || y > (self.maze.height-1) as i32 {
                            let tile = Image::new(empty, position);
                            tile.draw(&mut self.display);
                        } else if self.maze.data[(x+y*(self.maze.width as i32)) as usize] == 0 {
                            let tile = Image::new(ground, position);
                            tile.draw(&mut self.display);
                        } else {
                            let tile = Image::new(wall, position);
                            tile.draw(&mut self.display);
                        }
                    }
                }


    }


    pub fn initialize(&mut self) {


        println!("Loading image");
        let mut assets = Assets::new();
        assets.load();
        self.assets = Some(assets);

        self.maze.generate_maze(32, 32);
        self.relocate_avatar();
        self.maze.generate_coins();
        self.maze.generate_npcs();
        self.draw_maze(self.camera_x,self.camera_y);

    }

    pub fn render_frame(&mut self) -> &D {

        self.maze.move_npcs();
        self.check_npc_collision();
        self.draw_maze(self.camera_x,self.camera_y);

        #[cfg(any(feature = "imu_controls"))]
        let accel_threshold = 0.20;

        #[cfg(any(feature = "imu_controls"))]
        {
            let accel_norm = self.icm.accel_norm().unwrap();
            // let gyro_norm = self.icm.gyro_norm().unwrap();
            // println!(
            //     "ACCEL = X: {:+.04} Y: {:+.04} Z: {:+.04}; GYRO  = X: {:+.04} Y: {:+.04} Z: {:+.04}",
            //     accel_norm.x, accel_norm.y, accel_norm.z,
            //     gyro_norm.x, gyro_norm.y, gyro_norm.z
            // );

            if accel_norm.y > accel_threshold {
                self.move_left();
            }

            if accel_norm.y  < -accel_threshold {
                self.move_right();
            }

            if accel_norm.x > accel_threshold {
                self.move_down();
            }

            if accel_norm.x < -accel_threshold {
                self.move_up();
            }
        }


            match self.assets {
                Some(ref mut assets) => {

                    // #[cfg(any(feature = "button_controls"))]
                    // {
                    //     if button_down_pin.is_low().unwrap() {
                    //         if ghost_x > 0 {
                    //             if maze[(ghost_x/TILE_WIDTH)-1+ghost_y] == 0 {
                    //                 ghost_x -= TILE_WIDTH;
                    //             }
                    //         }
                    //     }

                    //     if button_up_pin.is_low().unwrap() {
                    //         if ghost_x < PLAYGROUND_WIDTH {
                    //             if maze[(ghost_x/TILE_WIDTH)+1+ghost_y] == 0 {
                    //                 ghost_x += TILE_WIDTH;
                    //             }
                    //         }
                    //     }

                    //     if button_menu_pin.is_low().unwrap() {
                    //         if ghost_y > 0 {
                    //             if maze[(ghost_x/TILE_WIDTH)+ghost_y-TILE_HEIGHT] == 0 {
                    //                 ghost_y -= TILE_HEIGHT;
                    //             }
                    //         }
                    //     }

                    //     if button_ok_pin.is_low().unwrap() {
                    //         if ghost_y < PLAYGROUND_HEIGHT {
                    //             if maze[(ghost_x/TILE_WIDTH)+ghost_y+TILE_HEIGHT] == 0 {
                    //                 ghost_y += TILE_HEIGHT;
                    //             }
                    //         }
                    //     }
                    // }

                    // if old_x != ghost_x || old_y != ghost_y {
                    //     let ground = Image::new(&ground_bmp, Point::new(old_x.try_into().unwrap(), old_y.try_into().unwrap()));

                    //     ground.draw(&mut display).unwrap();

                    //     let ghost2 = Image::new(&bmp, Point::new(ghost_x.try_into().unwrap(), ghost_y.try_into().unwrap()));

                    //     ghost2.draw(&mut display).unwrap();
                    //     old_x = ghost_x;
                    //     old_y = ghost_y;
                    // }

                    let coin_bmp:Bmp<Rgb565> = assets.coin.unwrap();
                    for index in 0..100 {
                        let coin = self.maze.coins[index];
                        if coin.x < 0 || coin.y < 0 {
                            continue;
                        }

                        let draw_x = coin.x - self.camera_x;
                        let draw_y = coin.y - self.camera_y;
                        if draw_x >= 0 && draw_y >= 0 && draw_x < (self.maze.visible_width*16).try_into().unwrap() && draw_y < (self.maze.visible_height*16).try_into().unwrap() {
                            let position = Point::new(draw_x, draw_y);
                            let tile = Image::new(&coin_bmp, position);
                            tile.draw(&mut self.display);
                        }
                    }

                    let npc_bmp:Bmp<Rgb565> = assets.npc.unwrap();
                    for index in 0..5 {
                        let item = self.maze.npcs[index];
                        if item.x < 0 || item.y < 0 {
                            continue;
                        }

                        let draw_x = item.x - self.camera_x;
                        let draw_y = item.y - self.camera_y;
                        if draw_x >= 0 && draw_y >= 0 && draw_x < (self.maze.visible_width*16).try_into().unwrap() && draw_y < (self.maze.visible_height*16).try_into().unwrap() {
                            let position = Point::new(draw_x, draw_y);
                            let tile = Image::new(&npc_bmp, position);
                            tile.draw(&mut self.display);
                        }
                    }

                    let bmp:Bmp<Rgb565> = assets.ghost1.unwrap();
                    let ghost1 = Image::new(&bmp, Point::new(self.ghost_x.try_into().unwrap(), self.ghost_y.try_into().unwrap()));
                    ghost1.draw(&mut self.display);
                    // display.flush().unwrap();
                },
                None => {
                    println!("No assets");
                }
            };

            let coin_message: String<5> = String::from(self.maze.coin_counter);
            Text::new(&coin_message, Point::new(10, 10), MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE))
                .draw(&mut self.display)
                ;

            // display.flush().unwrap();
            &self.display

        }

    }


#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 65535*4;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }

    let peripherals = Peripherals::take().unwrap();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();
    let mut clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    // let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    #[cfg(feature="esp32c3")]
    rtc.swd.disable();
    #[cfg(feature="xtensa-lx-rt")]
    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);
    // self.delay = Some(delay);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // https://espressif-docs.readthedocs-hosted.com/projects/espressif-esp-dev-kits/en/latest/esp32s3/esp32-s3-usb-otg/user_guide.html
    // let button_up = button::Button::new();

    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_ok_pin = io.pins.gpio0.into_pull_up_input();
    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_menu_pin = io.pins.gpio14.into_pull_up_input();
    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_up_pin = io.pins.gpio10.into_pull_up_input();
    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_down_pin = io.pins.gpio11.into_pull_up_input();

    #[cfg(feature = "esp32")]
    let mut backlight = io.pins.gpio5.into_push_pull_output();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let mut backlight = io.pins.gpio9.into_push_pull_output();
    #[cfg(feature = "esp32c3")]
    let mut backlight = io.pins.gpio0.into_push_pull_output();

    #[cfg(feature = "esp32")]
    let spi = spi::Spi::new(
        peripherals.SPI2,
        io.pins.gpio19,
        io.pins.gpio23,
        io.pins.gpio25,
        io.pins.gpio22,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks);

    #[cfg(any(feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let spi = spi::Spi::new(
        peripherals.SPI3,
        io.pins.gpio6,
        io.pins.gpio7,
        io.pins.gpio12,
        io.pins.gpio5,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks);

    #[cfg(any(feature = "esp32s3_box"))]
    let sclk = io.pins.gpio7;
    #[cfg(any(feature = "esp32s3_box"))]
    let mosi = io.pins.gpio6;

    let dma = Gdma::new(peripherals.DMA, &mut system.peripheral_clock_control);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    #[cfg(any(feature = "esp32s3_box"))]
    let spi = spi::Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        sclk,
        mosi,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );
    // .with_dma(dma_channel.configure(
    //     false,
    //     &mut descriptors,
    //     &mut rx_descriptors,
    //     DmaPriority::Priority0,
    // ));

    #[cfg(any(feature = "esp32s3_box"))]
    let mut backlight = io.pins.gpio45.into_push_pull_output();

    #[cfg(feature = "esp32")]
    backlight.set_low().unwrap();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    backlight.set_high().unwrap();


    #[cfg(feature = "esp32c3")]
    let spi = spi::Spi::new(
        peripherals.SPI2,
        io.pins.gpio6,
        io.pins.gpio7,
        io.pins.gpio12,
        io.pins.gpio20,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks);

    #[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let reset = io.pins.gpio18.into_push_pull_output();
    #[cfg(any(feature = "esp32s3_box"))]
    let reset = io.pins.gpio48.into_push_pull_output();
    #[cfg(any(feature = "esp32c3"))]
    let reset = io.pins.gpio9.into_push_pull_output();

    #[cfg(any(feature = "esp32", feature = "esp32c3"))]
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio21.into_push_pull_output());
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio4.into_push_pull_output());

    #[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
    let mut delay = Delay::new(&clocks);

    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let mut display = mipidsi::Display::st7789(di, reset);

    //https://github.com/espressif/esp-box/blob/master/components/bsp/src/boards/esp32_s3_box.c

    #[cfg(any(feature = "esp32s3_box"))]
    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .init(&mut delay, Some(reset)).unwrap();
    // let mut display = mipidsi::Display::ili9342c_rgb565(di, core::prelude::v1::Some(reset), display_options);
    #[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
    let mut display = Ili9341::new(di, reset, &mut delay, Orientation::Portrait, DisplaySize240x320).unwrap();

    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    display
    .init(
        &mut delay,
        DisplayOptions {
            ..DisplayOptions::default()
        },
    )
    .unwrap();

    // display.clear(RgbColor::WHITE).unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::BLACK),
    )
    .draw(&mut display)
    .unwrap();

    #[cfg(any(feature = "imu_controls"))]
    println!("Initializing IMU");
    #[cfg(any(feature = "imu_controls"))]
    let sda = io.pins.gpio8;
    #[cfg(any(feature = "imu_controls"))]
    let scl = io.pins.gpio18;

    #[cfg(any(feature = "imu_controls"))]
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .unwrap();

    #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);
    #[cfg(any(feature = "imu_controls"))]
    let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8;32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK ; 320*240];
    let mut fbuf = SpriteBuf::new(&mut data, 320, 240);

    let mut universe = Universe::new(fbuf, icm, Some(seed_buffer));
    universe.initialize();


    // #[cfg(any(feature = "imu_controls"))]
    // let accel_threshold = 0.20;

    loop {
        display.draw_iter(universe.render_frame().into_iter()).unwrap();
        // delay.delay_ms(300u32);
    }
}
