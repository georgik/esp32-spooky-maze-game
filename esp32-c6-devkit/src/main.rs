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

use esp_println::println;

use hal::{
    clock::{ClockControl, CpuClock},
    // gdma::Gdma,
    i2c,
    peripherals::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Delay,
    Rng,
    Rtc,
    IO,
};


mod app;
use app::app_loop;

mod setup;
use setup::*;

mod types;

// systimer was introduced in ESP32-S2, it's not available for ESP32
#[cfg(feature = "system_timer")]
use hal::systimer::SystemTimer;

// use panic_halt as _;
use esp_backtrace as _;

use embedded_graphics::pixelcolor::Rgb565;

use spooky_core::{engine::Engine, spritebuf::SpriteBuf, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let mut system = peripherals.PCR.split();
    let mut clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.LP_CLKRST);
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

    rtc.swd.disable();
    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
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

    println!("Initialzed");

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

    app_loop(configured_pins, &mut display, seed_buffer);
    loop {}
}
