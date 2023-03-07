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
    peripherals::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Delay,
    Rng,
    Rtc,
    IO,
};


use esp_wifi::esp_now::{PeerInfo, BROADCAST_ADDRESS};
use esp_wifi::{current_millis, initialize};

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

use spooky_core::{engine::Engine, spritebuf::SpriteBuf, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite } };

#[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::OutputPin;
use esp32s3_hal::Cpu;

pub struct Universe<I, D> {
    pub engine: Engine<D>,
    icm: I,
}

impl<I: Accelerometer, D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>>
    Universe<I, D>
{
    pub fn new(icm: I, seed: Option<[u8; 32]>, engine: Engine<D>) -> Universe<I, D> {
        Universe {
            engine,
            icm,
        }
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
        self.engine.start();
    }

    pub fn render_frame(&mut self) -> &D {
        #[cfg(any(feature = "imu_controls"))]
        {
            let accel_threshold = 0.20;
            let accel_norm = self.icm.accel_norm().unwrap();

            if accel_norm.y > accel_threshold {
                self.engine.action(Left);
            }

            if accel_norm.y < -accel_threshold {
                self.engine.action(Right);
            }

            if accel_norm.x > accel_threshold {
                self.engine.action(Down);
            }

            if accel_norm.x < -accel_threshold {
                self.engine.action(Up);
            }

            // Quickly move up to teleport
            // Quickly move down to place dynamite
            if accel_norm.z < -1.2 {
                self.engine.action(Teleport);
            } else if accel_norm.z > 1.5 {
                self.engine.action(PlaceDynamite);
            }
        }

        self.engine.tick();
        self.engine.draw()
    }
}

#[entry]
fn main() -> ! {
    esp_wifi::init_heap();
    // const HEAP_SIZE: usize = 65535 * 4;
    // static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    // unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }

    let peripherals = Peripherals::take();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    // let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    // let mut wdt0 = timer_group0.wdt;
    // let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    // let mut wdt1 = timer_group1.wdt;

    #[cfg(feature = "esp32c3")]
    rtc.swd.disable();
    // #[cfg(feature = "esp32s3")]
    // rtc.rwdt.disable();

    use hal::{ Cpu };
    use esp_hal_common::rtc_cntl::{get_reset_reason};
    // use esp_hal_common::rtc_cntl::rtc::SocResetReason;
    let reset_reason = get_reset_reason(Cpu::ProCpu).unwrap();
    println!("Reset reason: {:?}", reset_reason);
    // match reset_reason {
    //     SocResetReason::PowerOn => println!("Power-on reset"),
    //     SocResetReason::ExternalPin => println!("External reset"),
    //     SocResetReason::Software => println!("Software reset"),
    //     SocResetReason::Watchdog => println!("Watchdog reset"),
    //     SocResetReason::Brownout => println!("Brownout reset"),
    //     SocResetReason::Deepsleep => println!("Deepsleep reset"),
    //     SocResetReason::Unknown => println!("Unknown reset reason"),
    // }
    // wdt0.disable();
    // wdt1.disable();

    println!("Initializing TimerGroup");
    use hal::timer::TimerGroup;
    let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    initialize(timg1.timer0, Rng::new(peripherals.RNG), &clocks).unwrap();


    let mut delay = Delay::new(&clocks);
    // self.delay = Some(delay);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);


    #[cfg(any(feature = "esp32s3_box"))]
    let sclk = io.pins.gpio7;
    #[cfg(any(feature = "esp32s3_box"))]
    let mosi = io.pins.gpio6;

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

    #[cfg(any(feature = "esp32s3_box"))]
    let mut backlight = io.pins.gpio45.into_push_pull_output();

    #[cfg(feature = "esp32")]
    backlight.set_low().unwrap();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    backlight.set_high().unwrap();

    #[cfg(any(feature = "esp32s3_box"))]
    let reset = io.pins.gpio48.into_push_pull_output();

    #[cfg(any(feature = "esp32s2", feature = "esp32s3"))]
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio4.into_push_pull_output());

    //https://github.com/espressif/esp-box/blob/master/components/bsp/src/boards/esp32_s3_box.c

    #[cfg(any(feature = "esp32s3_box"))]
    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
        .unwrap();

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
    println!("Initializing I2C");
    #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);
    #[cfg(any(feature = "imu_controls"))]
    let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    // let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    // rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new(icm, Some(seed_buffer), engine);
    universe.initialize();

    println!("Initializing esp-now");
    let mut esp_now = esp_wifi::esp_now::esp_now().initialize().unwrap();

    println!("esp-now version {}", esp_now.get_version().unwrap());

    loop {
        let r = esp_now.receive();
        if let Some(r) = r {
            println!("Received {:x?}", r);
            let rec_bytes = r.get_data();

            let x_bits = ((rec_bytes[0] as u32) << 24)
                | ((rec_bytes[1] as u32) << 16)
                | ((rec_bytes[2] as u32) << 8);


            let y_bits = ((rec_bytes[3] as u32) << 24)
                | ((rec_bytes[4] as u32) << 16)
                | ((rec_bytes[5] as u32) << 8);

            let z_bits = ((rec_bytes[6] as u32) << 24)
                | ((rec_bytes[7] as u32) << 16)
                | ((rec_bytes[8] as u32) << 8);

            println!("Recieved: x:{:x?}, y:{:x?}, z:{:x?} ", f32::from_bits(x_bits), f32::from_bits(y_bits), f32::from_bits(z_bits));

            if r.info.dst_address == BROADCAST_ADDRESS {
                if !esp_now.peer_exists(&r.info.src_address).unwrap() {
                    esp_now
                        .add_peer(PeerInfo {
                            peer_address: r.info.src_address,
                            lmk: None,
                            channel: None,
                            encrypt: false,
                        })
                        .unwrap();
                }
                //esp_now.send(&r.info.src_address, b"Hello Peer").unwrap();
            }
        }
        display
            .draw_iter(universe.render_frame().into_iter())
            .unwrap();
    }
}
