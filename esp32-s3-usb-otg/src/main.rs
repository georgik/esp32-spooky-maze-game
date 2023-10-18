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

use hal::{
    clock::{ ClockControl, CpuClock },
    // gdma::Gdma,
    peripherals::Peripherals,
    prelude::*,
    spi,
    Rng,
    IO,
    Delay,
};

// systimer was introduced in ESP32-S2, it's not available for ESP32
#[cfg(feature="system_timer")]
use hal::systimer::{SystemTimer};

// use panic_halt as _;
use esp_backtrace as _;

use embedded_graphics::{pixelcolor::Rgb565};
// use esp32s2_hal::Rng;

#[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use spooky_core::{spritebuf::SpriteBuf, engine::Engine, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};

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
        self.engine.start();
    }

    pub fn move_up(&mut self) {
        self.engine.action(Up);
    }

    pub fn move_down(&mut self) {
        self.engine.action(Down);
    }

    pub fn move_left(&mut self) {
        self.engine.action(Left);
    }

    pub fn move_right(&mut self) {
        self.engine.action(Right);
    }

    pub fn teleport(&mut self) {
        self.engine.action(Teleport);
    }

    pub fn place_dynamite(&mut self) {
        self.engine.action(PlaceDynamite);
    }

    pub fn render_frame(&mut self) -> &D {

        self.engine.tick();
        self.engine.draw()

    }

}


#[xtensa_lx_rt::entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 65535*4;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }

    let peripherals = Peripherals::take();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let mut delay = Delay::new(&clocks);
    // self.delay = Some(delay);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // https://espressif-docs.readthedocs-hosted.com/projects/espressif-esp-dev-kits/en/latest/esp32s3/esp32-s3-usb-otg/user_guide.html
    // let button_up = button::Button::new();

    let button_ok_pin = io.pins.gpio0.into_pull_up_input();
    let button_menu_pin = io.pins.gpio14.into_pull_up_input();
    let button_up_pin = io.pins.gpio10.into_pull_up_input();
    let button_down_pin = io.pins.gpio11.into_pull_up_input();

    let mut backlight = io.pins.gpio9.into_push_pull_output();

    let spi = spi::Spi::new(
        peripherals.SPI3,
        io.pins.gpio6,
        io.pins.gpio7,
        io.pins.gpio12,
        io.pins.gpio5,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks);

    backlight.set_high().unwrap();

    let reset = io.pins.gpio18.into_push_pull_output();
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio4.into_push_pull_output());

    #[cfg(any(feature = "esp32s2_ili9341", feature = "esp32_wrover_kit", feature = "esp32c3_ili9341"))]
    let mut delay = Delay::new(&clocks);

    let mut display = mipidsi::Builder::st7789(di)
        .with_display_size(240, 240)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(reset)).unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::BLACK),
    )
    .draw(&mut display)
    .unwrap();


    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8;32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK ; 240*240];
    let fbuf = FrameBuf::new(&mut data, 240, 240);
    let spritebuf = SpriteBuf::new(fbuf);
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new(Some(seed_buffer), engine);
    universe.initialize();

    loop {
        let button_down = button_down_pin.is_low().unwrap();
        let button_up = button_up_pin.is_low().unwrap();
        let button_ok = button_ok_pin.is_low().unwrap();
        let button_menu = button_menu_pin.is_low().unwrap();

        if button_up && button_down {
            universe.teleport();
        } else if button_menu && button_ok {
            universe.place_dynamite();
        } else if button_down {
            universe.move_down();
        } else if button_up {
            universe.move_up();
        } else if button_menu {
            universe.move_left();
        } if button_ok {
            universe.move_right();
        }

        display.draw_iter(universe.render_frame().into_iter()).unwrap();
    }
}
