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

use hal::{
    clock::{ClockControl, CpuClock},
    i2c,
    peripherals::Peripherals,
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

#[cfg(feature = "mpu6886")]
use mpu6886::Mpu6886;

#[cfg(feature = "xtensa-lx-rt")]
use xtensa_lx_rt::entry;

use embedded_graphics::pixelcolor::Rgb565;

use spooky_core::{engine::Engine, spritebuf::SpriteBuf, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};

#[cfg(any(feature = "i2c"))]
use shared_bus::BusManagerSimple;

use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::OutputPin;

use axp192::{ I2CPowerManagementInterface, Axp192 };

pub struct Universe<D> {
    pub engine: Engine<D>,
}

impl<D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Universe<D> {
    pub fn new(seed: Option<[u8; 32]>, engine: Engine<D>) -> Universe<D> {
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

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    #[cfg(feature = "esp32c3")]
    rtc.swd.disable();
    #[cfg(feature = "xtensa-lx-rt")]
    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // I2C
    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;

    let i2c_bus = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        400u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    #[cfg(any(feature = "i2c"))]
    let bus = BusManagerSimple::new(i2c_bus);

    // Power management - AXP192
    let axp_interface = I2CPowerManagementInterface::new(bus.acquire_i2c());
    let mut axp = Axp192::new(axp_interface);
    axp.init().unwrap();

    // M5Stack CORE 2 - https://docs.m5stack.com/en/core/core2
    #[cfg(feature = "esp32")]
    let mut backlight = io.pins.gpio3.into_push_pull_output();

    #[cfg(feature = "esp32")]
    let spi = spi::Spi::new(
        peripherals.SPI3, // Real HW working with SPI2, but Wokwi seems to work only with SPI3
        io.pins.gpio18,   // SCLK
        io.pins.gpio23,   // MOSI
        io.pins.gpio38,   // MISO
        io.pins.gpio5,   // CS
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    backlight.set_high().unwrap();

    let reset = io.pins.gpio4.into_push_pull_output();
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio15.into_push_pull_output());

    #[cfg(feature = "m5stack_core2")]
    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
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

    // let button_a = io.pins.gpio39.into_pull_up_input();
    #[cfg(feature = "wokwi")]
    let button_b = io.pins.gpio34.into_pull_up_input();
    #[cfg(feature = "wokwi")]
    let button_c = io.pins.gpio35.into_pull_up_input();
    // #[cfg(feature = "m5stack_core2")]
    // let button_b = io.pins.gpio38.into_pull_up_input();
    // #[cfg(feature = "m5stack_core2")]
    // let button_c = io.pins.gpio37.into_pull_up_input();

    #[cfg(any(feature = "mpu9250"))]
    let mut icm = Mpu9250::imu_default(bus.acquire_i2c(), &mut delay).unwrap();

    #[cfg(any(feature = "mpu6050"))]
    let mut icm = Mpu6050::new(bus.acquire_i2c());

    #[cfg(any(feature = "mpu6886"))]
    let mut icm = Mpu6886::new(bus.acquire_i2c());

    #[cfg(any(feature = "mpu6050", feature = "mpu6886"))]
    let is_imu_enabled = match icm.init(&mut delay) {
        Ok(_) => true,
        Err(_) => false,
    };


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
        #[cfg(feature = "m5stack_core2")]
        {
            // if button_c.is_low().unwrap() {
            //     universe.teleport();
            // }

            // if button_b.is_low().unwrap() {
            //     universe.place_dynamite();
            // }

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

        #[cfg(any(feature = "mpu6050", feature = "mpu6886"))]
        if is_imu_enabled {
            #[cfg(feature = "mpu6050")]
            let accel_threshold = 1.00;

            #[cfg(feature = "mpu6886")]
            let accel_threshold = 0.30;

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
