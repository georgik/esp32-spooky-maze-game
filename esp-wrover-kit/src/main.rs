#![no_std]
#![no_main]

// Implementation specific for esp-wrover-kit
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
    Delay, Rng, Rtc, IO, gpio::{Pins, self},
};
use esp_backtrace as _;
use esp_println::println;
use embedded_graphics::pixelcolor::Rgb565;
use spooky_core::{engine::Engine, universe::Universe, spritebuf::SpriteBuf, demo_movement_controller::DemoMovementController};
use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::OutputPin;

mod button_keyboard;
use button_keyboard::ButtonKeyboard;

mod button_movement_controller;
use button_movement_controller::ButtonMovementController;

mod embedded_movement_controller;
use embedded_movement_controller::EmbeddedMovementController;
use embedded_hal::digital::v2::InputPin;

struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio19<MODE>,
    pub mosi: gpio::Gpio23<MODE>,
    pub miso: gpio::Gpio25<MODE>,
    pub cs: gpio::Gpio22<MODE>,
}


struct ConfiguredPins<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin,
                      Dc: OutputPin, Bckl: OutputPin, Reset: OutputPin> {
    pub up_button: Up,
    pub down_button: Down,
    pub left_button: Left,
    pub right_button: Right,
    pub dynamite_button: Dyn,
    pub teleport_button: Tel,
    pub dc: Dc,
    pub backlight: Bckl,
    pub reset: Reset,
}

fn setup_pins(pins: Pins) -> (UnconfiguredPins<gpio::Unknown>, ConfiguredPins<impl InputPin, impl InputPin, impl InputPin, impl InputPin, impl InputPin,
    impl InputPin, impl OutputPin, impl OutputPin, impl OutputPin>) {
            let unconfigured_pins = UnconfiguredPins {
        sclk: pins.gpio19,
        mosi: pins.gpio23,
        miso: pins.gpio25,
        cs: pins.gpio22,
    };

    let configured_pins = ConfiguredPins {
        up_button: pins.gpio14.into_pull_up_input(),
        down_button: pins.gpio12.into_pull_up_input(),
        left_button: pins.gpio13.into_pull_up_input(),
        right_button: pins.gpio15.into_pull_up_input(),
        dynamite_button: pins.gpio26.into_pull_up_input(),
        teleport_button: pins.gpio27.into_pull_up_input(),
        dc: pins.gpio21.into_push_pull_output(),
        backlight: pins.gpio5.into_push_pull_output(),
        reset: pins.gpio18.into_push_pull_output(),
    };

    (unconfigured_pins, configured_pins)
}




#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

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

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let (unconfigured_pins, mut configured_pins) = setup_pins(io.pins);

    let spi = spi::Spi::new(
        peripherals.SPI3,
        unconfigured_pins.sclk,
        unconfigured_pins.mosi,
        unconfigured_pins.miso,
        unconfigured_pins.cs,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    configured_pins.backlight.set_low();

    let di = SPIInterfaceNoCS::new(spi, configured_pins.dc);

    let mut display = match mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 320 as u16)
        .with_orientation(mipidsi::Orientation::Landscape(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(configured_pins.reset)) {
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

    let button_keyboard = ButtonKeyboard {
        up_button: configured_pins.up_button,
        down_button: configured_pins.down_button,
        left_button: configured_pins.left_button,
        right_button: configured_pins.right_button,
        dynamite_button: configured_pins.dynamite_button,
        teleport_button: configured_pins.teleport_button,
    };

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [1u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    let button_movement_controller = ButtonMovementController::new();
    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let mut movement_controller = EmbeddedMovementController::new(demo_movement_controller, button_movement_controller);


    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let mut button_movement_controller = ButtonMovementController::new();

    println!("Creating universe");
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();

    println!("Starting main loop");
    loop {
        let event = button_keyboard.poll();
        // movement_controller.react_to_event(event);
        display
            .draw_iter(universe.render_frame().into_iter())
            .unwrap();
    }
}
