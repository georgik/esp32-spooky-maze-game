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

use log::info;

use hal::{
    clock::{ClockControl, CpuClock},
    dma::DmaPriority,
    gdma::Gdma,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay,
    Rng,
    IO,
    adc::{AdcConfig, Attenuation, ADC, ADC1},
};

use spooky_embedded::{
    app::app_loop,
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
    controllers::{
        composites::ladder_composite::LadderCompositeController,
        ladder::LadderMovementController
    },
};

use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 80MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();

    esp_println::logger::init_logger_from_env();

    info!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let lcd_sclk = io.pins.gpio6;
    let lcd_mosi = io.pins.gpio7;
    let lcd_cs = io.pins.gpio20;
    let lcd_miso = io.pins.gpio0; // random unused pin
    let lcd_dc = io.pins.gpio21.into_push_pull_output();
    let _lcd_backlight = io.pins.gpio4.into_push_pull_output();
    let lcd_reset = io.pins.gpio3.into_push_pull_output();

    let adc_pin = io.pins.gpio2.into_analog();

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let spi = Spi::new(
        peripherals.SPI2,
        40u32.MHz(),
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
    let mut seed_buffer = [1u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    info!("Initializing the ADC");
    let mut adc1_config = AdcConfig::new();
    let adc_pin = adc1_config.enable_pin(adc_pin, Attenuation::Attenuation11dB);

    let analog = peripherals.APB_SARADC.split();
    let adc1 = ADC::<ADC1>::adc(analog.adc1, adc1_config).unwrap();

    info!("Entering main loop");

    let ladder_movement_controller = LadderMovementController::new(adc1, adc_pin);
    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = LadderCompositeController::new(demo_movement_controller, ladder_movement_controller);

    app_loop(&mut display, seed_buffer, movement_controller);
    loop {}
}
