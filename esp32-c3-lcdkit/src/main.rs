#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// use display_interface_spi::SPIInterfaceNoCS;
use spi_dma_displayinterface::spi_dma_displayinterface;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::Point,
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ClockControl, CpuClock},
    gpio::{Input, PullUp},
    dma::DmaPriority,
    gdma::Gdma,
    interrupt,
    peripherals::{self, Peripherals, TIMG0, TIMG1},
    prelude::*,
    riscv,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    timer::{Timer, Timer0, TimerGroup},
    Delay,
    Rng,
    IO,
};

use spooky_embedded::{
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE, LCD_PIXELS},
    controllers::{accel::AccelMovementController, composites::accel_composite::AccelCompositeController}
};

mod app;

mod lcdkit_composite_controller;
mod rotary_movement_controller;
use rotary_encoder_embedded::{standard::StandardMode, Direction, RotaryEncoder};

use esp_backtrace as _;

use core::{borrow::BorrowMut, cell::RefCell};
use critical_section::Mutex;

// Timer for encoder polling
static TIMER0: Mutex<RefCell<Option<Timer<Timer0<TIMG0>>>>> = Mutex::new(RefCell::new(None));

// Timer for frame rate calculation
static TIMER1: Mutex<RefCell<Option<Timer<Timer0<TIMG1>>>>> = Mutex::new(RefCell::new(None));

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

static FRAME_COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));
static TIMER_INTERVAL: u64 = 1_000; // 1 second

#[interrupt]
fn TG1_T0_LEVEL() {
    critical_section::with(|cs| {
        // Calculate and print frame rate
        let frame_count = *FRAME_COUNTER.borrow_ref_mut(cs);
        let frame_rate = frame_count as f64 / (TIMER_INTERVAL as f64 / 1_000.0);
        println!("Frame Rate: {:.2} FPS", frame_rate);

        // Reset frame counter
        *FRAME_COUNTER.borrow_ref_mut(cs) = 0;

        // Reset and restart the timer for the next interval
        let mut timer1 = TIMER1.borrow_ref_mut(cs);
        let timer1 = timer1.as_mut().unwrap();
        timer1.clear_interrupt();
        timer1.start(TIMER_INTERVAL.millis());
    });
}

// Call this function at the end of each frame rendering in your main loop
fn increment_frame_counter() {
    critical_section::with(|cs| {
        *FRAME_COUNTER.borrow_ref_mut(cs) += 1;
    });
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 80MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut timer1 = timer_group1.timer0;

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // https://docs.espressif.com/projects/espressif-esp-dev-kits/en/latest/esp32c3/esp32-c3-lcdkit/user_guide.html#gpio-allocation

    let lcd_sclk = io.pins.gpio1;
    let lcd_mosi = io.pins.gpio0;
    let lcd_miso = io.pins.gpio4;
    let lcd_cs = io.pins.gpio7;
    let lcd_dc = io.pins.gpio2.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio5.into_push_pull_output();
    let lcd_reset = io.pins.gpio8.into_push_pull_output();

    let rotary_dt = io.pins.gpio10.into_pull_up_input();
    let rotary_clk = io.pins.gpio6.into_pull_up_input();
    let rotary_switch = io.pins.gpio9.into_pull_up_input();

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

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

    println!("SPI ready");

    let di = spi_dma_displayinterface::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

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
        .init(&mut delay, Some(lcd_reset))
    {
        Ok(disp) => disp,
        Err(_) => {
            panic!()
        }
    };

    let _ = lcd_backlight.set_high();

    println!("Initializing...");
    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    let rotary_encoder =
        RotaryEncoder::new(rotary_dt, rotary_clk).into_standard_mode();

    interrupt::enable(
        peripherals::Interrupt::TG0_T0_LEVEL,
        interrupt::Priority::Priority1,
    )
    .unwrap();

    interrupt::enable(
        peripherals::Interrupt::TG1_T0_LEVEL,
        interrupt::Priority::Priority1,
    )
    .unwrap();

    timer0.start(10u64.millis());
    timer0.listen();

    // Meassure FPS
    timer1.start(TIMER_INTERVAL.millis());
    timer1.listen();

    critical_section::with(|cs| {
        ROTARY_ENCODER.borrow_ref_mut(cs).replace(rotary_encoder);
        TIMER0.borrow_ref_mut(cs).replace(timer0);
        TIMER1.borrow_ref_mut(cs).replace(timer1);
    });

    unsafe {
        riscv::interrupt::enable();
    }

    // app_loop( &mut display, seed_buffer, icm);
    println!("Starting application loop");

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let rotary_movement_controller = crate::rotary_movement_controller::RotaryMovementController::new();
    let movement_controller = crate::lcdkit_composite_controller::LcdKitCompositeController::new(demo_movement_controller, rotary_movement_controller);

    use embedded_graphics::pixelcolor::Rgb565;
    use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
    use embedded_graphics_framebuf::FrameBuf;
    use embedded_graphics::prelude::RgbColor;
    let mut data = [Rgb565::BLACK; LCD_PIXELS];
    let fbuf = FrameBuf::new(&mut data, LCD_H_RES as usize, LCD_V_RES as usize);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();

    let mut clockwise_action = spooky_core::engine::Action::Right;
    let mut counter_clockwise_action = spooky_core::engine::Action::Left;
    let mut switch_in_progress = false;

    // Do one poll to clear the initial state
    let _direction = critical_section::with(|cs| {
        if let Some(ref mut rotary_encoder) =
            ROTARY_ENCODER.borrow_ref_mut(cs).borrow_mut().as_mut()
        {
            return rotary_encoder.poll();
        }
        Direction::None
    });

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
        if rotary_switch.is_low().unwrap_or(false) && !switch_in_progress  {
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

        let pixel_iterator = universe.render_frame().get_pixel_iter();
        // -1 for some reason is necessary otherwise the display is skewed
        let _ = display.set_pixels(0, 0, LCD_V_RES-1, LCD_H_RES, pixel_iterator);

        increment_frame_counter();

    }
}
