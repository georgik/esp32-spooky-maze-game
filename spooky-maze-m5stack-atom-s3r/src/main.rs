#![no_std]
#![no_main]

extern crate alloc;
use alloc::boxed::Box;
use core::cell::RefCell;
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
use log::info;
use mipidsi::{Builder, models::GC9107};
use mipidsi::{interface::SpiInterface, options::ColorOrder};
use spooky_core::resources::MazeSeed;

// Embedded Graphics imports for our framebuffer drawing.
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;

// Inertial measurement unit (IMU): BMI270.
use bmi2::{Bmi2, I2cAddr};

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

// Type alias for our concrete I2C bus (to be wrapped in RefCell for sharing).
type I2cMasterBus = I2c<'static, Blocking>;

// --- LCD backlight control via LP5562 ---

// LP5562 Register addresses (outputs R/G/B are not used and thus ignored).
const LP5562_I2C_REG_ENABLE: u8 = 0x00;
const LP5562_I2C_REG_OP_MODE: u8 = 0x01;
const LP5562_I2C_REG_W_PWM: u8 = 0x0E;
const LP5562_I2C_REG_W_CURRENT: u8 = 0x0F;
const LP5562_I2C_REG_CONFIG: u8 = 0x08;
const LP5562_I2C_REG_LED_MAP: u8 = 0x70;

// LP5562 constants:
const LP5562_I2C_ADDR: u8 = 0x30;
const LP5562_I2C_MASTER_ENABLE: u8 = 0x40;

/// LP5562 LED driver for backlight control.
///
/// This abstraction handles the LP5562 chip initialization and brightness control.
/// Timing constants were taken from the Linux driver implementation.
pub struct LP5562Backlight<I2C> {
    i2c: I2C,
}

impl<I2C: embedded_hal::i2c::I2c> LP5562Backlight<I2C> {
    /// Initialize the LP5562 backlight controller.
    pub fn new(mut i2c: I2C, delay: &mut impl DelayNs) -> Result<Self, I2C::Error> {
        // Step 1: Enable internal clock.
        i2c.write(LP5562_I2C_ADDR, &[LP5562_I2C_REG_CONFIG, 0x01])?;
        delay.delay_ms(1);

        // Step 2: Enable chip.
        i2c.write(
            LP5562_I2C_ADDR,
            &[LP5562_I2C_REG_ENABLE, LP5562_I2C_MASTER_ENABLE],
        )?;
        delay.delay_us(500);

        // Step 3: Configure LED map - all LEDs are controlled from I2C registers.
        i2c.write(LP5562_I2C_ADDR, &[LP5562_I2C_REG_LED_MAP, 0x00])?;
        delay.delay_us(200);

        // Step 4: Set operation mode to direct PWM control.
        i2c.write(LP5562_I2C_ADDR, &[LP5562_I2C_REG_OP_MODE, 0x00])?;
        delay.delay_us(200);

        Ok(Self { i2c })
    }

    /// Set the backlight brightness (0-255 where a high value produces a brighter backlight).
    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), I2C::Error> {
        // Set PWM value for white LED (W channel).
        self.i2c
            .write(LP5562_I2C_ADDR, &[LP5562_I2C_REG_W_PWM, brightness])?;

        // Set current for white LED (W channel) - maximum current.
        self.i2c
            .write(LP5562_I2C_ADDR, &[LP5562_I2C_REG_W_CURRENT, 0xFF])?;

        Ok(())
    }
}

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
    GC9107,
    Output<'static>,
>;

struct DisplayResource {
    display: MyDisplay,
}

use crate::embedded_systems::player_input;
use crate::embedded_systems::player_input::AccelerometerResource;
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

    // Initialize heap allocator for internal RAM
    esp_alloc::heap_allocator!(size: 200 * 1024);

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
    .with_sck(peripherals.GPIO15)
    .with_mosi(peripherals.GPIO21)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf);
    let cs_output = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, cs_output, spi_delay).unwrap();

    // LCD interface: use GPIO4 for DC.
    let lcd_dc = Output::new(peripherals.GPIO42, Level::Low, OutputConfig::default());
    let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
    let di = SpiInterface::new(spi_device, lcd_dc, buffer);

    let mut display_delay = Delay::new();
    display_delay.delay_ns(500_000u32);

    // Reset pin for display
    let reset = Output::new(peripherals.GPIO48, Level::High, OutputConfig::default());

    let mut display: MyDisplay = Builder::new(GC9107, di)
        .reset_pin(reset)
        .display_size(128, 128)
        .display_offset(0, 32)
        .color_order(ColorOrder::Bgr)
        .init(&mut display_delay)
        .unwrap();
    display.clear(Rgb565::BLUE).unwrap();

    info!("Display initialized");

    unsafe { Instant::set_elapsed(elapsed_time) };

    // --- Initialize shared I2C bus for both backlight and IMU ---
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO45)
    .with_scl(peripherals.GPIO0);

    // Wrap the I2C device in a RefCell for sharing between backlight and IMU
    let i2c_bus: &'static RefCell<I2cMasterBus> = Box::leak(Box::new(RefCell::new(i2c)));

    // Initialize backlight controller
    let backlight_i2c = embedded_hal_bus::i2c::RefCellDevice::new(i2c_bus);
    let mut backlight_delay = Delay::new();
    let mut backlight = LP5562Backlight::new(backlight_i2c, &mut backlight_delay)
        .expect("failed to initialize backlight");
    backlight
        .set_brightness(0xFF)
        .expect("failed to set backlight brightness");

    info!("Backlight initialized");

    // Initialize IMU with the shared I2C bus
    let i2c_device = embedded_hal_bus::i2c::RefCellDevice::new(i2c_bus);
    let mut imu = Bmi2::new_i2c(i2c_device, I2cAddr::Default, bmi2::types::Burst::Other(255));
    imu.init(&bmi2::config::BMI270_CONFIG_FILE)
        .expect("failed to initialize IMU");
    imu.set_pwr_ctrl(bmi2::types::PwrCtrl {
        aux_en: false,
        gyr_en: true,
        acc_en: true,
        temp_en: false,
    })
    .expect("failed to set IMU power control");

    info!("IMU initialized");

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
        .insert_non_send_resource(AccelerometerResource { sensor: imu })
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
                player_input::dispatch_accelerometer_input::<
                    embedded_hal_bus::i2c::RefCellDevice<'static, I2cMasterBus>,
                >,
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
