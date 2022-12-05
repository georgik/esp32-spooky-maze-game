#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

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

#[cfg(feature="esp32")]
use esp32_hal as hal;
#[cfg(feature="esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature="esp32s3")]
use esp32s3_hal as hal;
#[cfg(feature="esp32c3")]
use esp32c3_hal as hal;

use hal::{
    clock::{ ClockControl, CpuClock },
    // gdma::Gdma,
    i2c,
    pac::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Rng,
    Rtc,
    IO,
    Delay,
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

use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use spooky_core::{spritebuf::SpriteBuf, engine::Engine};

#[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

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
    }

    pub fn render_frame(&mut self) -> &D {

        self.engine.tick();
        self.engine.draw()

    }

}


#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 65535*3-16790;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }

    let peripherals = Peripherals::take().unwrap();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
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

    // let button_down_pin = io.pins.gpio11.into_pull_up_input();
    // https://docs.makerfactory.io/m5stack/core/fire/
    #[cfg(feature = "esp32")]
    let mut backlight = io.pins.gpio32.into_push_pull_output();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let mut backlight = io.pins.gpio9.into_push_pull_output();
    #[cfg(feature = "esp32c3")]
    let mut backlight = io.pins.gpio0.into_push_pull_output();

    #[cfg(feature = "esp32")]
    let spi = spi::Spi::new(
        peripherals.SPI2,
        io.pins.gpio18,
        io.pins.gpio23,
        io.pins.gpio19,
        io.pins.gpio14,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks);

    #[cfg(any(feature = "esp32s3_box"))]
    let mut backlight = io.pins.gpio45.into_push_pull_output();

    // #[cfg(feature = "esp32")]
    // backlight.set_low().unwrap();
    // #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    backlight.set_high().unwrap();


    #[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let reset = io.pins.gpio33.into_push_pull_output();
    #[cfg(any(feature = "esp32s3_box"))]
    let reset = io.pins.gpio48.into_push_pull_output();
    #[cfg(any(feature = "esp32c3"))]
    let reset = io.pins.gpio9.into_push_pull_output();

    #[cfg(any(feature = "esp32", feature = "esp32c3"))]
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio27.into_push_pull_output());
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio4.into_push_pull_output());

    #[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
    let mut delay = Delay::new(&clocks);

    // let mut display = mipidsi::Builder::ili9342c_rgb565(di)
    //     .with_display_size(320, 240)
    //     .with_orientation(mipidsi::Orientation::PortraitInverted(false))
    //     .init(&mut delay, Some(reset)).unwrap();
    // let mut display = mipidsi::Display::ili9342c_rgb565(di, core::prelude::v1::Some(reset), display_options);
    let mut display = Ili9341::new(di, reset, &mut delay, Orientation::Portrait, DisplaySize240x320).unwrap();

    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    display
    .init(
        &mut delay,
        DisplayOptions {
            ..DisplayOptions::default()
        },
    )
    .unwrap();
    
    // display.clear(RgbColor::WHITE).unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::BLACK),
    )
    .draw(&mut display)
    .unwrap();

    #[cfg(any(feature = "imu_controls"))]
    println!("Initializing IMU");
    #[cfg(any(feature = "imu_controls"))]
    let sda = io.pins.gpio8;
    #[cfg(any(feature = "imu_controls"))]
    let scl = io.pins.gpio18;

    #[cfg(any(feature = "imu_controls"))]
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .unwrap();

    #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);
    #[cfg(any(feature = "imu_controls"))]
    let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8;32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK ; 320*240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new(Some(seed_buffer), engine);
    universe.initialize();


    // #[cfg(any(feature = "imu_controls"))]
    // let accel_threshold = 0.20;

    loop {
        display.draw_iter(universe.render_frame().into_iter()).unwrap();
        // delay.delay_ms(300u32);
    }
}
