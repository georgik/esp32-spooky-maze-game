#![no_std]
#![no_main]

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
    psram,
    spi,
    timer::TimerGroup,
    Delay,
    Rng,
    Rtc,
    IO, ledc::channel::config,
};

mod app;
use app::app_loop;

mod accel_movement_controller;
mod s3box_composite_controller;
mod setup;
use setup::setup_pins;

mod types;

use esp_backtrace as _;

use embedded_graphics::pixelcolor::Rgb565;

use spooky_core::{engine::Engine, spritebuf::SpriteBuf, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite } };

// #[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
// #[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

// use embedded_graphics_framebuf::FrameBuf;
// use embedded_hal::digital::v2::OutputPin;

// pub struct Universe<I, D> {
//     pub engine: Engine<D>,
//     icm: I,
// }

// impl<I: Accelerometer, D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>>
//     Universe<I, D>
// {
//     pub fn new(icm: I, seed: Option<[u8; 32]>, engine: Engine<D>) -> Universe<I, D> {
//         Universe {
//             engine,
//             icm,
//         }
//     }

//     pub fn initialize(&mut self) {
//         self.engine.initialize();
//         self.engine.start();
//     }

//     pub fn render_frame(&mut self) -> &D {
//         #[cfg(any(feature = "imu_controls"))]
//         {
//             let accel_threshold = 0.20;
//             let accel_norm = self.icm.accel_norm().unwrap();

//             if accel_norm.y > accel_threshold {
//                 self.engine.action(Left);
//             }

//             if accel_norm.y < -accel_threshold {
//                 self.engine.action(Right);
//             }

//             if accel_norm.x > accel_threshold {
//                 self.engine.action(Down);
//             }

//             if accel_norm.x < -accel_threshold {
//                 self.engine.action(Up);
//             }

//             // Quickly move up to teleport
//             // Quickly move down to place dynamite
//             if accel_norm.z < -1.2 {
//                 self.engine.action(Teleport);
//             } else if accel_norm.z > 1.5 {
//                 self.engine.action(PlaceDynamite);
//             }
//         }

//         self.engine.tick();
//         self.engine.draw()
//     }
// }

fn init_psram_heap() {
    unsafe {
        ALLOCATOR.init(psram::psram_vaddr_start() as *mut u8, psram::PSRAM_BYTES);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    psram::init_psram(peripherals.PSRAM);
    init_psram_heap();

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

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let (unconfigured_pins, /*configured_pins, */mut configured_system_pins) = setup_pins(io.pins);
    println!("SPI LED driver initialized");
    let spi = spi::Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        unconfigured_pins.sclk,
        unconfigured_pins.mosi,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    println!("SPI ready");

    let di = SPIInterfaceNoCS::new(spi, configured_system_pins.dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);

    let mut display = match mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(configured_system_pins.reset)) {
        Ok(display) => display,
        Err(e) => {
            // Handle the error and possibly exit the application
            panic!("Display initialization failed");
        }
    };

    configured_system_pins.backlight.set_high();

    println!("Initializing...");
        Text::new(
            "Initializing...",
            Point::new(80, 110),
            MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
        )
        .draw(&mut display)
        .unwrap();



    // #[cfg(any(feature = "imu_controls"))]
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        unconfigured_pins.sda,
        unconfigured_pins.scl,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    // #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);
    // #[cfg(any(feature = "imu_controls"))]
    let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    // app_loop(configured_pins, &mut display, seed_buffer);
    app_loop( &mut display, seed_buffer, icm);
    loop {}

}
