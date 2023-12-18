#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::{
    prelude::{Point, RgbColor},
    mono_font::{
        ascii::FONT_8X13,
        MonoTextStyle,
    },
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ ClockControl, CpuClock },
    dma::DmaPriority,
    gdma::Gdma,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Rng,
    IO,
    Delay,
};

use esp_backtrace as _;

use embedded_graphics::pixelcolor::Rgb565;

use spooky_core::{spritebuf::SpriteBuf, engine::Engine, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite }};

use embedded_hal::digital::v2::OutputPin;
use embedded_graphics_framebuf::FrameBuf;

pub struct Universe<D> {
    pub engine: Engine<D>,
}


impl <D:embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Universe <D> {
    pub fn new(_seed: Option<[u8; 32]>, engine:Engine<D>) -> Universe<D> {
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

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 65535*4;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_mut_ptr(), HEAP_SIZE) }

    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // https://docs.espressif.com/projects/espressif-esp-dev-kits/en/latest/esp32s3/esp32-s3-usb-otg/user_guide.html

    let button_ok_pin = io.pins.gpio0.into_pull_up_input();
    let button_menu_pin = io.pins.gpio14.into_pull_up_input();
    let button_up_pin = io.pins.gpio10.into_pull_up_input();
    let button_down_pin = io.pins.gpio11.into_pull_up_input();

    let lcd_h_res = 240;
    let lcd_v_res = 240;

    let lcd_sclk = io.pins.gpio6;
    let lcd_mosi = io.pins.gpio7;
    let lcd_miso = io.pins.gpio12; // random unused pin
    let lcd_cs = io.pins.gpio5;
    let lcd_dc = io.pins.gpio4.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio9.into_push_pull_output();
    let lcd_reset = io.pins.gpio18.into_push_pull_output();

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let spi = Spi::new(
        peripherals.SPI2,
        60u32.MHz(),
        SpiMode::Mode0,
        &clocks
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

    lcd_backlight.set_high().unwrap();

    let di = SPIInterfaceNoCS::new(spi, lcd_dc);

    delay.delay_ms(500u32);

    let mut display = mipidsi::Builder::st7789(di)
        .with_display_size(lcd_h_res, lcd_v_res)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(lcd_reset)).unwrap();

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
    let mut data = [Rgb565::BLACK ; 240 * 240];
    let fbuf = FrameBuf::new(&mut data, lcd_h_res.into(), lcd_v_res.into());
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

        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0, 0, lcd_h_res, lcd_v_res, pixel_iterator);
    }
}
