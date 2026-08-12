#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::{
    alloc::Layout,
    sync::atomic::{AtomicU32, Ordering},
};

use bevy::{
    DefaultPlugins,
    app::{App, Startup},
    prelude::Update,
};
use bevy_ecs::prelude::*;
use bevy_platform::time::Instant;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_graphics_framebuf::FrameBuf;

use esp_backtrace as _;
use esp_hal::{
    clock::{
        CpuClock,
        ll::{
            MipiDsiPhyPllRefclkConfig,
            MipiDsiPhyPllRefclkSclk,
        },
    },
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    i2c::master::{
        Config as I2cConfig,
        I2c,
    },
    main,
    mipi_dsi::{
        Config as MipiConfig,
        DataLanes,
        MipiDsi,
        dpi::{
            ColorFormat,
            DpiClockSource,
            DpiConfig,
            FrameTiming,
        },
    },
    psram,
    rng::Rng,
    time::Rate,
};
use esp_println::{
    logger::init_logger_from_env,
    println,
};

use spooky_core::{
    events::{
        coin::CoinCollisionMessage,
        dynamite::DynamiteCollisionMessage,
        npc::NpcCollisionMessage,
        player::PlayerInputMessage,
        walker::WalkerCollisionMessage,
    },
    resources::MazeSeed,
    systems::{
        self,
        collisions,
        hud::HudState,
        process_player_input::process_player_input,
    },
};

mod embedded_systems {
    pub mod render;
}

mod heapbuffer;

mod touch;

use crate::{
    embedded_systems::render::render_system,
    heapbuffer::HeapBuffer,
};

use crate::touch::{
    Gt911,
    TouchEvent,
    TouchInputState,
    direction_at,
    dispatch_touch_input,
    transform_touch_point,
};



// Required by espflash and the ESP-IDF bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

// -----------------------------------------------------------------------------
// Display configuration
// -----------------------------------------------------------------------------

const LCD_H_RES: usize = 1024;
const LCD_V_RES: usize = 600;

const LCD_BUFFER_SIZE: usize = LCD_H_RES * LCD_V_RES;

const BYTES_PER_PIXEL: usize = 2;
const MIPI_FB_SIZE: usize =
    LCD_BUFFER_SIZE * BYTES_PER_PIXEL;

// Approximate interval between game updates.
const FRAME_TIME_MS: u32 = 50;

// -----------------------------------------------------------------------------
// EK79007 panel initialization
// -----------------------------------------------------------------------------

static EK79007_INIT: &[(u8, &[u8])] = &[
    (0xB2, &[0x10]), // Two MIPI data lanes
    (0x80, &[0x8B]),
    (0x81, &[0x78]),
    (0x82, &[0x84]),
    (0x83, &[0x88]),
    (0x84, &[0xA8]),
    (0x85, &[0xE3]),
    (0x86, &[0x88]),
];


// -----------------------------------------------------------------------------
// Software framebuffer used by embedded-graphics
// -----------------------------------------------------------------------------

type FbBuffer = HeapBuffer<Rgb565, LCD_BUFFER_SIZE>;
type MyFrameBuf = FrameBuf<Rgb565, FbBuffer>;

#[derive(Resource)]
pub(crate) struct FrameBufferResource {
    pub(crate) frame_buf: MyFrameBuf,
}

impl FrameBufferResource {
    fn new() -> Self {
        // This allocation is placed in PSRAM because the PSRAM allocator is
        // initialized before this function is called.
        let fb_data: Box<[Rgb565; LCD_BUFFER_SIZE]> =
            Box::new([Rgb565::BLACK; LCD_BUFFER_SIZE]);

        let heap_buffer = HeapBuffer::new(fb_data);

        let frame_buf = MyFrameBuf::new(
            heap_buffer,
            LCD_H_RES,
            LCD_V_RES,
        );

        Self { frame_buf }
    }
}

// -----------------------------------------------------------------------------
// Bevy time source
// -----------------------------------------------------------------------------

static ELAPSED: AtomicU32 = AtomicU32::new(0);

fn elapsed_time() -> core::time::Duration {
    let milliseconds = ELAPSED.load(Ordering::Relaxed);

    core::time::Duration::from_millis(
        milliseconds as u64,
    )
}

// -----------------------------------------------------------------------------
// MIPI framebuffer allocation
// -----------------------------------------------------------------------------

