// #![no_std]
// #![no_main]

use std::time::{Duration, SystemTime, UNIX_EPOCH};
// use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Sector},
    image::Image,
};
use embedded_graphics_web_simulator::{
    display::WebSimulatorDisplay, output_settings::OutputSettingsBuilder,
};

use wasm_bindgen::prelude::*;
use web_sys::console;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;

use embedded_graphics::{
    prelude::RgbColor,
    mono_font::{
        ascii::{FONT_8X13, FONT_9X18_BOLD},
        MonoTextStyle,
    },
    prelude::Point,
    text::{Alignment, Text},
    Drawable,
};
use gloo_timers::callback::Timeout;

use tinybmp::Bmp;

use maze_generator::prelude::*;
use maze_generator::recursive_backtracking::{RbGenerator};

// use esp32s3_hal::{
//     clock::ClockControl,
//     pac::Peripherals,
//     prelude::*,
//     spi,
//     timer::TimerGroup,
//     Rtc,
//     IO,
//     Delay,
// };

// use esp_println::println;

// use mipidsi::DisplayOptions;

// #[allow(unused_imports)]
// use esp_backtrace as _;

// use xtensa_lx_rt::entry;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn perf_to_system(amt: f64) -> SystemTime {
    let secs = (amt as u64) / 1_000;
    let nanos = (((amt as u64) % 1_000) as u32) * 1_000_000;
    UNIX_EPOCH + Duration::new(secs, nanos)
}

