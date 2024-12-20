#![no_std]
#![no_main]

use esp_bsp::prelude::*;
use esp_display_interface_spi_dma::display_interface_spi_dma;

#[allow(unused_imports)]
use esp_backtrace as _;

use esp_hal::rng::Rng;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};
use embedded_hal::delay::DelayNs;
use esp_println::println;

use esp_hal::{
    delay::Delay,
    dma::Dma,
    dma::DmaPriority,
    gpio::{Level, Output},
    i2c::master::I2c,
    prelude::*,
    spi::master::Spi,
};

use spooky_embedded::{
    app::app_loop,
    controllers::{
        accel::AccelMovementController, composites::accel_composite::AccelCompositeController,
    },
    embedded_display::LCD_MEMORY_SIZE,
};

#[cfg(feature = "accelerometer")]
use icm42670::{Address, Icm42670};

use esp_hal::gpio::OutputOpenDrain;
use esp_hal::gpio::Pull;

#[entry]
fn main() -> ! {
    // Initialize peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    #[cfg(not(feature = "no-psram"))]
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    #[cfg(feature = "no-psram")]
    esp_alloc::heap_allocator!(280 * 1024);

    let mut delay = Delay::new();

    println!("Initializing SPI LCD driver");

    // Use the `lcd_i2c_init` macro to initialize I2C for accelerometer
    let i2c = i2c_init!(peripherals);

    // Use the `lcd_spi` macro to initialize the SPI interface
    let spi = lcd_spi!(peripherals);

    println!("SPI ready");

    // Use the `lcd_display_interface` macro to create the display interface
    let di = lcd_display_interface!(peripherals, spi);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    delay.delay_ns(500_000u32);

    let mut display = lcd_display!(peripherals, di).init(&mut delay).unwrap();

    // Use the `lcd_backlight_init` macro to turn on the backlight
    lcd_backlight_init!(peripherals);

    println!("Initializing...");

    // Render an "Initializing..." message on the display
    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    // Initialize the accelerometer
    #[cfg(feature = "accelerometer")]
    let icm = Icm42670::new(i2c, Address::Primary).unwrap();

    // Initialize the random number generator
    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer);

    // Initialize the movement controllers
    #[cfg(feature = "accelerometer")]
    let accel_movement_controller = AccelMovementController::new(icm, 0.2);

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);

    #[cfg(feature = "accelerometer")]
    let movement_controller = AccelCompositeController::new(demo_movement_controller, accel_movement_controller);

    #[cfg(not(feature = "accelerometer"))]
    let movement_controller = demo_movement_controller;

    println!("Entering main loop");

    // Enter the application loop
    app_loop(&mut display, seed_buffer, movement_controller);
}
