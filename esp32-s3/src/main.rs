#![no_std]
#![no_main]

extern crate alloc;
use alloc::boxed::Box;

use core::fmt::Write;
use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::Update;
use bevy_ecs::prelude::*;
use esp_hal::{
    Blocking,
    gpio::{DriveMode, Level, Output, OutputConfig},
    main,
    rng::Rng,
    spi::master::{Spi, SpiDmaBus},
    time::Rate,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_hal::delay::DelayNs;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::delay::Delay;
// use esp_hal_bus::spi::ExclusiveDevice;
use esp_println::{logger::init_logger_from_env, println};
use log::info;
use mipidsi::{Builder, models::ILI9486Rgb565};
use mipidsi::{
    interface::SpiInterface,
    options::{ColorOrder},
};
use spooky_core::maze::Maze;
use spooky_core::resources::{MazeResource, PlayerPosition};
use spooky_core::systems;
use spooky_core::components::Player;

// Embedded Graphics imports for our framebuffer drawing.
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_framebuf::backends::FrameBufferBackend;


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", _info);
    loop {}
}

// ------------------------------------------------------------------------------------
// A simple Heap‑allocated framebuffer backend for drawing to our LCD.
// (Similar to the Conway’s game of life example.)
pub struct HeapBuffer<C: PixelColor, const N: usize>(Box<[C; N]>);

impl<C: PixelColor, const N: usize> HeapBuffer<C, N> {
    pub fn new(data: Box<[C; N]>) -> Self {
        Self(data)
    }
}

impl<C: PixelColor, const N: usize> core::ops::Deref for HeapBuffer<C, N> {
    type Target = [C; N];
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<C: PixelColor, const N: usize> core::ops::DerefMut for HeapBuffer<C, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<C: PixelColor, const N: usize> FrameBufferBackend for HeapBuffer<C, N> {
    type Color = C;
    fn set(&mut self, index: usize, color: Self::Color) {
        self.0[index] = color;
    }
    fn get(&self, index: usize) -> Self::Color {
        self.0[index]
    }
    fn nr_elements(&self) -> usize {
        N
    }
}

// ------------------------------------------------------------------------------------
// LCD resolution and framebuffer definitions.
const LCD_H_RES: usize = 320;
const LCD_V_RES: usize = 240;
const LCD_BUFFER_SIZE: usize = LCD_H_RES * LCD_V_RES;

type FbBuffer = HeapBuffer<Rgb565, LCD_BUFFER_SIZE>;
type MyFrameBuf = FrameBuf<Rgb565, FbBuffer>;

#[derive(Resource)]
struct FrameBufferResource {
    frame_buf: MyFrameBuf,
}

impl FrameBufferResource {
    fn new() -> Self {
        let fb_data: Box<[Rgb565; LCD_BUFFER_SIZE]> = Box::new([Rgb565::BLACK; LCD_BUFFER_SIZE]);
        let heap_buffer = HeapBuffer::new(fb_data);
        let frame_buf = MyFrameBuf::new(heap_buffer, LCD_H_RES, LCD_V_RES);
        Self { frame_buf }
    }
}

// Use the DMA-enabled SPI bus type.
type MyDisplay = mipidsi::Display<
    SpiInterface<
        'static,
        ExclusiveDevice<SpiDmaBus<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ILI9486Rgb565,
    Output<'static>,
>;

struct DisplayResource {
    display: MyDisplay,
}

// ------------------------------------------------------------------------------------
// A simple render system that draws the maze tile map.
// (This example assumes a basic color mapping; you could extend this to use textures.)
fn render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut fb_res: ResMut<FrameBufferResource>,
    maze_res: Res<MazeResource>,
) {
    fb_res.frame_buf.clear(Rgb565::BLACK).unwrap();
    let maze = &maze_res.maze;
    let (left, bottom, _right, _top) = maze.playable_bounds();
    for ty in 0..maze.height as i32 {
        for tx in 0..maze.width as i32 {
            // Maze data: note that row 0 is the top row in the maze data.
            let maze_row = (maze.height as i32 - 1) - ty;
            let index = (maze_row * maze.width as i32 + tx) as usize;
            let color = match maze.data[index] {
                1 => Rgb565::BLUE,   // wall
                0 => Rgb565::GREEN,  // ground
                2 => Rgb565::RED,    // scorched
                _ => Rgb565::GREEN,
            };
            let x = left + tx * maze.tile_width as i32;
            let y = bottom + ty * maze.tile_height as i32;
            let rect = embedded_graphics::primitives::Rectangle::new(
                Point::new(x, y),
                Size::new(maze.tile_width, maze.tile_height),
            )
                .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_fill(color));
            rect.draw(&mut fb_res.frame_buf).unwrap();
        }
    }
    let area = Rectangle::new(Point::zero(), fb_res.frame_buf.size());
    display_res
        .display
        .fill_contiguous(&area, fb_res.frame_buf.data.iter().copied())
        .unwrap();
}