const NUM_ITER: i32 = 6;
// fn document() -> web_sys::Document {
//     window()
//         .document()
//         .expect("should have a document on window")
// }
// #[entry]
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // let peripherals = Peripherals::take().unwrap();
    // let mut system = peripherals.SYSTEM.split();
    // let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    // let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    // let mut wdt0 = timer_group0.wdt;
    // let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    // let mut wdt1 = timer_group1.wdt;

    // rtc.rwdt.disable();

    // wdt0.disable();
    // wdt1.disable();

    // let mut delay = Delay::new(&clocks);

    // let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // let sclk = io.pins.gpio7;
    // let mosi = io.pins.gpio6;

    // let spi = spi::Spi::new_no_cs_no_miso(
    //     peripherals.SPI2,
    //     sclk,
    //     mosi,
    //     4u32.MHz(),
    //     spi::SpiMode::Mode0,
    //     &mut system.peripheral_clock_control,
    //     &clocks,
    // );

    // let mut backlight = io.pins.gpio45.into_push_pull_output();
    // backlight.set_high().unwrap();

    // let reset = io.pins.gpio48.into_push_pull_output();

    // let di = SPIInterfaceNoCS::new(spi, io.pins.gpio4.into_push_pull_output());

    // let display_options = DisplayOptions {
    //     orientation: mipidsi::Orientation::PortraitInverted(false),
    //     ..Default::default()
    // };

    let document = web_sys::window().unwrap().document().unwrap();
    let output_settings = OutputSettingsBuilder::new()
        .scale(1)
        .pixel_spacing(1)
        .build();
        let mut display:WebSimulatorDisplay<Rgb565> = WebSimulatorDisplay::new(
            (320, 240),
            &output_settings,
            document.get_element_by_id("graphics").as_ref(),
        );
    // let mut window = Window::new("ESP32-S3-BOX", &output_settings);
    // let mut display = mipidsi::Display::ili9342c_rgb565(di, core::prelude::v1::Some(reset), display_options);
    // display.init(&mut delay).unwrap();

    display.clear(Rgb565::WHITE).unwrap();
    // let espressif_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_10X20)
    //     .text_color(RgbColor::BLACK)
    //     .build();

    // Text::with_alignment("HELLO WORLD!", Point::new(160, 120), espressif_style,  Alignment::Center)
    //     .draw(&mut display)
    //     .unwrap();

    display.flush().unwrap();
    
    // println!("Hello World!");

    println!("Loading image");

    let ground_data = include_bytes!("../../assets/img/ground.bmp");
    let ground_bmp = Bmp::<Rgb565>::from_slice(ground_data).unwrap();

    let wall_data = include_bytes!("../../assets/img/wall.bmp");
    let wall_bmp = Bmp::<Rgb565>::from_slice(wall_data).unwrap();

    println!("Rendering maze");

    // Dimension of tiles
    const TILE_WIDTH:usize = 16;
    const TILE_HEIGHT:usize = 16;

    // Simplified maze map in memory for tile mapping
    // #[cfg(any(feature = "esp32s3_box"))]
    const MAZE_WIDTH:usize = 21;
    // #[cfg(not(feature = "esp32s3_box"))]
    // const MAZE_WIDTH:usize = 16;
    const MAZE_HEIGHT:usize = 16;

    // Tile map should have small border top line and left column
    const MAZE_OFFSET:usize = MAZE_WIDTH + 1;

    // Dimension of Playground
    const PLAYGROUND_WIDTH:usize = MAZE_WIDTH * TILE_WIDTH;
    const PLAYGROUND_HEIGHT:usize = MAZE_HEIGHT * MAZE_HEIGHT;

    // Dimensions of maze graph produced by algorithm
    // #[cfg(any(feature = "esp32s3_box"))]
    const MAZE_GRAPH_WIDTH:usize = 10;
    // #[cfg(not(feature = "esp32s3_box"))]
    // const MAZE_GRAPH_WIDTH:usize = 8;
    const MAZE_GRAPH_HEIGHT:usize = 8;

    let mut maze: [u8; MAZE_WIDTH*MAZE_HEIGHT] = [1; MAZE_WIDTH*MAZE_HEIGHT];

    println!("Initializing Random Number Generator Seed");
    // let mut rng = Rng::new(peripherals.RNG);
    // let mut rng = Rng::new( 0x12345678 );
    let mut seed_buffer = [0u8;32];
    // rng.read(&mut seed_buffer).unwrap();

    println!("Acquiring maze generator");
    let mut generator = RbGenerator::new(Some(seed_buffer));
    println!("Generating maze");
    let maze_graph = generator.generate(MAZE_GRAPH_WIDTH as i32, MAZE_GRAPH_HEIGHT as i32).unwrap();

    println!("Converting to tile maze");
    for y in 1usize..MAZE_GRAPH_HEIGHT {
        for x in 1usize..MAZE_GRAPH_WIDTH {
            let field = maze_graph.get_field(&(x.try_into().unwrap(),y.try_into().unwrap()).into()).unwrap();
            let tile_index = (x-1)*2+(y-1)*2*MAZE_WIDTH+MAZE_OFFSET;

            maze[tile_index] = 0;

            if field.has_passage(&Direction::West) {
                maze[tile_index + 1] = 0;
            }

            if field.has_passage(&Direction::South) {
                maze[tile_index + MAZE_WIDTH] = 0;
            }
        }
    }

    println!("Rendering the maze to display");
    #[cfg(feature = "system_timer")]
    let start_timestamp = SystemTimer::now();

    for x in 0..(MAZE_WIDTH-1) {
        for y in 0..(MAZE_HEIGHT-1) {
            let position = Point::new((x*TILE_WIDTH).try_into().unwrap(), (y*TILE_HEIGHT).try_into().unwrap());
            if maze[x+y*MAZE_WIDTH] == 0 {
                let tile = Image::new(&ground_bmp, position);
                tile.draw(&mut display).unwrap();
            } else {
                let tile = Image::new(&wall_bmp, position);
                tile.draw(&mut display).unwrap();

            }
        }
    }


    let bmp_data = include_bytes!("../../assets/img/ghost1.bmp");
    println!("Transforming image");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
    println!("Drawing image");
    let ghost1 = Image::new(&bmp, Point::new(16, 16));
    ghost1.draw(&mut display).unwrap();
    println!("Image visible");

    println!("Loading 2nd image");
    let bmp_data = include_bytes!("../../assets/img/ghost2.bmp");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();

    Text::new(
        "Ready",
        Point::new(90, 140),
        MonoTextStyle::new(&FONT_9X18_BOLD, RgbColor::RED),
    )
    .draw(&mut display)
    .unwrap();

    // let mut delay = Delay::new(&clocks);

    let mut ghost_x = TILE_HEIGHT;
    let mut ghost_y = TILE_WIDTH;
    let mut old_x = ghost_x;
    let mut old_y = ghost_y;

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    let performance = window()
    .performance()
    .expect("performance should be available");
    let mut start = (performance.now() as u64) / 1000;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let end = (performance.now() as u64) / 1000;
        if start != end {
            start = end;
        
        if i > NUM_ITER {
            // text_container().set_text_content(Some("All done!"));

            // Drop our handle to this closure so that it will get cleaned
            // up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
        ghost_x += TILE_WIDTH;
        let ghost1 = Image::new(&bmp, Point::new(ghost_x.try_into().unwrap(), 16));
        ghost1.draw(&mut display).unwrap();
        display.flush().unwrap();
    }
        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
        // i += 1;
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
