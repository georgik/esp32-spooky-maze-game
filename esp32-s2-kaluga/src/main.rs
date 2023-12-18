#![no_std]
#![no_main]

// Main board: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s2/hw-reference/esp32s2/user-guide-esp32-s2-kaluga-1-kit.html
// Buttons - Lyra extension board: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s2/hw-reference/esp32s2/user-guide-esp-lyrat-8311a_v1.3.html

use spi_dma_displayinterface::spi_dma_displayinterface;

use embedded_graphics::{
    prelude::{Point, RgbColor},
    mono_font::{
        ascii::FONT_8X13,
        MonoTextStyle,
    },
    text::Text,
    Drawable,
};

use hal::{
    clock::{ ClockControl, CpuClock },
    dma::DmaPriority,
    pdma::Dma,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Rng,
    IO,
    Delay,
    adc::{AdcConfig, Attenuation, ADC, ADC1},
};
use log::info;

use spooky_embedded::{
    app::app_loop,
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
    controllers::{
        ladder::LadderMovementController,
        composites::ladder_composite::LadderCompositeController,
    }
};

use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 160MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    esp_println::logger::init_logger_from_env();

    info!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Backlight is on GPIO6 in version 1.2, version 1.3 has display always on
    // let mut backlight = io.pins.gpio6.into_push_pull_output();
    // backlight.set_high().unwrap();

    let lcd_sclk = io.pins.gpio15;
    let lcd_mosi = io.pins.gpio9;
    let lcd_miso = io.pins.gpio8;
    let lcd_cs = io.pins.gpio11;
    let lcd_dc = io.pins.gpio13.into_push_pull_output();
    let _lcd_backlight = io.pins.gpio5.into_push_pull_output();
    let lcd_reset = io.pins.gpio16.into_push_pull_output();

    let adc_pin = io.pins.gpio6.into_analog();

    let dma = Dma::new(system.dma);
    let dma_channel = dma.spi2channel;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let spi = Spi::new(
        peripherals.SPI2,
        60u32.MHz(),
        SpiMode::Mode0,
        &clocks
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

    let di = spi_dma_displayinterface::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

    let mut delay = Delay::new(&clocks);
    delay.delay_ms(500u32);

    info!("Initializing display");
    let mut display = match mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(LCD_H_RES, LCD_V_RES)
        .with_orientation(mipidsi::Orientation::Landscape(true))
        .with_color_order(mipidsi::ColorOrder::Rgb)
        .init(&mut delay, Some(lcd_reset)) {
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
    let mut seed_buffer = [0u8;32];
    rng.read(&mut seed_buffer).unwrap();

    info!("Initializing the ADC");
    let mut adc1_config = AdcConfig::new();
    let adc_pin = adc1_config.enable_pin(adc_pin, Attenuation::Attenuation11dB);

    let analog = peripherals.SENS.split();
    let adc1 = ADC::<ADC1>::adc(analog.adc1, adc1_config).unwrap();

    info!("Entering main loop");

    let ladder_movement_controller = LadderMovementController::new(adc1, adc_pin);
    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = LadderCompositeController::new(demo_movement_controller, ladder_movement_controller);

    app_loop( &mut display, seed_buffer, movement_controller);
    loop {}
}
