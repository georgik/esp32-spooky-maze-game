#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// use display_interface_spi::SPIInterfaceNoCS;
use spi_dma_displayinterface::spi_dma_displayinterface::SPIInterfaceNoCS;

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
    peripherals::Peripherals,
    prelude::*,
    psram,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay,
    Rng,
    IO
};

mod app;
use app::app_loop;

mod accel_movement_controller;
mod s3box_composite_controller;

use esp_backtrace as _;

// #[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
// #[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

fn init_psram_heap() {
    unsafe {
        ALLOCATOR.init(psram::psram_vaddr_start() as *mut u8, psram::PSRAM_BYTES);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    psram::init_psram(peripherals.PSRAM);
    init_psram_heap();

    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let lcd_h_res = 320;
    let lcd_v_res = 240;

    let lcd_sclk = io.pins.gpio7;
    let lcd_mosi = io.pins.gpio6;
    let lcd_cs = io.pins.gpio5;
    let lcd_miso = io.pins.gpio19;
    let lcd_dc = io.pins.gpio4.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio45.into_push_pull_output();
    let lcd_reset = io.pins.gpio48.into_push_pull_output();

    // let i2c_sda = io.pins.gpio8;
    // let i2c_scl = io.pins.gpio18;

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let spi = Spi::new(
        peripherals.SPI2,
        lcd_sclk,
        lcd_mosi,
        lcd_miso,
        lcd_cs,
        40u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    ).with_dma(dma_channel.configure(
        false,
        &mut descriptors,
        &mut rx_descriptors,
        DmaPriority::Priority0,
    ));

    println!("SPI ready");

    let di = SPIInterfaceNoCS::new(spi, lcd_dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);

    let mut display = match mipidsi::Builder::st7789(di)
        .with_display_size(240, 320)
        .with_orientation(mipidsi::Orientation::LandscapeInverted(true))
        .with_color_order(mipidsi::ColorOrder::Rgb)
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(lcd_reset)) {
        Ok(display) => display,
        Err(e) => {
            // Handle the error and possibly exit the application
            panic!("Display initialization failed");
        }
    };

    let _ = lcd_backlight.set_low();

    println!("Initializing...");
        Text::new(
            "Initializing...",
            Point::new(80, 110),
            MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
        )
        .draw(&mut display)
        .unwrap();



    // #[cfg(any(feature = "imu_controls"))]
    // let i2c = i2c::I2C::new(
    //     peripherals.I2C0,
    //     unconfigured_pins.sda,
    //     unconfigured_pins.scl,
    //     100u32.kHz(),
    //     &mut system.peripheral_clock_control,
    //     &clocks,
    // );

    // #[cfg(any(feature = "imu_controls"))]
    // let bus = BusManagerSimple::new(i2c);
    // #[cfg(any(feature = "imu_controls"))]
    // let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();


    // app_loop( &mut display, seed_buffer, icm);
    app_loop( &mut display, lcd_h_res, lcd_v_res, seed_buffer);
    loop {}

}
