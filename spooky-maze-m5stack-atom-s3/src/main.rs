#![no_std]
#![no_main]

extern crate alloc;
use alloc::boxed::Box;
use spooky_core::events::{coin::CoinCollisionEvent, player::PlayerInputEvent};
use spooky_core::systems;
use spooky_core::systems::hud::HudState;
use spooky_core::systems::process_player_input::process_player_input;

use bevy::app::AppExit;
use bevy::app::{App, ScheduleRunnerPlugin, Startup, TaskPoolPlugin};
use bevy::ecs::event::Events;
use bevy::prelude::Update;
use bevy::time::TimePlugin;
use bevy_ecs::prelude::*;
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::{
    Blocking,
    gpio::{Level, Output, OutputConfig},
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
    options::{ColorInversion, ColorOrder},
};
use spooky_core::resources::MazeSeed;

// Embedded Graphics imports for our framebuffer drawing.
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;

// ESP-IDF App Descriptor required by newer espflash
esp_bootloader_esp_idf::esp_app_desc!();

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
use mpu6886::Mpu6886;

/// A resource wrapping the accelerometer sensor.
/// (We make this NonSend because hardware sensor drivers typically aren’t Sync.)

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", _info);
    loop {}
}

// For example, define a type alias for your concrete I2C:
// Adjust the lifetime and driver mode if needed.
type I2cMasterBus = I2c<'static, Blocking>;
type I2cMasterBusError = esp_hal::i2c::master::Error;

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
        info!(
            "Allocating framebuffer of size {} bytes",
            LCD_BUFFER_SIZE * 2
        );
        let fb_data: Box<[Rgb565; LCD_BUFFER_SIZE]> = Box::new([Rgb565::BLACK; LCD_BUFFER_SIZE]);
        info!("Framebuffer allocated successfully");
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
use crate::embedded_systems::player_input::AccelerometerResource;
use core::sync::atomic::{AtomicU32, Ordering};
use spooky_core::events::dynamite::DynamiteCollisionEvent;
use spooky_core::events::npc::NpcCollisionEvent;
use spooky_core::events::walker::WalkerCollisionEvent;
use spooky_core::systems::collisions;
// Using bevy's Instant which supports set_elapsed
use bevy_platform::time::Instant;

static ELAPSED: AtomicU32 = AtomicU32::new(0);
fn elapsed_time() -> core::time::Duration {
    core::time::Duration::from_millis(ELAPSED.load(Ordering::Relaxed) as u64)
}

// Event processing is now handled by the TimePlugin and other essential plugins

// ------------------------------------------------------------------------------------
// Our embedded main: initialize HW, set up the game world, and run the schedule.
#[main]
fn main() -> ! {
    // Initialize ESP‑hal peripherals.
    let peripherals = esp_hal::init(esp_hal::Config::default());
    init_logger_from_env();

    // Initialize heap allocator for internal RAM
    esp_alloc::heap_allocator!(size: 180 * 1024);

    info!("Heap allocator initialized");

    // --- DMA Buffers for SPI ---
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(512);
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
    let i2c = I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO38)
        .with_scl(peripherals.GPIO39);
    // let icm_sensor = Icm42670::new(i2c, icm42670::Address::Primary).unwrap();
    let mut icm_sensor = Mpu6886::new(i2c);
    let mut delay = Delay::new();
    match icm_sensor.init(&mut delay) {
        Ok(_) => {
            info!("MPU6886 initialized");
        }
        Err(_) => {
            info!("Failed to initialize MPU6886");
        }
    }

    let hardware_rng = Rng::new();
    let mut seed = [0u8; 32];
    hardware_rng.read(&mut seed);

    // --- Build the Bevy app with minimal essential plugins for embedded
    let mut app = App::new();

    // Add essential plugins for event processing to work
    app.add_plugins((
        TaskPoolPlugin::default(),       // Required for system scheduling
        TimePlugin::default(),           // Required for frame timing and updates
        ScheduleRunnerPlugin::default(), // Required since we don't have windowing
    ));

    // Initialize the app properly for our minimal setup
    app.init_resource::<Events<AppExit>>(); // Required for basic app functionality

    app.insert_non_send_resource(DisplayResource { display })
        .insert_non_send_resource(AccelerometerResource { sensor: icm_sensor })
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
                player_input::dispatch_accelerometer_input::<I2cMasterBus, I2cMasterBusError>,
                process_player_input,
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
                render_system,
            ),
        );

    let mut loop_delay = Delay::new();
    loop {
        app.update();
        info!("tick");
        loop_delay.delay_ms(300u32);
    }
}
