#![no_std]
#![no_main]

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

use icm42670::{Address, Icm42670};

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let mut delay = Delay::new();

    println!("About to initialize the SPI LED driver");

    let lcd_sclk = peripherals.GPIO7;
    let lcd_mosi = peripherals.GPIO6;
    let lcd_cs = peripherals.GPIO5;
    let lcd_miso = peripherals.GPIO2; // random unused pin
    let lcd_dc = Output::new(peripherals.GPIO4, Level::Low);
    let mut lcd_backlight = Output::new(peripherals.GPIO45, Level::Low);
    let lcd_reset = Output::new(peripherals.GPIO48, Level::Low);

    let i2c_sda = peripherals.GPIO8;
    let i2c_scl = peripherals.GPIO18;
    let i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
        .with_sda(i2c_sda)
        .with_scl(i2c_scl);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let spi = Spi::new_with_config(
        peripherals.SPI2,
        esp_hal::spi::master::Config {
            frequency: 40u32.MHz(),
            ..esp_hal::spi::master::Config::default()
        },
    )
    .with_sck(lcd_sclk)
    .with_mosi(lcd_mosi)
    .with_miso(lcd_miso)
    .with_cs(lcd_cs)
    .with_dma(dma_channel.configure(false, DmaPriority::Priority0));

    println!("SPI ready");

    let di = display_interface_spi_dma::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ns(500_000u32);

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

    lcd_backlight.set_high();

    println!("Initializing...");
    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    let icm = Icm42670::new(i2c, Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer);

    let accel_movement_controller = AccelMovementController::new(icm, 0.2);
    let demo_movement_controller =
        spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller =
        AccelCompositeController::new(demo_movement_controller, accel_movement_controller);

    println!("Entering main loop");
    app_loop(&mut display, seed_buffer, movement_controller);
}
