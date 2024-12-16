#![no_std]
#![no_main]

use esp_display_interface_spi_dma::display_interface_spi_dma;
use esp_bsp::{BoardType, DisplayConfig};
use esp_bsp::boards::esp32s3box::{lcd_spi, lcd_display_interface, lcd_reset_pin, lcd_backlight_init};

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
    peripherals::Peripherals,
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

use icm42670::{Address, Icm42670};

#[entry]
fn main() -> ! {
    // Initialize peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let mut delay = Delay::new();

    println!("Initializing SPI LCD driver for ESP32S3Box");

    // Initialize I2C for accelerometer
    let i2c_sda = peripherals.GPIO8;
    let i2c_scl = peripherals.GPIO18;
    let i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
        .with_sda(i2c_sda)
        .with_scl(i2c_scl);

    // Initialize DMA for SPI
    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    // Use the `lcd_spi` macro to initialize the SPI interface
    let spi = lcd_spi!(peripherals, dma_channel);

    println!("SPI ready");

    // Use the `lcd_display_interface` macro to create the display interface
    let di = lcd_display_interface!(peripherals, spi);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    delay.delay_ns(500_000u32);

    // Use the `lcd_reset_pin` macro to set the reset pin
    let lcd_reset = lcd_reset_pin!(peripherals);

    // Initialize the display using the builder pattern
    let mut display = mipidsi::Builder::new(mipidsi::models::ILI9341Rgb565, di)
        .display_size(240, 320)
        .orientation(
            mipidsi::options::Orientation::new()
                .flip_vertical()
                .flip_horizontal(),
        )
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .reset_pin(lcd_reset)
        .init(&mut delay)
        .unwrap();

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
    let icm = Icm42670::new(i2c, Address::Primary).unwrap();

    // Initialize the random number generator
    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer);

    // Initialize the movement controllers
    let accel_movement_controller = AccelMovementController::new(icm, 0.2);
    let demo_movement_controller =
        spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller =
        AccelCompositeController::new(demo_movement_controller, accel_movement_controller);

    println!("Entering main loop");

    // Enter the application loop
    app_loop(&mut display, seed_buffer, movement_controller);
}
