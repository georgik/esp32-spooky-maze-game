#![no_std]
#![no_main]

extern crate alloc;
use spooky_core::systems::player_input::PlayerInputEvent;
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
    i2c::master::I2c,
    time::Rate,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_hal::delay::DelayNs;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::delay::Delay;
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

// Bring in our custom render system from our embedded module.
mod embedded_systems {
    pub mod render;
    pub mod player_input;
}
use embedded_systems::render::render_system;

// Bring in our heapbuffer helper.
mod heapbuffer;

use crate::heapbuffer::HeapBuffer;

// --- NEW: Imports for the ICM-42670 accelerometer ---
use icm42670::prelude::*;
use icm42670::Icm42670;

/// A resource wrapping the accelerometer sensor.
/// (We make this NonSend because hardware sensor drivers typically aren’t Sync.)


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", _info);
    loop {}
}

// For example, define a type alias for your concrete I2C:
// Adjust the lifetime and driver mode if needed.
type MyI2c = esp_hal::i2c::master::I2c<'static, Blocking>;
type MyI2cError = esp_hal::i2c::master::Error;

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

use core::sync::atomic::{Ordering};
use bevy::platform_support::sync::atomic::AtomicU64;
use bevy::platform_support::time::Instant;
use spooky_core::systems::setup::NoStdTransform;
use crate::embedded_systems::player_input::AccelerometerResource;

static ELAPSED: AtomicU64 = AtomicU64::new(0);
fn elapsed_time() -> core::time::Duration {
    core::time::Duration::from_nanos(ELAPSED.load(Ordering::Relaxed))
}

// ------------------------------------------------------------------------------------
// Our embedded main: initialize HW, set up the game world, and run the schedule.
#[main]
fn main() -> ! {
    // Initialize ESP‑hal peripherals.
    let peripherals = esp_hal::init(esp_hal::Config::default());
    init_logger_from_env();
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
    let mut display: MyDisplay = Builder::new(ILI9486Rgb565, di)
        .reset_pin(reset)
        .display_size(320, 240)
        .color_order(ColorOrder::Bgr)
        .init(&mut display_delay)
        .unwrap();
    display.clear(Rgb565::BLUE).unwrap();
    let mut backlight = Output::new(peripherals.GPIO47, Level::Low, OutputConfig::default());
    backlight.set_high();
    info!("Display initialized");
    unsafe { Instant::set_elapsed(elapsed_time) };

    // --- Initialize the accelerometer sensor.
    const DEVICE_ADDR: u8 = 0x77;
    let mut i2c = I2c::new( peripherals.I2C0, esp_hal::i2c::master::Config::default(), ).unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO18);
    let icm_sensor = Icm42670::new(i2c, icm42670::Address::Primary).unwrap();

    // --- Build the Bevy app.
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    // Insert the accelerometer resource.
    app.insert_non_send_resource(AccelerometerResource { sensor: icm_sensor });

    app.insert_non_send_resource(DisplayResource { display });

    app.insert_resource(FrameBufferResource::new());
    // Register the custom event.
    app.add_event::<PlayerInputEvent>();

    // Use spooky_core's setup system to spawn the maze, coins, and player.
    app.add_systems(Startup, systems::setup::setup);
    // Add game logic system.
    app.add_systems(Update, spooky_core::systems::game_logic::update_game);
    // Add the embedded render system.
    app.add_systems(Update, render_system);
    // Add the accelerometer dispatch system (from the embedded module).
    app.add_systems(
        Update,
        crate::embedded_systems::player_input::dispatch_accelerometer_input::<MyI2c, MyI2cError>,
    );

    // Add the common event processing system.
    app.add_systems(Update, spooky_core::systems::player_input::process_player_input);

    let mut loop_delay = Delay::new();
    loop {
        app.update();
        loop_delay.delay_ms(50u32);
    }
}
