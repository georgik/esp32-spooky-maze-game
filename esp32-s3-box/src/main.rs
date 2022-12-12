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

#[cfg(feature = "esp32")]
use esp32_hal as hal;
#[cfg(feature = "esp32c3")]
use esp32c3_hal as hal;
#[cfg(feature = "esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature = "esp32s3")]
use esp32s3_hal as hal;

use hal::{
    clock::{ClockControl, CpuClock},
    // gdma::Gdma,
    i2c,
    pac::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Delay,
    Rng,
    Rtc,
    IO,
};

// systimer was introduced in ESP32-S2, it's not available for ESP32
#[cfg(feature = "system_timer")]
use hal::systimer::SystemTimer;

// use panic_halt as _;
use esp_backtrace as _;

#[cfg(feature = "riscv-rt")]
use riscv_rt::entry;
#[cfg(feature = "xtensa-lx-rt")]
use xtensa_lx_rt::entry;

use embedded_graphics::pixelcolor::Rgb565;
// use esp32s2_hal::Rng;

#[cfg(any(
    feature = "esp32s2_ili9341",
    feature = "esp32_wrover_kit",
    feature = "esp32c3_ili9341"
))]
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use spooky_core::{engine::Engine, spritebuf::SpriteBuf};

#[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::OutputPin;

pub struct Universe<I, D> {
    pub engine: Engine<D>,
    // #[cfg(any(feature = "imu_controls"))]
    icm: I,
    // icm: Option<Icm42670<shared_bus::I2cProxy<shared_bus::NullMutex<i2c::I2C<I2C0>>>>>
    // delay: Some(Delay),
}

impl<I: Accelerometer, D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>>
    Universe<I, D>
{
    pub fn new(icm: I, seed: Option<[u8; 32]>, engine: Engine<D>) -> Universe<I, D> {
        Universe {
            engine,
            // #[cfg(any(feature = "imu_controls"))]
            icm,
            // delay: None,
        }
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
    }

    pub fn render_frame(&mut self) -> &D {
        #[cfg(any(feature = "imu_controls"))]
        {
            let accel_threshold = 0.20;
            let accel_norm = self.icm.accel_norm().unwrap();

            if accel_norm.y > accel_threshold {
                self.engine.move_left();
            }

            if accel_norm.y < -accel_threshold {
                self.engine.move_right();
            }

            if accel_norm.x > accel_threshold {
                self.engine.move_down();
            }

            if accel_norm.x < -accel_threshold {
                self.engine.move_up();
            }

            // Quickly move up to teleport
            // Quickly move down to place dynamite
            if accel_norm.z < -1.2 {
                self.engine.teleport();
            } else if accel_norm.z > 1.5 {
                self.engine.place_dynamite();
            }
        }

        self.engine.tick();
        self.engine.draw()
    }
}

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 65535 * 4;
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

    #[cfg(feature = "esp32c3")]
    rtc.swd.disable();
    #[cfg(feature = "xtensa-lx-rt")]
    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);
    // self.delay = Some(delay);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // https://espressif-docs.readthedocs-hosted.com/projects/espressif-esp-dev-kits/en/latest/esp32s3/esp32-s3-usb-otg/user_guide.html
    // let button_up = button::Button::new();

    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_ok_pin = io.pins.gpio0.into_pull_up_input();
    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_menu_pin = io.pins.gpio14.into_pull_up_input();
    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_up_pin = io.pins.gpio10.into_pull_up_input();
    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let button_down_pin = io.pins.gpio11.into_pull_up_input();

    #[cfg(feature = "esp32")]
    let mut backlight = io.pins.gpio5.into_push_pull_output();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let mut backlight = io.pins.gpio9.into_push_pull_output();
    #[cfg(feature = "esp32c3")]
    let mut backlight = io.pins.gpio0.into_push_pull_output();

    #[cfg(feature = "esp32")]
    let spi = spi::Spi::new(
        peripherals.SPI2,
        io.pins.gpio19,
        io.pins.gpio23,
        io.pins.gpio25,
        io.pins.gpio22,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );

    #[cfg(any(feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let spi = spi::Spi::new(
        peripherals.SPI3,
        io.pins.gpio6,
        io.pins.gpio7,
        io.pins.gpio12,
        io.pins.gpio5,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );

    #[cfg(any(feature = "esp32s3_box"))]
    let sclk = io.pins.gpio7;
    #[cfg(any(feature = "esp32s3_box"))]
    let mosi = io.pins.gpio6;

    // let dma = Gdma::new(peripherals.DMA, &mut system.peripheral_clock_control);
    // let dma_channel = dma.channel0;

    // let mut descriptors = [0u32; 8 * 3];
    // let mut rx_descriptors = [0u32; 8 * 3];

    #[cfg(any(feature = "esp32s3_box"))]
    let spi = spi::Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        sclk,
        mosi,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );
    // .with_dma(dma_channel.configure(
    //     false,
    //     &mut descriptors,
    //     &mut rx_descriptors,
    //     DmaPriority::Priority0,
    // ));

    #[cfg(any(feature = "esp32s3_box"))]
    let mut backlight = io.pins.gpio45.into_push_pull_output();

    #[cfg(feature = "esp32")]
    backlight.set_low().unwrap();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    backlight.set_high().unwrap();

    #[cfg(feature = "esp32c3")]
    let spi = spi::Spi::new(
        peripherals.SPI2,
        io.pins.gpio6,
        io.pins.gpio7,
        io.pins.gpio12,
        io.pins.gpio20,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );

    #[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3_usb_otg"))]
    let reset = io.pins.gpio18.into_push_pull_output();
    #[cfg(any(feature = "esp32s3_box"))]
    let reset = io.pins.gpio48.into_push_pull_output();
    #[cfg(any(feature = "esp32c3"))]
    let reset = io.pins.gpio9.into_push_pull_output();

    #[cfg(any(feature = "esp32", feature = "esp32c3"))]
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio21.into_push_pull_output());
    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio4.into_push_pull_output());

    #[cfg(any(
        feature = "esp32s2_ili9341",
        feature = "esp32_wrover_kit",
        feature = "esp32c3_ili9341"
    ))]
    let mut delay = Delay::new(&clocks);

    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let mut display = mipidsi::Display::st7789(di, reset);

    //https://github.com/espressif/esp-box/blob/master/components/bsp/src/boards/esp32_s3_box.c

    #[cfg(any(feature = "esp32s3_box"))]
    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .init(&mut delay, Some(reset))
        .unwrap();
    // let mut display = mipidsi::Display::ili9342c_rgb565(di, core::prelude::v1::Some(reset), display_options);
    #[cfg(any(
        feature = "esp32s2_ili9341",
        feature = "esp32_wrover_kit",
        feature = "esp32c3_ili9341"
    ))]
    let mut display = Ili9341::new(
        di,
        reset,
        &mut delay,
        Orientation::Portrait,
        DisplaySize240x320,
    )
    .unwrap();

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
    );

    #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);
    #[cfg(any(feature = "imu_controls"))]
    let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new(icm, Some(seed_buffer), engine);
    universe.initialize();

    // #[cfg(any(feature = "imu_controls"))]
    // let accel_threshold = 0.20;

    loop {
        display
            .draw_iter(universe.render_frame().into_iter())
            .unwrap();
        // delay.delay_ms(300u32);
    }
}
