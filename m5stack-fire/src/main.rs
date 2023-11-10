#![no_std]
#![no_main]

// https://docs.makerfactory.io/m5stack/core/fire/

// use display_interface_spi::SPIInterfaceNoCS;
use spi_dma_displayinterface::spi_dma_displayinterface::SPIInterfaceNoCS;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{DrawTarget, Point, RgbColor},
    text::Text,
    Drawable,
};

use hal::{
    clock::{ClockControl, CpuClock},
    i2c::I2C,
    peripherals::Peripherals,
    dma::DmaPriority,
    pdma::Dma,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay, Rng, IO,
};

// use panic_halt as _;
use esp_backtrace as _;

#[cfg(feature = "mpu9250")]
use mpu9250::{ImuMeasurements, Mpu9250};

#[cfg(feature = "mpu6050")]
use mpu6050::Mpu6050;

use embedded_graphics::pixelcolor::Rgb565;

use spooky_core::{engine::Engine, spritebuf::SpriteBuf, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};

#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::OutputPin;

pub struct Universe<D> {
    pub engine: Engine<D>,
}

impl<D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Universe<D> {
    pub fn new(_seed: Option<[u8; 32]>, engine: Engine<D>) -> Universe<D> {
        Universe { engine }
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
        self.engine.start()
    }

    pub fn move_up(&mut self) {
        self.engine.action(Up);
    }

    pub fn move_down(&mut self) {
        self.engine.action(Down);
    }

    pub fn move_left(&mut self) {
        self.engine.action(Left);
    }

    pub fn move_right(&mut self) {
        self.engine.action(Right);
    }

    pub fn teleport(&mut self) {
        self.engine.action(Teleport)
    }

    pub fn place_dynamite(&mut self) {
        self.engine.action(PlaceDynamite);
    }

    pub fn render_frame(&mut self) -> &D {
        self.engine.tick();
        self.engine.draw()
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 160MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let lcd_h_res = 240;
    let lcd_v_res = 320;

    let lcd_sclk = io.pins.gpio18;
    let lcd_mosi = io.pins.gpio23;
    let lcd_miso = io.pins.gpio19;
    let lcd_cs = io.pins.gpio14;
    let lcd_dc = io.pins.gpio27.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio32.into_push_pull_output();
    let lcd_reset = io.pins.gpio33.into_push_pull_output();

    let dma = Dma::new(system.dma);
    let dma_channel = dma.spi2channel;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let spi = Spi::new(
        peripherals.SPI2,
        lcd_sclk,
        lcd_mosi,
        lcd_miso,
        lcd_cs,
        60u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    ).with_dma(dma_channel.configure(
        false,
        &mut descriptors,
        &mut rx_descriptors,
        DmaPriority::Priority0,
    ));

    lcd_backlight.set_high().unwrap();

    let di = SPIInterfaceNoCS::new(spi, lcd_dc);

    #[cfg(feature = "m5stack_fire")]
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(lcd_reset))
        .unwrap();

    #[cfg(feature = "wokwi")]
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::Landscape(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
        .unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    // let button_a = io.pins.gpio39.into_pull_up_input();
    #[cfg(feature = "wokwi")]
    let button_b = io.pins.gpio34.into_pull_up_input();
    #[cfg(feature = "wokwi")]
    let button_c = io.pins.gpio35.into_pull_up_input();
    #[cfg(feature = "m5stack_fire")]
    let button_b = io.pins.gpio38.into_pull_up_input();
    #[cfg(feature = "m5stack_fire")]
    let button_c = io.pins.gpio37.into_pull_up_input();

    #[cfg(any(feature = "imu_controls"))]
    let sda = io.pins.gpio21;
    #[cfg(any(feature = "imu_controls"))]
    let scl = io.pins.gpio22;

    #[cfg(any(feature = "imu_controls"))]
    let i2c = I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &clocks,
    );

    #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);

    #[cfg(any(feature = "mpu9250"))]
    let mut icm = Mpu9250::imu_default(bus.acquire_i2c(), &mut delay).unwrap();

    #[cfg(any(feature = "mpu6050"))]
    let mut icm = Mpu6050::new(bus.acquire_i2c());
    #[cfg(any(feature = "mpu6050"))]
    icm.init(&mut delay).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    let engine = Engine::new(spritebuf, Some(seed_buffer));
    let mut universe = Universe::new(Some(seed_buffer), engine);
    universe.initialize();

    loop {
        #[cfg(feature = "m5stack_fire")]
        {
            if button_c.is_low().unwrap() {
                universe.teleport();
            }

            if button_b.is_low().unwrap() {
                universe.place_dynamite();
            }

        }

        #[cfg(feature = "wokwi")]
        {
            if button_c.is_high().unwrap() {
                universe.teleport();
            }

            if button_b.is_high().unwrap() {
                universe.place_dynamite();
            }

        }

        #[cfg(feature = "mpu9250")]
        {
            let accel_threshold = 1.00;
            let measurement: ImuMeasurements<[f32; 3]> = icm.all().unwrap();

            if measurement.accel[0] > accel_threshold {
                universe.move_left();
            }

            if measurement.accel[0] < -accel_threshold {
                universe.move_right();
            }

            if measurement.accel[1] > accel_threshold {
                universe.move_down();
            }

            if measurement.accel[1] < -accel_threshold {
                universe.move_up();
            }

            // Quickly move up to teleport
            // Quickly move down to place dynamite
            if measurement.accel[2] < -10.2 {
                universe.teleport();
            } else if measurement.accel[2] > 20.5 {
                universe.place_dynamite();
            }
        }

        #[cfg(feature = "mpu6050")]
        {
            let accel_threshold = 1.00;
            let measurement = icm.get_acc().unwrap();
            // let measurement: ImuMeasurements<[f32; 3]> = icm.all().unwrap();

            if measurement.x > accel_threshold {
                universe.move_left();
            }

            if measurement.x < -accel_threshold {
                universe.move_right();
            }

            if measurement.y > accel_threshold {
                universe.move_down();
            }

            if measurement.y < -accel_threshold {
                universe.move_up();
            }

            // Quickly move up to teleport
            // Quickly move down to place dynamite
            if measurement.z < -10.2 {
                universe.teleport();
            } else if measurement.z > 20.5 {
                universe.place_dynamite();
            }
        }
        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0, 0, lcd_v_res-1, lcd_h_res, pixel_iterator);
        // delay.delay_ms(300u32);
    }
}
