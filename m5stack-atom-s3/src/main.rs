#![no_std]
#![no_main]

extern crate alloc;
use alloc::boxed::Box;
use spooky_core::events::{coin::CoinCollisionEvent, player::PlayerInputEvent};
use spooky_core::systems;
use spooky_core::systems::hud::HudState;
use spooky_core::systems::process_player_input::process_player_input;

use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::prelude::Update;
use bevy_ecs::prelude::*;
use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::{
    Blocking,
    gpio::{DriveMode, Level, Output, OutputConfig},
    i2c::master::I2c,
    main,
    rng::Rng,
    spi::master::{Spi, SpiDmaBus},
    time::Rate,
};
use esp_println::{logger::init_logger_from_env, println};
use log::info;
use mipidsi::{Builder, models::GC9A01};
use mipidsi::{
    interface::SpiInterface,
    options::{ColorInversion, ColorOrder, Orientation},
};
use spooky_core::components::Player;
use spooky_core::maze::Maze;
use spooky_core::resources::{MazeResource, MazeSeed, PlayerPosition};

// Embedded Graphics imports for our framebuffer drawing.
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_framebuf::backends::FrameBufferBackend;

// Bring in our custom render system from our embedded module.
mod embedded_systems {
    pub mod player_input;
    pub mod render;
}
use embedded_systems::render::render_system;

// Bring in our heapbuffer helper.
mod heapbuffer;

use crate::heapbuffer::HeapBuffer;

// --- NEW: Imports for the ICM-42670 accelerometer ---
// use icm42670::Icm42670;
// use icm42670::prelude::*;

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
const LCD_H_RES: usize = 130;
const LCD_V_RES: usize = 129;
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
    GC9A01,
    Output<'static>,
>;

struct DisplayResource {
    display: MyDisplay,
}

use crate::embedded_systems::player_input;
// use crate::embedded_systems::player_input::AccelerometerResource;
use bevy::platform_support::sync::atomic::AtomicU64;
use bevy::platform_support::time::Instant;
use core::sync::atomic::Ordering;
use spooky_core::events::dynamite::DynamiteCollisionEvent;
use spooky_core::events::npc::NpcCollisionEvent;
use spooky_core::events::walker::WalkerCollisionEvent;
use spooky_core::systems::collisions;
use spooky_core::systems::setup::NoStdTransform;

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
    // esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    esp_alloc::heap_allocator!(size: 160 * 1024);

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
        .with_sck(peripherals.GPIO17)
        .with_mosi(peripherals.GPIO21)
        .with_dma(peripherals.DMA_CH0)
        .with_buffers(dma_rx_buf, dma_tx_buf);
    let cs_output = Output::new(peripherals.GPIO15, Level::High, OutputConfig::default());
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, cs_output, spi_delay).unwrap();

    // LCD interface: use GPIO4 for DC.
    let lcd_dc = Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default());
    let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
    let di = SpiInterface::new(spi_device, lcd_dc, buffer);

    let mut display_delay = Delay::new();
    display_delay.delay_ns(500_000u32);

    // Reset pin for display
    let reset = Output::new(peripherals.GPIO34, Level::High, OutputConfig::default());

    let mut display: MyDisplay = Builder::new(GC9A01, di)
        .reset_pin(reset)
        .display_size(130, 129)
        // .orientation(Orientation::new().flip_horizontal())
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut display_delay)
        .unwrap();
    display.clear(Rgb565::BLUE).unwrap();

    let mut backlight = Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default());
    backlight.set_high();

    info!("Display initialized");
    unsafe { Instant::set_elapsed(elapsed_time) };

    // --- Initialize the accelerometer sensor.
    // TODO: MPU6886
    // const DEVICE_ADDR: u8 = 0x77;
    // let mut i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
    //     .unwrap()
    //     .with_sda(peripherals.GPIO8)
    //     .with_scl(peripherals.GPIO18);
    // let icm_sensor = Icm42670::new(i2c, icm42670::Address::Primary).unwrap();

    let mut hardware_rng = Rng::new(peripherals.RNG);
    let mut seed = [0u8; 32];
    hardware_rng.read(&mut seed);

    // --- Build the Bevy app.
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,))
        .insert_non_send_resource(DisplayResource { display })
        // .insert_non_send_resource(AccelerometerResource { sensor: icm_sensor })
        .insert_resource(FrameBufferResource::new())
        .insert_resource(HudState::default())
        .insert_resource(MazeSeed(Some(seed)))
        .add_systems(Startup, systems::setup::setup)
        .add_event::<PlayerInputEvent>()
        .add_event::<CoinCollisionEvent>()
        .add_event::<DynamiteCollisionEvent>()
        .add_event::<WalkerCollisionEvent>()
        .add_event::<NpcCollisionEvent>()
        .add_systems(
            Update,
            (
                // player_input::dispatch_accelerometer_input::<MyI2c, MyI2cError>,
                // systems::process_player_input::process_player_input,
                collisions::coin::detect_coin_collision,
                collisions::coin::remove_coin_on_collision,
                collisions::dynamite::handle_dynamite_collision,
                collisions::walker::detect_walker_collision,
                collisions::walker::handle_walker_collision,
                collisions::npc::detect_npc_collision,
                collisions::npc::handle_npc_collision,
                systems::dynamite_logic::handle_dynamite_collision,
                systems::npc_logic::update_npc_movement,
                systems::game_logic::update_game,
                embedded_systems::render::render_system,
            ),
        )
        .run();

    let mut loop_delay = Delay::new();
    loop {
        app.update();
        loop_delay.delay_ms(50u32);
    }
}
