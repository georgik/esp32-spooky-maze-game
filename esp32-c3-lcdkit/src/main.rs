#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ClockControl, CpuClock},
    gpio::{Input, PullUp},
    // gdma::Gdma,
    i2c,
    interrupt,
    peripherals::{self, Peripherals, TIMG0, TIMG1},
    prelude::*,
    riscv,
    spi,
    timer::{Timer, Timer0, TimerGroup},
    Delay,
    Rng,
    IO,
};

mod app;
use app::app_loop;

mod accel_movement_controller;
mod lcdkit_composite_controller;
mod setup;
mod rotary_movement_controller;
use rotary_encoder_embedded::{standard::StandardMode, Direction, RotaryEncoder};
use setup::setup_pins;

mod types;

use esp_backtrace as _;

// #[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
// #[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

struct NoOpPin;
use core::{borrow::BorrowMut, cell::RefCell};
use critical_section::Mutex;

static TIMER0: Mutex<RefCell<Option<Timer<Timer0<TIMG0>>>>> = Mutex::new(RefCell::new(None));
static ROTARY_ENCODER: Mutex<
    RefCell<
        Option<
            RotaryEncoder<
                StandardMode,
                hal::gpio::GpioPin<Input<PullUp>, 10>,
                hal::gpio::GpioPin<Input<PullUp>, 6>,
            >,
        >,
    >,
> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TG0_T0_LEVEL() {
    critical_section::with(|cs| {
        if let Some(ref mut rotary_encoder) =
            ROTARY_ENCODER.borrow_ref_mut(cs).borrow_mut().as_mut()
        {
            rotary_encoder.update();
        }

        let mut timer0 = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer0.as_mut().unwrap();

        timer0.clear_interrupt();
        timer0.start(10u64.millis());
    });
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // https://docs.espressif.com/projects/espressif-esp-dev-kits/en/latest/esp32c3/esp32-c3-lcdkit/user_guide.html#gpio-allocation
    let (uninitialized_pins, mut configured_system_pins, mut rotary_pins) = setup_pins(io.pins);
    println!("SPI LED driver initialized");
    let spi = spi::Spi::new(
        peripherals.SPI2,
        uninitialized_pins.sclk,
        uninitialized_pins.mosi,
        uninitialized_pins.miso,
        uninitialized_pins.cs,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &clocks,
    );

    println!("SPI ready");

    let di = SPIInterfaceNoCS::new(spi, configured_system_pins.dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);
    let mut display = match mipidsi::Builder::gc9a01(di)
        // let mut display = match mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 240 as u16)
        .with_orientation(mipidsi::Orientation::Portrait(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(configured_system_pins.reset))
    {
        Ok(disp) => disp,
        Err(_) => {
            panic!()
        }
    };

    let _ = configured_system_pins.backlight.set_high();

    println!("Initializing...");
    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    // #[cfg(any(feature = "imu_controls"))]
    // let i2c = i2c::I2C::new(
    //     peripherals.I2C0,
    //     unconfigured_pins.sda,
    //     unconfigured_pins.scl,
    //     100u32.kHz(),
    //     &mut system.peripheral_clock_control,
    //     &clocks,
    // );

    // #[cfg(any(feature = "imu_controls"))]
    // let bus = BusManagerSimple::new(i2c);
    // #[cfg(any(feature = "imu_controls"))]
    // let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    let mut rotary_encoder =
        RotaryEncoder::new(rotary_pins.dt, rotary_pins.clk).into_standard_mode();

    interrupt::enable(
        peripherals::Interrupt::TG0_T0_LEVEL,
        interrupt::Priority::Priority1,
    )
    .unwrap();
    timer0.start(10u64.millis());
    timer0.listen();

    critical_section::with(|cs| {
        ROTARY_ENCODER.borrow_ref_mut(cs).replace(rotary_encoder);
        TIMER0.borrow_ref_mut(cs).replace(timer0);
    });

    unsafe {
        riscv::interrupt::enable();
    }

    let event_bus = types::EventBus {
        direction: Direction::None,
    };

    // app_loop( &mut display, seed_buffer, icm);
    println!("Starting application loop");

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let rotary_movement_controller = crate::rotary_movement_controller::RotaryMovementController::new();
    let movement_controller = crate::lcdkit_composite_controller::LcdKitCompositeController::new(demo_movement_controller, rotary_movement_controller);

    use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
    use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
    use embedded_graphics_framebuf::FrameBuf;
    use embedded_graphics::prelude::RgbColor;
    let mut data = [Rgb565::BLACK; 240 * 240];
    let fbuf = FrameBuf::new(&mut data, 240, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();

    let mut clockwise_action = spooky_core::engine::Action::Right;
    let mut counter_clockwise_action = spooky_core::engine::Action::Left;
    let mut switch_in_progress = false;

    loop {
        let direction = critical_section::with(|cs| {
            if let Some(ref mut rotary_encoder) =
                ROTARY_ENCODER.borrow_ref_mut(cs).borrow_mut().as_mut()
            {
                return rotary_encoder.poll();
            }
            Direction::None
        });


        // Switch direction of actions
        if rotary_pins.switch.is_low().unwrap_or(false) && !switch_in_progress  {
            println!("Switching direction");
            if clockwise_action == spooky_core::engine::Action::Right {
                clockwise_action = spooky_core::engine::Action::Down;
                counter_clockwise_action = spooky_core::engine::Action::Up;
            } else {
                clockwise_action = spooky_core::engine::Action::Right;
                counter_clockwise_action = spooky_core::engine::Action::Left;
            }
            switch_in_progress = true;
        } else {
            switch_in_progress = false;
        }

        let controller = universe.get_movement_controller_mut();
        match direction {
            Direction::Clockwise => {
                controller.set_movement(clockwise_action);
                println!("Clockwise");
            }
            Direction::Anticlockwise => {
                let controller = universe.get_movement_controller_mut();
                controller.set_movement(counter_clockwise_action);
                println!("Anticlockwise");
            }
            Direction::None => {
                controller.set_movement(spooky_core::engine::Action::None);
                println!("None");
            }
        }


        let _ = display
            .draw_iter(universe.render_frame().into_iter());
    }
}
