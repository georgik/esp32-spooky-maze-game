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
    Delay, Rng, Rtc, IO, gpio::Pins,
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

fn setup_pins(io: &Pins) -> (impl InputPin, impl InputPin, impl InputPin, impl InputPin, impl InputPin, impl InputPin) {
    (
        io.gpio14.into_pull_up_input(),
        io.gpio12.into_pull_up_input(),
        io.gpio13.into_pull_up_input(),
        io.gpio15.into_pull_up_input(),
        io.gpio26.into_pull_up_input(),
        io.gpio27.into_pull_up_input()
    )
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
    let gpio = peripherals.gpio.split();
    let (up_button, down_button, left_button, right_button, dynamite_button, teleport_button) = setup_pins(&io.pins);

    let button_keyboard = ButtonKeyboard {
        up_button,
        down_button,
        left_button,
        right_button,
        // dynamite_button,
        // teleport_button,
    };
    
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

    // let button_boot = io.pins.gpio2.into_pull_up_input();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [1u8; 32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    
    let mut button_movement_controller = ButtonMovementController::new();
   
    // let movement_controller = EmbeddedMovementController::new(
    //     DemoMovementController::new(seed_buffer),
    //     ButtonMovementController::new(
    //         io.pins.gpio14.into_pull_up_input(),
    //         io.pins.gpio12.into_pull_up_input(),
    //         io.pins.gpio13.into_pull_up_input(),
    //         io.pins.gpio15.into_pull_up_input(),
    //         io.pins.gpio26.into_pull_up_input(),
    //         io.pins.gpio27.into_pull_up_input(),
    //     ),
    //     start_button,
    // );

    println!("Creating universe");
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();

    println!("Starting main loop");
    loop {
        // if button_boot.is_low().unwrap() {
        //     universe.teleport();
        // }

        let event = button_keyboard.poll();
        button_movement_controller.react_to_event(event);
        display
            .draw_iter(universe.render_frame().into_iter())
            .unwrap();
        
    }
}
