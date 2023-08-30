#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

// Main baord: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s2/hw-reference/esp32s2/user-guide-esp32-s2-kaluga-1-kit.html
// Buttons - Lyra extension board: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s2/hw-reference/esp32s2/user-guide-esp-lyrat-8311a_v1.3.html

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    prelude::{Point, DrawTarget, RgbColor},
    mono_font::{
        ascii::{FONT_8X13},
        MonoTextStyle,
    },
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ ClockControl, CpuClock },
    // gdma::Gdma,
    i2c,
    peripherals::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Rng,
    Rtc,
    IO,
    Delay,
    adc::{AdcConfig, Attenuation, ADC, ADC1},
};

mod app;
use app::app_loop;

mod kaluga_composite_controller;
mod ladder_movement_controller;

mod setup;
use setup::setup_pins;

mod types;

// systimer was introduced in ESP32-S2, it's not available for ESP32
#[cfg(feature="system_timer")]
use hal::systimer::{SystemTimer};

use esp_backtrace as _;

use embedded_graphics::{pixelcolor::Rgb565};

use spooky_core::{spritebuf::SpriteBuf, engine::Engine, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};

use embedded_hal::digital::v2::OutputPin;
use embedded_graphics_framebuf::{FrameBuf};

pub struct Universe<D> {
    pub engine: Engine<D>,
}



#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);
    // self.delay = Some(delay);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Backlight is on GPIO6 in version 1.2, version 1.3 has display always on
    // let mut backlight = io.pins.gpio6.into_push_pull_output();
    // backlight.set_high().unwrap();
    let (unconfigured_pins, configured_pins, configured_system_pins) = setup_pins(io.pins);


    // let mut adc1_config = AdcConfig::new();

    // let mut resistor_ladder_adc =
    //     adc1_config.enable_pin(configured_pins.adc, Attenuation::Attenuation11dB);

    // let adc = setup::setup_adc();

    let spi = spi::Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        unconfigured_pins.sclk,
        unconfigured_pins.mosi,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let di = SPIInterfaceNoCS::new(spi, configured_system_pins.dc);

    let mut delay = Delay::new(&clocks);

    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::Landscape(true))
        .with_color_order(mipidsi::ColorOrder::Rgb)
        .init(&mut delay, Some(configured_system_pins.reset))
        .unwrap();

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


    app::app_loop(configured_pins.adc_pin, &mut display, seed_buffer);
    // loop {

    //     let button_value: u16 = nb::block!(adc1.read(&mut button_ladder_pin)).unwrap();
    //     // Based on https://github.com/espressif/esp-bsp/blob/master/esp32_s2_kaluga_kit/include/bsp/esp32_s2_kaluga_kit.h#L299
        // if button_value > 4000 && button_value < 5000 {
        //     universe.move_right();
        // } else if button_value >= 5000 && button_value < 6000 {
        //     universe.move_left();
        // } else if button_value >= 6000 && button_value < 7000 {
        //     universe.move_down();
        // } else if button_value >= 7000 && button_value < 8180 {
        //     universe.move_up();
        // }

    //     display.draw_iter(universe.render_frame().into_iter()).unwrap();
    //     // delay.delay_ms(300u32);
    // }
}
