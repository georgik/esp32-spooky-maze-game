#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    prelude::RgbColor,
    mono_font::{
        ascii::{FONT_8X13, FONT_9X18_BOLD},
        MonoTextStyle,
    },
    prelude::Point,
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
    clock::ClockControl,
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

use embedded_graphics::{image::Image, pixelcolor::Rgb565};
use tinybmp::Bmp;
// use esp32s2_hal::Rng;

#[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg", feature = "esp32s3_box"))]
use mipidsi::{Display, DisplayOptions, Orientation};
#[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use maze_generator::prelude::*;
use maze_generator::recursive_backtracking::{RbGenerator};

#[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
#[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 65535;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }

    let peripherals = Peripherals::take().unwrap();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();
    let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

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
        &mut clocks);

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
        &mut clocks);

        // let mut spi = Spi::new(
        //     peripherals.SPI2,
        //     sclk,
        //     mosi,
        //     miso,
        //     cs,
        //     100u32.kHz(),
        //     SpiMode::Mode0,
        //     &mut system.peripheral_clock_control,
        //     &clocks,
        // );

    #[cfg(any(feature = "esp32s3_box"))]
    let sclk = io.pins.gpio7;
    #[cfg(any(feature = "esp32s3_box"))]
    let mosi = io.pins.gpio6;

    #[cfg(any(feature = "esp32s3_box"))]
    let spi = spi::Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        sclk,
        mosi,
        4u32.MHz(),
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
        &mut clocks);

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

    #[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
    let mut delay = Delay::new(&clocks);

    #[cfg(any(feature = "esp32s2_usb_otg", feature = "esp32s3_usb_otg"))]
    let mut display = mipidsi::Display::st7789(di, reset);

    //https://github.com/espressif/esp-box/blob/master/components/bsp/src/boards/esp32_s3_box.c
    #[cfg(any(feature = "esp32s3_box"))]
    let mut display = mipidsi::Display::ili9342c_rgb565(di, reset);
    #[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
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

    #[cfg(any(feature = "esp32s3_box"))]
    display
    .init(
        &mut delay,
        DisplayOptions {
            orientation: Orientation::PortraitInverted(false),
            ..DisplayOptions::default()
        },
    )
    .unwrap();

    // display.clear(RgbColor::WHITE).unwrap();
    println!("Display initialized");

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
    let mut icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    println!("Loading image");

    let ground_data = include_bytes!("../assets/img/ground.bmp");
    let ground_bmp = Bmp::<Rgb565>::from_slice(ground_data).unwrap();

    let wall_data = include_bytes!("../assets/img/wall.bmp");
    let wall_bmp = Bmp::<Rgb565>::from_slice(wall_data).unwrap();

    println!("Rendering maze");

    let mut maze: [u8; 16*16] = [1; 16*16];

    println!("Initializing Random Number Generator Seed");
    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8;32];
    rng.read(&mut seed_buffer).unwrap();

    println!("Acquiring maze generator");
    let mut generator = RbGenerator::new(Some(seed_buffer));
    println!("Generating maze");
    let maze_graph = generator.generate(8, 8).unwrap();

    println!("Converting to tile maze");
    for y in 1usize..8 {
        for x in 1usize..8 {
            let field = maze_graph.get_field(&(x.try_into().unwrap(),y.try_into().unwrap()).into()).unwrap();
            let tile_index = (x-1)*2+(y-1)*2*16+1+16;

            maze[tile_index] = 0;

            if field.has_passage(&Direction::West) {
                maze[tile_index + 1] = 0;
            }

            if field.has_passage(&Direction::South) {
                maze[tile_index + 16] = 0;
            }
        }
    }

    println!("Rendering the maze to display");
    #[cfg(feature = "system_timer")]
    let start_timestamp = SystemTimer::now();

    for x in 0..15 {
        for y in 0..15 {
            let position = Point::new((x*16).try_into().unwrap(), (y*16).try_into().unwrap());
            if maze[x+y*16] == 0 {
                let tile = Image::new(&ground_bmp, position);
                tile.draw(&mut display).unwrap();
            } else {
                let tile = Image::new(&wall_bmp, position);
                tile.draw(&mut display).unwrap();

            }
        }
    }


    #[cfg(feature = "system_timer")]
    let end_timestamp = SystemTimer::now();
    #[cfg(feature = "system_timer")]
    println!("Rendering took: {}ms", (end_timestamp - start_timestamp) / 100000);

    let bmp_data = include_bytes!("../assets/img/ghost1.bmp");
    println!("Transforming image");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
    println!("Drawing image");
    let ghost1 = Image::new(&bmp, Point::new(16, 16));
    ghost1.draw(&mut display).unwrap();
    println!("Image visible");

    println!("Loading 2nd image");
    let bmp_data = include_bytes!("../assets/img/ghost2.bmp");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();

    Text::new(
        "Ready",
        Point::new(90, 140),
        MonoTextStyle::new(&FONT_9X18_BOLD, RgbColor::RED),
    )
    .draw(&mut display)
    .unwrap();

    let mut delay = Delay::new(&clocks);

    let step_size = 16;
    let mut ghost_x = step_size;
    let mut ghost_y = step_size;
    let mut old_x = step_size;
    let mut old_y = step_size;

    #[cfg(any(feature = "imu_controls"))]
    let accel_threshold = 0.20;

    loop {

        old_x = ghost_x;
        old_y = ghost_y;

        #[cfg(any(feature = "imu_controls"))]
        {
            let accel_norm = icm.accel_norm().unwrap();
            let gyro_norm = icm.gyro_norm().unwrap();
            println!(
                "ACCEL = X: {:+.04} Y: {:+.04} Z: {:+.04}",
                accel_norm.x, accel_norm.y, accel_norm.z
            );
            println!(
                "GYRO  = X: {:+.04} Y: {:+.04} Z: {:+.04}",
                gyro_norm.x, gyro_norm.y, gyro_norm.z
            );

            if accel_norm.y > accel_threshold {
                if maze[(ghost_x/16)-1+ghost_y] == 0 {
                    ghost_x -= step_size;
                }
            }

            if accel_norm.y  < -accel_threshold {
                if ghost_x < 16*16 {
                    if maze[(ghost_x/16)+1+ghost_y] == 0 {
                        ghost_x += step_size;
                    }
                }
            }

            if accel_norm.x > accel_threshold {
                if ghost_y < 16*16 {
                    if maze[(ghost_x/16)+ghost_y+step_size] == 0 {
                        ghost_y += step_size;
                    }
                }
            }

            if accel_norm.x < -accel_threshold {
                if ghost_y > 0 {
                    if maze[(ghost_x/16)+ghost_y-step_size] == 0 {
                        ghost_y -= step_size;
                    }
                }
            }
        }

        #[cfg(any(feature = "button_controls"))]
        {
            if button_down_pin.is_low().unwrap() {
                if ghost_x > 0 {
                    if maze[(ghost_x/16)-1+ghost_y] == 0 {
                        ghost_x -= step_size;
                    }
                }
            }

            if button_up_pin.is_low().unwrap() {
                if ghost_x < 16*16 {
                    if maze[(ghost_x/16)+1+ghost_y] == 0 {
                        ghost_x += step_size;
                    }
                }
            }

            if button_menu_pin.is_low().unwrap() {
                if ghost_y > 0 {
                    if maze[(ghost_x/16)+ghost_y-step_size] == 0 {
                        ghost_y -= step_size;
                    }
                }
            }

            if button_ok_pin.is_low().unwrap() {
                if ghost_y < 16*16 {
                    if maze[(ghost_x/16)+ghost_y+step_size] == 0 {
                        ghost_y += step_size;
                    }
                }
            }
        }

        if old_x != ghost_x || old_y != ghost_y {
            let ground = Image::new(&ground_bmp, Point::new(old_x.try_into().unwrap(), old_y.try_into().unwrap()));

            ground.draw(&mut display).unwrap();

            let ghost2 = Image::new(&bmp, Point::new(ghost_x.try_into().unwrap(), ghost_y.try_into().unwrap()));

            ghost2.draw(&mut display).unwrap();
            old_x = ghost_x;
            old_y = ghost_y;
        }
        delay.delay_ms(300u32);
        // ghost1.draw(&mut display).unwrap();
        // delay.delay_ms(500u32);
    }
}
