#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

// https://docs.makerfactory.io/m5stack/core/fire/

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{DrawTarget, Point, RgbColor},
    text::Text,
    Drawable,
};

#[cfg(feature = "esp32")]
use esp32_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal as hal;
#[cfg(feature = "esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature = "esp32s3")]
use esp32s3_hal as hal;

use hal::{
    clock::{ClockControl, CpuClock},
    i2c,
    pac::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Delay, Rng, Rtc, IO,
};

// use panic_halt as _;
use esp_backtrace as _;

#[cfg(feature = "wokwi")]
use mipidsi::hal::{ Orientation, Rotation };

#[cfg(feature = "mpu9250")]
use mpu9250::{ImuMeasurements, Mpu9250};

#[cfg(feature = "mpu6050")]
use mpu6050::Mpu6050;

#[cfg(feature = "xtensa-lx-rt")]
use xtensa_lx_rt::entry;

use embedded_graphics::pixelcolor::Rgb565;

use spooky_core::{engine::Engine, spritebuf::SpriteBuf};

#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::OutputPin;

pub struct Universe<D> {
    pub engine: Engine<D>,
}

impl<D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Universe<D> {
    pub fn new(seed: Option<[u8; 32]>, engine: Engine<D>) -> Universe<D> {
        Universe { engine }
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
    }

    pub fn move_up(&mut self) {
        self.engine.move_up();
    }

    pub fn move_down(&mut self) {
        self.engine.move_down();
    }

    pub fn move_left(&mut self) {
        self.engine.move_left();
    }

    pub fn move_right(&mut self) {
        self.engine.move_right();
    }

    pub fn teleport(&mut self) {
        self.engine.teleport();
    }

    pub fn place_dynamite(&mut self) {
        self.engine.place_dynamite();
    }

    pub fn render_frame(&mut self) -> &D {
        self.engine.tick();
        self.engine.draw()
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    #[cfg(feature = "esp32c3")]
    rtc.swd.disable();
    #[cfg(feature = "xtensa-lx-rt")]
    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    #[cfg(feature = "esp32")]
    let mut backlight = io.pins.gpio32.into_push_pull_output();

    #[cfg(feature = "esp32")]
    let spi = spi::Spi::new(
        peripherals.SPI3, // Real HW working with SPI2, but Wokwi seems to work only with SPI3
        io.pins.gpio18,   // SCLK
        io.pins.gpio23,   // MOSI
        io.pins.gpio19,   // MISO
        io.pins.gpio14,   // CS
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    backlight.set_high().unwrap();

    let reset = io.pins.gpio33.into_push_pull_output();
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio27.into_push_pull_output());

    #[cfg(feature = "m5stack_core2")]
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .init(&mut delay, Some(reset))
        .unwrap();

    #[cfg(feature = "wokwi")]
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(Orientation::new().rotate(Rotation::Deg90).flip_vertical())
        .init(&mut delay, Some(reset))
        .unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    #[cfg(any(feature = "imu_controls"))]
    let sda = io.pins.gpio21;
    #[cfg(any(feature = "imu_controls"))]
    let scl = io.pins.gpio22;

    #[cfg(any(feature = "imu_controls"))]
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
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

        display
            .draw_iter(universe.render_frame().into_iter())
            .unwrap();
        // delay.delay_ms(300u32);
    }
}
