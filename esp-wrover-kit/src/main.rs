#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

// https://docs.espressif.com/projects/esp-idf/en/latest/esp32/hw-reference/esp32/get-started-wrover-kit.html

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{DrawTarget, Point, RgbColor},
    text::Text,
    Drawable,
};

use hal::{
    clock::{ClockControl, CpuClock},
    peripherals::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Delay, Rng, Rtc, IO,
};

// use panic_halt as _;
use esp_backtrace as _;

#[cfg(feature = "xtensa-lx-rt")]
use xtensa_lx_rt::entry;

use embedded_graphics::pixelcolor::Rgb565;

use spooky_core::{engine::Engine, spritebuf::SpriteBuf, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};

#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::OutputPin;

pub struct Universe<D> {
    pub engine: Engine<D>,
}

impl<D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Universe<D> {
    pub fn new(seed: Option<[u8; 32]>, engine: Engine<D>) -> Universe<D> {
        Universe { engine }
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

    pub fn teleport(&mut self) {
        self.engine.action(Teleport);
    }

    pub fn place_dynamite(&mut self) {
        self.engine.action(PlaceDynamite);
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

    #[cfg(feature = "esp32c3")]
    rtc.swd.disable();
    #[cfg(feature = "xtensa-lx-rt")]
    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut backlight = io.pins.gpio5.into_push_pull_output();

    let spi = spi::Spi::new(
        peripherals.SPI3, // Real HW working with SPI2, but Wokwi seems to work only with SPI3
        io.pins.gpio19,   // SCLK
        io.pins.gpio23,   // MOSI
        io.pins.gpio25,   // MISO
        io.pins.gpio22,   // CS
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    backlight.set_low().unwrap();

    let reset = io.pins.gpio18.into_push_pull_output();
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio21.into_push_pull_output());

    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 320 as u16)
        .with_orientation(mipidsi::Orientation::Landscape(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
        .unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    let button_boot = io.pins.gpio2.into_pull_up_input();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    let engine = Engine::new(spritebuf, Some(seed_buffer));
    let mut universe = Universe::new(Some(seed_buffer), engine);
    universe.initialize();

    loop {
        if button_boot.is_low().unwrap() {
            universe.teleport();
        }

        display
            .draw_iter(universe.render_frame().into_iter())
            .unwrap();
        // delay.delay_ms(300u32);
    }
}
