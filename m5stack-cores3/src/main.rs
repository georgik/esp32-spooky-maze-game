#![no_std]
#![no_main]

use esp_display_interface_spi_dma::display_interface_spi_dma;
use aw9523::I2CGpioExpanderInterface;
use axp2101::{Axp2101, I2CPowerManagementInterface};

use esp_bsp::prelude::*;

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

use log::info;
use mipidsi::options::ColorInversion;
use shared_bus::BusManagerSimple;
use spooky_embedded::{
    app::app_loop,
    embedded_display::LCD_MEMORY_SIZE,
};

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let mut delay = Delay::new();

    println!("Initializing M5Stack CoreS3");

    // Initialize I2C shared bus
    let bus = BusManagerSimple::new(i2c_init!(peripherals));

    // Initialize AXP2101 power management
    info!("Initializing AXP2101");
    let axp_interface = I2CPowerManagementInterface::new(bus.acquire_i2c());
    let mut axp = Axp2101::new(axp_interface);
    axp.init().unwrap();

    // Initialize AW9523 GPIO expander
    info!("Initializing AW9523");
    let aw_interface = I2CGpioExpanderInterface::new(bus.acquire_i2c());
    let mut aw = aw9523::Aw9523::new(aw_interface);
    aw.init().unwrap();


    println!("Initializing SPI LCD driver for ESP32S3Box");

    // Use the `lcd_i2c_init` macro to initialize I2C for accelerometer
    // let i2c = i2c_init!(peripherals);

    // Use the `lcd_spi` macro to initialize the SPI interface
    let spi = lcd_spi!(peripherals);

    println!("SPI ready");

    // Use the `lcd_display_interface` macro to create the display interface
    let di = lcd_display_interface!(peripherals, spi);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    delay.delay_ns(500_000u32);

    let mut display = lcd_display!(peripherals, di)
        .init(&mut delay)
        .unwrap();

    println!("Initializing display...");
    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
        .draw(&mut display)
        .unwrap();

    // Initialize RNG
    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer);

    // Create the movement controller
    let demo_movement_controller =
        spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);

    println!("Entering main loop");
    app_loop(&mut display, seed_buffer, demo_movement_controller);
}
