#![no_std]
#![no_main]

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
use esp32s2_hal::{
    clock::ClockControl,
    pac::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    RtcCntl,
    IO,
    Delay,
    systimer::{SystemTimer},
};
use panic_halt as _;
use xtensa_lx_rt::entry;

use embedded_graphics::{image::Image, pixelcolor::Rgb565};
use tinybmp::Bmp;
// use esp32s2_hal::Rng;

#[cfg(feature = "esp32s2_ili9341")]
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.SYSTEM.split();
    let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc_cntl.set_wdt_global_enable(false);
    wdt0.disable();
    wdt1.disable();

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let backlight = io.pins.gpio9;
    let mut backlight = backlight.into_push_pull_output();
    backlight.set_high().unwrap();

    let mosi = io.pins.gpio7;
    let cs = io.pins.gpio5;
    let rst = io.pins.gpio8;
    let dc = io.pins.gpio4;
    let sck = io.pins.gpio6;
    let miso = io.pins.gpio12;

    let spi = spi::Spi::new(
        peripherals.SPI3,
        sck,
        mosi,
        miso,
        cs,
        100000u32.kHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &mut clocks,
    );

    let di = SPIInterfaceNoCS::new(spi, dc.into_push_pull_output());
    let reset = rst.into_push_pull_output();
    let mut delay = Delay::new(&clocks);

    #[cfg(feature="esp32s2_usb_otg")]
    let mut display = st7789::ST7789::new(di, reset, 240, 240);
    #[cfg(feature="esp32s2_ili9341")]
    let mut display = Ili9341::new(di, reset, &mut delay, Orientation::Portrait, DisplaySize240x320).unwrap();


    #[cfg(feature="esp32s2_usb_otg")]
    display.init(&mut delay).unwrap();
    #[cfg(feature="esp32s2_usb_otg")]
    display.set_orientation(st7789::Orientation::Portrait).unwrap();

    // display.clear(RgbColor::WHITE).unwrap();
    println!("Initialized");

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::BLACK),
    )
    .draw(&mut display)
    .unwrap();

    println!("Loading image");

    let ground_data = include_bytes!("../assets/img/ground.bmp");
    let ground_bmp = Bmp::<Rgb565>::from_slice(ground_data).unwrap();

    let wall_data = include_bytes!("../assets/img/wall.bmp");
    let wall_bmp = Bmp::<Rgb565>::from_slice(wall_data).unwrap();

    println!("Rendering maze");
    // let mut rng = Rng::new(peripherals.RNG);
    // let mut buffer = [0u8;8];
    // rng.read(&mut buffer).unwrap();

    let maze: [u8; 16*16] = [
        1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
        1,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,
        1,1,1,0,1,1,1,1,1,0,1,0,1,1,0,1,
        1,0,0,0,1,0,0,0,1,0,1,0,1,0,0,1,
        1,1,1,1,1,0,1,1,1,0,1,0,1,1,1,1,
        1,0,1,0,0,0,0,0,0,0,1,0,0,0,0,1,
        1,0,1,0,1,1,1,1,1,1,1,1,1,1,0,1,
        1,0,1,0,1,0,0,0,0,0,1,0,0,0,0,1,
        1,0,1,0,1,1,1,0,1,0,1,0,0,1,0,1,
        1,0,1,0,0,0,0,0,1,0,0,0,0,1,0,1,
        1,0,1,1,1,1,1,1,1,1,1,1,1,1,0,1,
        1,0,0,0,1,0,0,0,0,0,0,0,0,1,0,1,
        1,0,1,0,1,0,1,1,1,1,1,1,0,1,0,1,
        1,0,1,0,1,0,1,0,0,1,0,1,0,1,0,1,
        1,0,1,0,0,0,0,0,0,0,0,1,0,0,0,1,
        1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1
    ];

    let start_timestamp = SystemTimer::now();

    for x in 0..15 {
        for y in 0..15 {
            let position = Point::new((x*16).try_into().unwrap(), (y*16).try_into().unwrap());
            if maze[x+y*16] == 1 {
                let tile = Image::new(&wall_bmp, position);
                tile.draw(&mut display).unwrap();
            } else {
                let tile = Image::new(&ground_bmp, position);
                tile.draw(&mut display).unwrap();

            }
        }
    }
    let end_timestamp = SystemTimer::now();
    println!("Rendering took: {}ms", (end_timestamp - start_timestamp) / 100000);

    let bmp_data = include_bytes!("../assets/img/ghost1.bmp");
    println!("Transforming image");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
    println!("Drawing image");
    let ghost1 = Image::new(&bmp, Point::new(10, 20));
    ghost1.draw(&mut display).unwrap();
    println!("Image visible");

    println!("Loading 2nd image");
    let bmp_data = include_bytes!("../assets/img/ghost2.bmp");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
    let ghost2 = Image::new(&bmp, Point::new(10, 20));

    Text::new(
        "Ready",
        Point::new(90, 140),
        MonoTextStyle::new(&FONT_9X18_BOLD, RgbColor::RED),
    )
    .draw(&mut display)
    .unwrap();
    let mut delay = Delay::new(&clocks);
    loop {
        ghost2.draw(&mut display).unwrap();
        delay.delay_ms(500u32);
        ghost1.draw(&mut display).unwrap();
        delay.delay_ms(500u32);
    }
}
