#![no_std]
#![no_main]

// Main baord: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s2/hw-reference/esp32s2/user-guide-esp32-s2-kaluga-1-kit.html
// Buttons - Lyra extension board: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s2/hw-reference/esp32s2/user-guide-esp-lyrat-8311a_v1.3.html

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    prelude::{Point, RgbColor},
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

use esp_backtrace as _;

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

    // let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Backlight is on GPIO6 in version 1.2, version 1.3 has display always on
    // let mut backlight = io.pins.gpio6.into_push_pull_output();
    // backlight.set_high().unwrap();

    let (unconfigured_pins, configured_pins, configured_system_pins) = setup_pins(io.pins);

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

    let mut display = match mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::Landscape(true))
        .with_color_order(mipidsi::ColorOrder::Rgb)
        .init(&mut delay, Some(configured_system_pins.reset)) {
            Ok(disp) => { disp },
            Err(_) => { panic!() },
    };

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

    let mut adc1_config = AdcConfig::new();
    let adc_pin = adc1_config.enable_pin(configured_pins.adc_pin, Attenuation::Attenuation11dB);

    let analog = peripherals.SENS.split();
    let adc1 = ADC::<ADC1>::adc(analog.adc1, adc1_config).unwrap();

    app_loop(adc1, adc_pin, &mut display, seed_buffer);
    loop {}
}