fn allocate_mipi_framebuffer() -> &'static mut [u8] {
    let layout = Layout::from_size_align(
        MIPI_FB_SIZE,
        64,
    )
    .expect("Invalid MIPI framebuffer layout");

    let pointer = unsafe {
        alloc::alloc::alloc_zeroed(layout)
    };

    assert!(
        !pointer.is_null(),
        "MIPI framebuffer allocation failed"
    );

    unsafe {
        core::slice::from_raw_parts_mut(
            pointer,
            MIPI_FB_SIZE,
        )
    }
}

// -----------------------------------------------------------------------------
// Application entry point
// -----------------------------------------------------------------------------

#[main]
fn main() -> ! {
    // -------------------------------------------------------------------------
    // ESP-HAL and PSRAM initialization
    // -------------------------------------------------------------------------

    let config = esp_hal::Config::default()
        .with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    init_logger_from_env();

    esp_alloc::psram_allocator!(
        peripherals.PSRAM,
        esp_hal::psram,
        psram::PsramConfig::default()
    );

    println!("PSRAM initialized");

    let delay = Delay::new();

    // -------------------------------------------------------------------------
    // EK79007 reset and backlight
    // -------------------------------------------------------------------------

    let mut lcd_reset = Output::new(
        peripherals.GPIO27,
        Level::Low,
        OutputConfig::default(),
    );

    delay.delay_millis(10);

    lcd_reset.set_high();

    delay.delay_millis(120);

    // Keep this variable alive. Dropping an Output could release the GPIO.
    let _lcd_backlight = Output::new(
        peripherals.GPIO26,
        Level::High,
        OutputConfig::default(),
    );

    // -------------------------------------------------------------------------
    // GT911 touch-controller initialization
    // -------------------------------------------------------------------------

    let i2c_config = I2cConfig::default()
        .with_frequency(Rate::from_khz(400));

    let touch_i2c = I2c::new(
        peripherals.I2C0,
        i2c_config,
    )
    .expect("I2C initialization failed")
    .with_sda(peripherals.GPIO7)
    .with_scl(peripherals.GPIO8);

    let (mut touch_controller, touch_product_id) =
        Gt911::new(touch_i2c)
            .expect("GT911 touch controller not detected");

    println!(
        "GT911 detected at 0x{:02X}, product ID: {:?}",
        touch_controller.address(),
        touch_product_id,
    );

    // -------------------------------------------------------------------------
    // MIPI-DSI initialization
    // -------------------------------------------------------------------------

    let mut mipi_bus = MipiDsi::new(
        peripherals.MIPI_DSI,
        peripherals.VDMA_CH0,
        MipiConfig::default()
            .with_num_data_lanes(DataLanes::_2)
            .with_lane_bit_rate_mbps(1000.0)
            .with_phy_pll_refclk(
                MipiDsiPhyPllRefclkConfig::new(
                    MipiDsiPhyPllRefclkSclk::Xtal,
                    0,
                ),
            )
            .with_force_clock_lane_hs(false),
    )
    .expect("MIPI-DSI initialization failed");

    println!("MIPI-DSI bus initialized");

    // -------------------------------------------------------------------------
    // EK79007 command-mode initialization
    // -------------------------------------------------------------------------

    {
        let mut dbi = mipi_bus.dbi(0);

        for &(command, parameters) in EK79007_INIT {
            dbi.write_cmd(command, parameters)
                .expect("EK79007 command failed");
        }

        // Sleep out.
        dbi.write_cmd(0x11, &[])
            .expect("EK79007 sleep-out command failed");

        delay.delay_millis(120);

        // Display on.
        dbi.write_cmd(0x29, &[])
            .expect("EK79007 display-on command failed");

        delay.delay_millis(20);
    }

    println!("EK79007 initialized");

    // -------------------------------------------------------------------------
    // Allocate the two VDMA framebuffers
    // -------------------------------------------------------------------------

    let framebuffer_1 = allocate_mipi_framebuffer();
    let framebuffer_2 = allocate_mipi_framebuffer();

    let mipi_framebuffers: [&mut [u8]; 2] = [
        framebuffer_1,
        framebuffer_2,
    ];

    // -------------------------------------------------------------------------
    // Start DPI video output
    // -------------------------------------------------------------------------

    let dpi_config = DpiConfig {
        virtual_channel: 0,

        pixel_clock_mhz: 48.0,
        dpi_clk_src: DpiClockSource::PllF240m,

        in_color_format: ColorFormat::Rgb565,
        out_color_format: ColorFormat::Rgb565,

        timing: FrameTiming {
            h_active: LCD_H_RES as u32,
            hsw: 10,
            hbp: 120,
            hfp: 120,

            v_active: LCD_V_RES as u32,
            vsw: 1,
            vbp: 20,
            vfp: 20,
        },
    };

    let mut dpi = mipi_bus
        .dpi(dpi_config, &mipi_framebuffers)
        .expect("MIPI DPI initialization failed");

    println!("MIPI video streaming started");

    // -------------------------------------------------------------------------
    // Initialize Bevy's clock
    // -------------------------------------------------------------------------

    unsafe {
        Instant::set_elapsed(elapsed_time);
    }

    // -------------------------------------------------------------------------
    // Random seed for maze generation
    // -------------------------------------------------------------------------

    let hardware_rng = Rng::new();

    let mut seed = [0u8; 32];
    hardware_rng.read(&mut seed);

    // -------------------------------------------------------------------------
    // Construct the Bevy application
    // -------------------------------------------------------------------------

    let mut app = App::new();

    app.add_plugins((DefaultPlugins,))
        .insert_resource(FrameBufferResource::new())
        .insert_resource(TouchInputState::default())
        .insert_resource(HudState::default())
        .insert_resource(MazeSeed(Some(seed)))
        .add_systems(
            Startup,
            systems::setup::setup,
        )
        .add_message::<PlayerInputMessage>()
        .add_message::<CoinCollisionMessage>()
        .add_message::<DynamiteCollisionMessage>()
        .add_message::<WalkerCollisionMessage>()
        .add_message::<NpcCollisionMessage>()
        .add_systems(
            Update,
            (
                // Convert a queued touch direction into PlayerInputMessage.
                dispatch_touch_input,

                // Apply PlayerInputMessage to the player.
                process_player_input,

                // Check collisions after moving the player.
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

                // Render after movement and collision processing.
                render_system,
            )
                .chain(),
        );

    println!("Bevy application initialized");

    // -------------------------------------------------------------------------
    // Main game/display loop
    // -------------------------------------------------------------------------

    loop {
        // Advance the clock used by Bevy.
        ELAPSED.fetch_add(
            FRAME_TIME_MS,
            Ordering::Relaxed,
        );


                // Poll the GT911 before running the Bevy frame.
        match touch_controller.poll_event() {
        Ok(TouchEvent::Point(point)) => {
            let (screen_x, screen_y) =
                transform_touch_point(point);

            let direction =
                direction_at(screen_x, screen_y);

            println!(
                "Touch raw=({}, {}), screen=({}, {}), button={:?}",
                point.x,
                point.y,
                screen_x,
                screen_y,
                direction,
            );

            app.world_mut()
                .resource_mut::<TouchInputState>()
                .update_pressed(direction);
        }

        Ok(TouchEvent::Released) => {
            println!("Touch released");

            app.world_mut()
                .resource_mut::<TouchInputState>()
                .update_pressed(None);
        }

        Ok(TouchEvent::NoUpdate) => {
            // Do not change the current state here.
        }

        Err(error) => {
            println!(
                "GT911 polling error: {:?}",
                error,
            );
        }
    }

app.update();

    app.update();
        // Run one Bevy frame. The render system draws the maze into
        // FrameBufferResource.

        // Synchronize our buffer switch with the display.
        dpi.wait_for_vsync();

        // Get the MIPI framebuffer that is not currently being displayed.
        let mipi_back_buffer = dpi.framebuffer_mut();

        // Borrow the embedded-graphics framebuffer from Bevy.
        {
            let software_framebuffer = app
                .world()
                .resource::<FrameBufferResource>();

            // Convert each embedded-graphics Rgb565 pixel into the two-byte
            // little-endian layout expected by the MIPI framebuffer.
            for (destination, source) in mipi_back_buffer
                .chunks_exact_mut(BYTES_PER_PIXEL)
                .zip(
                    software_framebuffer
                        .frame_buf
                        .data
                        .iter(),
                )
            {
                let raw_color: u16 =
                    (*source).into_storage();

                destination.copy_from_slice(
                    &raw_color.to_le_bytes(),
                );
            }
        }

        // Flush the PSRAM cache and switch VDMA to the completed buffer.
        dpi.commit();

        delay.delay_millis(FRAME_TIME_MS);
    }
}