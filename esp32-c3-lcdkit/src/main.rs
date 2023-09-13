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
    spi,
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
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // https://docs.espressif.com/projects/espressif-esp-dev-kits/en/latest/esp32c3/esp32-c3-lcdkit/user_guide.html#gpio-allocation
    let (unconfigured_pins, /*configured_pins, */mut configured_system_pins) = setup_pins(io.pins);
    println!("SPI LED driver initialized");
    let spi = spi::Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        unconfigured_pins.sclk,
        unconfigured_pins.mosi,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    println!("SPI ready");

    let di = SPIInterfaceNoCS::new(spi, configured_system_pins.dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);

    let mut display = match mipidsi::Builder::gc9a01(di)
    .with_display_size(240 as u16, 320 as u16)
    .with_orientation(mipidsi::Orientation::Landscape(true))
    .with_color_order(mipidsi::ColorOrder::Rgb)
        .init(&mut delay, Some(configured_system_pins.reset)) {
        Ok(display) => display,
        Err(e) => {
            // Handle the error and possibly exit the application
            panic!("Display initialization failed");
        }
    };

    // configured_system_pins.backlight.set_high();

    println!("Initializing...");
        Text::new(
            "Initializing...",
            Point::new(80, 110),
            MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
        )
        .draw(&mut display)
        .unwrap();



    // #[cfg(any(feature = "imu_controls"))]
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        unconfigured_pins.sda,
        unconfigured_pins.scl,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

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
