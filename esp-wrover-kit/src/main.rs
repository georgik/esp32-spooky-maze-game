#![no_std]
#![no_main]
// Implementation specific for esp-wrover-kit
// https://docs.espressif.com/projects/esp-idf/en/latest/esp32/hw-reference/esp32/get-started-wrover-kit.html

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use spi_dma_displayinterface::spi_dma_displayinterface;

use esp_backtrace as _;
use hal::{psram, prelude::*,
    peripherals::Peripherals,
    dma::DmaPriority,
    pdma::Dma,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    clock::{ClockControl, CpuClock}, Delay, Rng, IO};

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

mod setup;
use setup::{setup_button_keyboard, setup_movement_controller};
mod types;

use spooky_embedded::{
    app::app_loop,
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
};

use crate::types::ConfiguredPins;

pub fn init_psram_heap() {
    unsafe {
        ALLOCATOR.init(psram::psram_vaddr_start() as *mut u8, psram::PSRAM_BYTES);
    }
}


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    psram::init_psram(peripherals.PSRAM);
    init_psram_heap();

    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 160MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let lcd_sclk = io.pins.gpio19;
    let lcd_mosi = io.pins.gpio23;
    let lcd_miso = io.pins.gpio25;
    let lcd_cs = io.pins.gpio22;
    let lcd_dc = io.pins.gpio21.into_push_pull_output();
    let _lcd_backlight = io.pins.gpio5.into_push_pull_output();
    let lcd_reset = io.pins.gpio18.into_push_pull_output();

    let dma = Dma::new(system.dma);
    let dma_channel = dma.spi2channel;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let configured_pins = ConfiguredPins {
        up_button: io.pins.gpio14.into_pull_up_input(),
        down_button: io.pins.gpio12.into_pull_up_input(),
        left_button: io.pins.gpio13.into_pull_up_input(),
        right_button: io.pins.gpio15.into_pull_up_input(),
        dynamite_button: io.pins.gpio26.into_pull_up_input(),
        teleport_button: io.pins.gpio27.into_pull_up_input(),
    };

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

    let di = spi_dma_displayinterface::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

    let mut display = match mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(LCD_H_RES, LCD_V_RES)
        .with_orientation(mipidsi::Orientation::Landscape(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(lcd_reset)) {
            Ok(disp) => { disp },
            Err(_) => { panic!() },
    };

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [1u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    let button_keyboard = setup_button_keyboard(configured_pins);

    let movement_controller = setup_movement_controller(seed_buffer, button_keyboard);

    app_loop(&mut display, seed_buffer, movement_controller);
    loop {}
}
