#![no_std]
#![no_main]

extern crate alloc;
use alloc::boxed::Box;
use spooky_core::events::{coin::CoinCollisionMessage, player::PlayerInputMessage};
use spooky_core::systems;
use spooky_core::systems::hud::HudState;
use spooky_core::systems::process_player_input::process_player_input;

use bevy::app::{App, ScheduleRunnerPlugin, Startup, TaskPoolPlugin};
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
    i2c::master::{Config as I2cConfig, I2c},
    main,
    rng::Rng,
    spi::master::{Spi, SpiDmaBus},
    time::Rate,
};
use esp_println::{logger::init_logger_from_env, println};
use log::{error, info};
use mipidsi::{Builder, models::ILI9342CRgb565};
use mipidsi::{interface::SpiInterface, options::{ColorInversion, ColorOrder}};
use spooky_core::resources::MazeSeed;

// Embedded Graphics imports for our framebuffer drawing.
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;

// M5Stack CoreS3 Power Management and GPIO Expander
use axp2101::{Axp2101, I2CPowerManagementInterface};
use aw9523::{Aw9523, I2CGpioExpanderInterface};

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

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", _info);
    loop {}
}

// ------------------------------------------------------------------------------------
// LCD resolution and framebuffer definitions.
// CoreS3 has 320x240 display
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
    ILI9342CRgb565,
    Output<'static>,
>;

struct DisplayResource {
    display: MyDisplay,
}

use core::sync::atomic::{AtomicU32, Ordering};
use spooky_core::events::dynamite::DynamiteCollisionMessage;
use spooky_core::events::npc::NpcCollisionMessage;
use spooky_core::events::walker::WalkerCollisionMessage;
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

    // PSRAM allocator for heap memory - needed for larger framebuffer on CoreS3
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    info!("PSRAM allocator initialized");

    // --- DMA Buffers for SPI ---
    // CoreS3 needs larger buffer for 320x240 display
    #[allow(clippy::manual_div_ceil)]
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(8912);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // --------------------------------------------------------------------------------
    // POWER MANAGEMENT - CRITICAL for CoreS3
    // Must be initialized BEFORE display
    // --------------------------------------------------------------------------------

    // --- Initialize I2C bus for power management (GPIO12=SDA, GPIO11=SCL) ---
    let i2c_bus = I2c::new(
        peripherals.I2C0,
        I2cConfig::default(),
    )
    .unwrap()
    .with_sda(peripherals.GPIO12)
    .with_scl(peripherals.GPIO11);

    // --- Initialize AXP2101 Power Management IC ---
    info!("Initializing AXP2101 Power Management IC");
    let axp_interface = I2CPowerManagementInterface::new(i2c_bus);
    let mut axp = Axp2101::new(axp_interface);
    match axp.init() {
        Ok(_) => info!("AXP2101 initialized successfully"),
        Err(e) => error!("AXP2101 initialization failed (continuing anyway): {:?}", e),
    }

    // --- Initialize AW9523 GPIO Expander ---
    info!("Initializing AW9523 GPIO Expander");
    // Get the I2C interface back by consuming the AXP2101
    let i2c_bus = axp.release_i2c();

    let aw_interface = I2CGpioExpanderInterface::new(i2c_bus);
    let mut aw = Aw9523::new(aw_interface);
    match aw.init() {
        Ok(_) => info!("AW9523 initialized successfully"),
        Err(e) => error!("AW9523 initialization failed (continuing anyway): {:?}", e),
    }

    // --------------------------------------------------------------------------------
    // DISPLAY INITIALIZATION
    // --------------------------------------------------------------------------------

    // --- Initialize SPI for CoreS3 display ---
    // CoreS3: SCK=GPIO36, MOSI=GPIO37, CS=GPIO3
    let spi = Spi::<Blocking>::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO36)
    .with_mosi(peripherals.GPIO37)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf);
    let cs_output = Output::new(peripherals.GPIO3, Level::High, OutputConfig::default());
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, cs_output, spi_delay).unwrap();

    // LCD interface: DC = GPIO35
    let lcd_dc = Output::new(peripherals.GPIO35, Level::Low, OutputConfig::default());
    let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
    let di = SpiInterface::new(spi_device, lcd_dc, buffer);

    let mut display_delay = Delay::new();
    display_delay.delay_ns(500_000u32);

    // Reset pin: GPIO15 (open-drain for CoreS3)
    let reset = Output::new(peripherals.GPIO15, Level::High, OutputConfig::default());

    // Initialize ILI9342C display for CoreS3
    // 320x240, BGR color order, inverted colors
    let mut display: MyDisplay = Builder::new(ILI9342CRgb565, di)
        .reset_pin(reset)
        .display_size(320, 240)
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut display_delay)
        .unwrap();
    display.clear(Rgb565::BLUE).unwrap();

    info!("Display initialized");

    unsafe { Instant::set_elapsed(elapsed_time) };

    // --------------------------------------------------------------------------------
    // IMU INITIALIZATION - TEMPORARILY DISABLED
    // --------------------------------------------------------------------------------
    // TODO: Implement MPU6886 driver for esp-hal 1.0.0
    // The CoreS3 uses MPU6886 IMU, not BMI270
    // Reference: /Users/georgik/projects/esp32-spooky-maze-game-old/m5stack-cores3
    //
    // For now, the game will use demo/random movement until IMU is implemented
    info!("IMU disabled - will use demo movement controller");

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

    app.insert_non_send_resource(DisplayResource { display })
        .insert_resource(FrameBufferResource::new())
        .insert_resource(HudState::default())
        .insert_resource(MazeSeed(Some(seed)))
        .add_systems(Startup, systems::setup::setup)
        .add_message::<PlayerInputMessage>()
        .add_message::<CoinCollisionMessage>()
        .add_message::<DynamiteCollisionMessage>()
        .add_message::<WalkerCollisionMessage>()
        .add_message::<NpcCollisionMessage>()
        .add_systems(
            Update,
            (
                // IMU input system temporarily disabled
                // player_input::dispatch_accelerometer_input::<...>,
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
