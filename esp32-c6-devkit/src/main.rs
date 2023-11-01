#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{DrawTarget, Point, RgbColor},
    text::Text,
    Drawable,
};

use log::info;

use hal::{
    clock::{ClockControl, CpuClock},
    // gdma::Gdma,
    peripherals::Peripherals,
    prelude::*,
    spi::{master::Spi, SpiMode},
    Delay,
    Rng,
    IO,
    adc::{AdcConfig, Attenuation, ADC, ADC1},
};


mod app;
use app::app_loop;

mod devkitc6_composite_controller;
mod ladder_movement_controller;

mod setup;
use setup::*;

mod types;

// systimer was introduced in ESP32-S2, it's not available for ESP32
#[cfg(feature = "system_timer")]
use hal::systimer::SystemTimer;

// use panic_halt as _;
use esp_backtrace as _;

use embedded_graphics::pixelcolor::Rgb565;


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let mut system = peripherals.SYSTEM.split();
    let mut clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    info!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let (uninitialized_pins, configured_pins, configured_system_pins) = setup_pins(io.pins);

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

    let di = SPIInterfaceNoCS::new(spi, configured_system_pins.dc);

    let mut delay = Delay::new(&clocks);

//     let mut display = mipidsi::Builder::ili9341_rgb565(di)
//     .with_display_size(240 as u16, 320 as u16)
//     // .with_framebuffer_size(240 as u16, 320 as u16)
//     .with_orientation(mipidsi::Orientation::Landscape(true))
//     .with_color_order(mipidsi::ColorOrder::Rgb)
//     .init(&mut delay, Some(configured_system_pins.reset)) {
//         Ok(disp) => { disp },
//         Err(_) => { panic!() },
// };

    let mut display = match mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 320 as u16)
        .with_orientation(mipidsi::Orientation::Landscape(true))
        .with_color_order(mipidsi::ColorOrder::Rgb)
        .init(&mut delay, Some(configured_system_pins.reset)) {
            Ok(disp) => { disp },
            Err(_) => { panic!() },
    };

    info!("Display initialized");

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::BLACK),
    )
    .draw(&mut display)
    .unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [1u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    info!("Initializing the ADC");
    let mut adc1_config = AdcConfig::new();
    let adc_pin = adc1_config.enable_pin(configured_pins.adc_pin, Attenuation::Attenuation11dB);

    let analog = peripherals.APB_SARADC.split();
    let adc1 = ADC::<ADC1>::adc(analog.adc1, adc1_config).unwrap();

    info!("Entering main loop");
    app_loop(adc1, adc_pin, &mut display, seed_buffer);
    loop {}
}
