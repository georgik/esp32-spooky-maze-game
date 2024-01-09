#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use spi_dma_displayinterface::spi_dma_displayinterface;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ClockControl, CpuClock},
    dma::DmaPriority,
    gdma::Gdma,
    i2c,
    peripherals::{
        Peripherals,
        Interrupt
    },
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay,
    Rng,
    IO, interrupt
};

use spooky_embedded::app::app_loop;

use spooky_embedded::{
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
    controllers::{accel::AccelMovementController, composites::accel_composite::AccelCompositeController}
};

use esp_backtrace as _;

use icm42670::{Address, Icm42670};
use shared_bus::BusManagerSimple;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 80MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    esp_println::logger::init_logger_from_env();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let lcd_sclk = io.pins.gpio0;
    let lcd_mosi = io.pins.gpio6;
    let lcd_miso = io.pins.gpio11; // random unused pin
    let lcd_cs = io.pins.gpio5;
    let lcd_dc = io.pins.gpio4.into_push_pull_output();
    let _lcd_backlight = io.pins.gpio1.into_push_pull_output();
    let lcd_reset = io.pins.gpio3.into_push_pull_output();

    let i2c_sda = io.pins.gpio10;
    let i2c_scl = io.pins.gpio8;

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let spi = Spi::new(
        peripherals.SPI2,
        40u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    ).with_pins(
        Some(lcd_sclk),
        Some(lcd_mosi),
        Some(lcd_miso),
        Some(lcd_cs),
    ).with_dma(dma_channel.configure(
        false,
        &mut descriptors,
        &mut rx_descriptors,
        DmaPriority::Priority0,
    ));

    println!("SPI ready");

    let di = spi_dma_displayinterface::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);

    let mut display = match mipidsi::Builder::st7789(di)
    .with_display_size(LCD_H_RES, LCD_V_RES)
    .with_orientation(mipidsi::Orientation::Landscape(true))
    .with_color_order(mipidsi::ColorOrder::Rgb)
        .init(&mut delay, Some(lcd_reset)) {
        Ok(display) => display,
        Err(_e) => {
            // Handle the error and possibly exit the application
            panic!("Display initialization failed");
        }
    };

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
        i2c_sda,
        i2c_scl,
        2u32.kHz(), // Set just to 2 kHz, it seems that there is an interference on Rust board
        &clocks,
    );

    println!("I2C ready");

    // let bus = BusManagerSimple::new(i2c);
    let icm = Icm42670::new(i2c, Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    let accel_movement_controller = AccelMovementController::new(icm, 0.2);
    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = AccelCompositeController::new(demo_movement_controller, accel_movement_controller);

    app_loop( &mut display, seed_buffer, movement_controller);
    loop {}

}
