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

use mipidsi::hal::{ Orientation, Rotation };
use mipidsi::ColorOrder;

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

// systimer was introduced in ESP32-S2, it's not available for ESP32
#[cfg(feature="system_timer")]
use hal::systimer::{SystemTimer};

// use panic_halt as _;
use esp_backtrace as _;

#[cfg(feature="xtensa-lx-rt")]
use xtensa_lx_rt::entry;
#[cfg(feature="riscv-rt")]
use riscv_rt::entry;

use embedded_graphics::{pixelcolor::Rgb565};
// use esp32s2_hal::Rng;

use spooky_core::{spritebuf::SpriteBuf, engine::Engine, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};

use embedded_hal::digital::v2::OutputPin;
use embedded_graphics_framebuf::{FrameBuf};

pub struct Universe<D> {
    pub engine: Engine<D>,
}


impl <D:embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Universe <D> {
    pub fn new(seed: Option<[u8; 32]>, engine:Engine<D>) -> Universe<D> {
        Universe {
            engine,
        }
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
        self.engine.start();
    }

    pub fn move_up(&mut self) {
        self.engine.action(Up);
    }

    pub fn move_down(&mut self) {
        self.engine.action(Down);
    }

    pub fn move_left(&mut self) {
        self.engine.action(Left);
    }

    pub fn move_right(&mut self) {
        self.engine.action(Right);
    }

    pub fn render_frame(&mut self) -> &D {

        self.engine.tick();
        self.engine.draw()

    }

}


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
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

    #[cfg(feature="esp32c3")]
    rtc.swd.disable();
    #[cfg(feature="xtensa-lx-rt")]
    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);
    // self.delay = Some(delay);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Create ADC instances
    let analog = peripherals.SENS.split();

    let mut adc1_config = AdcConfig::new();

    let mut button_ladder_pin =
        adc1_config.enable_pin(io.pins.gpio6.into_analog(), Attenuation::Attenuation11dB);

    let mut adc1 = ADC::<ADC1>::adc(analog.adc1, adc1_config).unwrap();

    // Backlight is on GPIO6 in version 1.2, version 1.3 has display always on
    // let mut backlight = io.pins.gpio6.into_push_pull_output();
    // backlight.set_high().unwrap();

    let spi = spi::Spi::new(
        peripherals.SPI2,
        io.pins.gpio15,  // SCLK
        io.pins.gpio9,   // MOSI
        io.pins.gpio8,   // MISO
        io.pins.gpio11,   // CS
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks);

    let reset = io.pins.gpio16.into_push_pull_output();

    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio13.into_push_pull_output());

    #[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
    let mut delay = Delay::new(&clocks);

    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_color_order(ColorOrder::Bgr)
        .with_orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut delay, Some(reset))
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
    let mut data = [Rgb565::BLACK ; 320*240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new(Some(seed_buffer), engine);
    universe.initialize();

    loop {

        let button_value: u16 = nb::block!(adc1.read(&mut button_ladder_pin)).unwrap();
        // Based on https://github.com/espressif/esp-bsp/blob/master/esp32_s2_kaluga_kit/include/bsp/esp32_s2_kaluga_kit.h#L299
        if button_value > 4000 && button_value < 5000 {
            universe.move_right();
        } else if button_value >= 5000 && button_value < 6000 {
            universe.move_left();
        } else if button_value >= 6000 && button_value < 7000 {
            universe.move_down();
        } else if button_value >= 7000 && button_value < 8180 {
            universe.move_up();
        }

        display.draw_iter(universe.render_frame().into_iter()).unwrap();
        // delay.delay_ms(300u32);
    }
}
