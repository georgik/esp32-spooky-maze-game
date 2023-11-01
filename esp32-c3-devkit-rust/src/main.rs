#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ClockControl, CpuClock},
    // gdma::Gdma,
    i2c,
    peripherals::Peripherals,
    prelude::*,
    spi::{master::Spi, SpiMode},
    Delay,
    Rng,
    IO
};

mod app;
use app::app_loop;

mod accel_movement_controller;
mod s3box_composite_controller;
mod setup;
use setup::setup_pins;

mod types;

use esp_backtrace as _;

// #[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
// #[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let (uninitialized_pins, /*configured_pins, */configured_system_pins) = setup_pins(io.pins);
    println!("SPI LED driver initialized");

    let spi = Spi::new(
        peripherals.SPI2,
        uninitialized_pins.sclk,
        uninitialized_pins.mosi,
        uninitialized_pins.miso,
        uninitialized_pins.cs,
        60u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    );

    println!("SPI ready");

    let di = SPIInterfaceNoCS::new(spi, configured_system_pins.dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);

    let mut display = match mipidsi::Builder::st7789(di)
    .with_display_size(240 as u16, 320 as u16)
    .with_orientation(mipidsi::Orientation::Landscape(true))
    .with_color_order(mipidsi::ColorOrder::Rgb)
        .init(&mut delay, Some(configured_system_pins.reset)) {
        Ok(display) => display,
        Err(_e) => {
            // Handle the error and possibly exit the application
            panic!("Display initialization failed");
        }
    };

    // configured_system_pins.backlight.set_high();

    println!("Initializing...");
        Text::new(
            "Initializing...",
            Point::new(80, 110),
            MonoTextStyle::new(&FONT_8X13, RgbColor::GREEN),
        )
        .draw(&mut display)
        .unwrap();

    println!("Initialized");
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        uninitialized_pins.sda,
        uninitialized_pins.scl,
        100u32.kHz(),
        &clocks,
    );
    println!("I2C ready");

    // #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);
    // #[cfg(any(feature = "imu_controls"))]
    let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();


    app_loop( &mut display, seed_buffer, icm);
    loop {}

}