use core::sync::atomic::{Ordering};
use bevy::platform_support::sync::atomic::AtomicU64;
use bevy::platform_support::time::Instant;
use embedded_graphics::primitives::Rectangle;

static ELAPSED: AtomicU64 = AtomicU64::new(0);

fn elapsed_time() -> core::time::Duration {
    // Return the monotonic elapsed time as a Duration.
    core::time::Duration::from_nanos(ELAPSED.load(Ordering::Relaxed))
}

// ------------------------------------------------------------------------------------
// Our embedded main: initialize HW, set up the game world, and run the schedule.
#[main]
fn main() -> ! {
    // Initialize ESP‑hal peripherals.
    let peripherals = esp_hal::init(esp_hal::Config::default());

    init_logger_from_env();

    // Allocate PSRAM for the heap.
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    // --- DMA Buffers for SPI ---
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(8912);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // --- Initialize SPI.
    let spi = Spi::<Blocking>::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(esp_hal::spi::Mode::_0),
    )
        .unwrap()
        .with_sck(peripherals.GPIO7)
        .with_mosi(peripherals.GPIO6)
        .with_dma(peripherals.DMA_CH0)
        .with_buffers(dma_rx_buf, dma_tx_buf);
    let cs_output = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, cs_output, spi_delay).unwrap();

    // LCD interface: use GPIO4 for DC.
    let lcd_dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
    let di = SpiInterface::new(spi_device, lcd_dc, buffer);

    let mut display_delay = Delay::new();
    display_delay.delay_ns(500_000u32);

    // Reset pin for display: GPIO48 (OpenDrain required).
    let reset = Output::new(
        peripherals.GPIO48,
        Level::High,
        OutputConfig::default().with_drive_mode(DriveMode::OpenDrain),
    );
    // Initialize the display using mipidsi.
    let mut display: mipidsi::Display<
        SpiInterface<
            'static,
            ExclusiveDevice<SpiDmaBus<'static, Blocking>, Output<'static>, Delay>,
            Output<'static>,
        >,
        ILI9486Rgb565,
        Output<'static>,
    > = Builder::new(ILI9486Rgb565, di)
        .reset_pin(reset)
        .display_size(320, 240)
        .color_order(ColorOrder::Bgr)
        .init(&mut display_delay)
        .unwrap();
    display.clear(Rgb565::BLUE).unwrap();

    // Turn on backlight (GPIO47).
    let mut backlight = Output::new(peripherals.GPIO47, Level::Low, OutputConfig::default());
    backlight.set_high();

    info!("Display initialized");

    unsafe {Instant::set_elapsed(elapsed_time);}
    // Build the schedule.
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
    ));
    app.insert_non_send_resource(DisplayResource { display });
    app.insert_resource(FrameBufferResource::new());

    app.add_systems(Startup, systems::setup::setup);
    app.add_systems(Update, spooky_core::systems::game_logic::update_game);
    app.add_systems(Update, render_system);


    // Create a delay for our main loop.
    let mut loop_delay = Delay::new();

    loop {
        app.update();
        loop_delay.delay_ms(50u32);
    }
}
