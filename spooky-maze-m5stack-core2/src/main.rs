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
use bevy::prelude::*;
use bevy::time::TimePlugin;
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
    psram::Psram,
    rng::Rng,
    rtc_cntl::Rtc,
    spi::master::{Spi, SpiDmaBus},
    time::Rate,
};
use esp_println::{logger::init_logger_from_env, println};
use log::{error, info};

// IMU - Core2 uses MPU6886
use mipidsi::{Builder, models::ILI9342CRgb565};
use mipidsi::{
    interface::SpiInterface,
    options::{ColorInversion, ColorOrder},
};
use mpu6886::Mpu6886;
use spooky_core::resources::MazeSeed;

// Embedded Graphics imports for our framebuffer drawing.
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;

// M5Stack Core2 Power Management
use axp192::Axp192;

// ESP-IDF App Descriptor required by newer espflash
esp_bootloader_esp_idf::esp_app_desc!();

// Bring in our custom render system from our embedded module.
mod embedded_systems {
    pub mod player_input;
    pub mod render;
}
use embedded_systems::player_input;
use embedded_systems::player_input::AccelerometerResource;
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
// Core2 has 320x240 display
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

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// Static reference to AXP192 for backlight control (used in power button handler)
static mut AXP192_REF: Option<
    axp192::Axp192<embedded_hal_bus::i2c::RefCellDevice<'static, I2c<'static, Blocking>>>,
> = None;
// Display on/off state
static DISPLAY_ON: AtomicBool = AtomicBool::new(true);

// Light sleep function - preserves game state and RAM
fn enter_light_sleep() {
    info!("=== ENTERING LIGHT SLEEP ===");
    info!("Touch or press power button to wake up");

    unsafe {
        if let Some(ref mut axp) = AXP192_REF {
            // Turn off display before sleeping
            let _ = axp.set_dcdc3_on(false);
            // Turn off power LED to save ~1mA
            let _ = axp.set_gpio1_output(true); // LED off (inverted logic)
        }
    }

    // Get RTC control
    let mut rtc = Rtc::new(unsafe { esp_hal::peripherals::LPWR::steal() });

    // For light sleep, we can use GPIO wakeup (any GPIO works!)
    // Use GpioWakeupSource which allows any GPIO with wakeup_enable
    use esp_hal::rtc_cntl::sleep::GpioWakeupSource;

    // Configure GPIO wakeup (any GPIO can be used in light sleep)
    // The touch controller or power button will wake us up
    let gpio_wakeup = GpioWakeupSource::new();

    // Timer wakeup as backup - wake every 60 seconds to check status
    use esp_hal::rtc_cntl::sleep::TimerWakeupSource;
    let timer_wakeup = TimerWakeupSource::new(core::time::Duration::from_secs(60));

    info!("Entering light sleep - touch or power button will wake the device");
    info!("Game state and RAM are preserved!");

    // Light sleep - preserves RAM and execution state
    rtc.sleep_light(&[&gpio_wakeup, &timer_wakeup]);

    // Code resumes here after wakeup
    info!("=== WOKE FROM LIGHT SLEEP ===");

    unsafe {
        if let Some(ref mut axp) = AXP192_REF {
            // Turn power LED back on
            let _ = axp.set_gpio1_output(false); // LED on (inverted logic)
            // Turn display back on
            let _ = axp.set_dcdc3_on(true);
        }
    }

    info!("Resuming game...");
}
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

    // Try to use PSRAM allocator (ESP32 with psram feature)
    // M5Stack Core2 may have external PSRAM via SPI
    let psram = Psram::new(peripherals.PSRAM, Default::default());
    esp_alloc::psram_allocator!(&psram);
    esp_alloc::heap_allocator!(size: 72 * 1024);

    info!("PSRAM allocator initialized");

    // Backlight fade constants - defined at main function scope for accessibility
    const BACKLIGHT_VOLTAGE_MAX: u16 = 2800; // Maximum brightness (mV)
    const BACKLIGHT_VOLTAGE_MIN: u16 = 700; // Minimum brightness (mV)
    const BACKLIGHT_FADE_STEPS: u16 = 20; // Number of fade steps

    // --- DMA Buffers for SPI ---
    // Core2 uses same buffer size for 320x240 display
    #[allow(clippy::manual_div_ceil)]
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(8912);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    // --------------------------------------------------------------------------------
    // POWER MANAGEMENT - CRITICAL for Core2
    // Must be initialized BEFORE display
    // --------------------------------------------------------------------------------

    // --- Initialize I2C bus for power management (GPIO21=SDA, GPIO22=SCL for Core2) ---
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22);

    // Wrap I2C in RefCell for sharing between AXP192 and MPU6886
    let i2c_bus: &'static RefCell<I2c<'static, Blocking>> = Box::leak(Box::new(RefCell::new(i2c)));

    // --- Initialize AXP192 Power Management IC ---
    info!("Initializing AXP192 Power Management IC");
    let axp_i2c = embedded_hal_bus::i2c::RefCellDevice::new(i2c_bus);
    let mut axp = Axp192::new(axp_i2c);

    // Configure AXP192 for M5Stack Core2
    // Based on original m5sc2_init() function
    let mut delay = Delay::new();
    axp.set_dcdc1_voltage(3350).unwrap(); // MCU voltage
    axp.set_ldo2_voltage(3300).unwrap(); // Peripherals (LCD)
    axp.set_ldo2_on(true).unwrap();
    axp.set_ldo3_voltage(2000).unwrap(); // Vibration motor
    axp.set_ldo3_on(false).unwrap();
    axp.set_dcdc3_voltage(2800).unwrap(); // LCD backlight
    axp.set_dcdc3_on(true).unwrap();

    // Configure GPIO modes
    axp.set_gpio1_mode(axp192::GpioMode12::NmosOpenDrainOutput)
        .unwrap(); // LED
    axp.set_gpio1_output(true).unwrap(); // LED off to save ~1mA (inverted logic)
    axp.set_gpio2_mode(axp192::GpioMode12::NmosOpenDrainOutput)
        .unwrap(); // Speaker
    axp.set_gpio2_output(true).unwrap();
    axp.set_gpio4_mode(axp192::GpioMode34::NmosOpenDrainOutput)
        .unwrap(); // LCD reset

    // LCD reset sequence via AXP192 GPIO4
    axp.set_gpio4_output(false).unwrap(); // Assert reset
    axp.set_ldo3_on(true).unwrap(); // Buzz vibration motor
    delay.delay_ms(100u32);
    axp.set_gpio4_output(true).unwrap(); // Release reset
    axp.set_ldo3_on(false).unwrap(); // Stop vibration motor
    delay.delay_ms(100u32);

    // Enable AXP192 PEK (power button) interrupts so we can detect button presses
    info!("Enabling AXP192 power button interrupts");
    axp.enable_power_key_interrupts().unwrap();

    // Initialize: ensure display starts ON at full brightness
    info!("Initializing display backlight to ON state at full brightness");
    axp.set_dcdc3_voltage(BACKLIGHT_VOLTAGE_MAX).unwrap();
    axp.set_dcdc3_on(true).unwrap();
    DISPLAY_ON.store(true, Ordering::Relaxed);

    // Store AXP192 reference for backlight control via power button
    // Safety: This is safe because we're in main() and the reference will live for the program duration
    unsafe {
        AXP192_REF = Some(axp);
    }

    info!("AXP192 initialized successfully");

    // --------------------------------------------------------------------------------
    // DISPLAY INITIALIZATION
    // --------------------------------------------------------------------------------

    // --- Initialize SPI for Core2 display ---
    // Core2: SCK=GPIO18, MOSI=GPIO23, MISO=GPIO38, CS=GPIO5
    // ESP32 uses DMA_SPI2 instead of DMA_CH0 (PDMA vs GDMA)
    let spi = Spi::<Blocking>::new(
        peripherals.SPI2,
        esp_hal::spi::master::Config::default()
            .with_frequency(Rate::from_mhz(60))
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23)
    .with_miso(peripherals.GPIO38)
    .with_dma(peripherals.DMA_SPI2)
    .with_buffers(dma_rx_buf, dma_tx_buf);
    let cs_output = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, cs_output, spi_delay).unwrap();

    // LCD interface: DC = GPIO15 for Core2
    let lcd_dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
    let di = SpiInterface::new(spi_device, lcd_dc, buffer);

    let mut display_delay = Delay::new();
    display_delay.delay_ns(500_000u32);

    // Core2 doesn't have a separate reset pin - reset is handled by AXP192 GPIO4
    // Create a dummy reset pin for the display builder (it won't be used)
    let lcd_reset = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());

    // Initialize ILI9342C display for Core2
    // 320x240, BGR color order, inverted colors
    let mut display: MyDisplay = Builder::new(ILI9342CRgb565, di)
        .reset_pin(lcd_reset)
        .display_size(320, 240)
        .color_order(ColorOrder::Bgr)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut display_delay)
        .unwrap();
    display.clear(Rgb565::BLUE).unwrap();

    info!("Display initialized");

    unsafe { Instant::set_elapsed(elapsed_time) };

    // --------------------------------------------------------------------------------
    // IMU INITIALIZATION
    // --------------------------------------------------------------------------------
    // Core2 uses MPU6886 accelerometer
    info!("Initializing MPU6886 IMU");

    // Add delay after power-on to let MPU6886 stabilize
    let mut imu_delay = Delay::new();
    imu_delay.delay_ms(100u32);

    let imu_i2c = embedded_hal_bus::i2c::RefCellDevice::new(i2c_bus);
    let mut imu = Mpu6886::new(imu_i2c);
    match imu.init(&mut imu_delay) {
        Ok(_) => info!("MPU6886 initialized successfully"),
        Err(e) => {
            error!("MPU6886 initialization failed: {:?}", e);
            panic!("Failed to initialize MPU6886 IMU");
        }
    }

    info!("MPU6886 IMU ready");

    // --------------------------------------------------------------------------------
    // POWER SAVE FEATURE
    // --------------------------------------------------------------------------------
    // M5Stack Core2's power button (PWR_KEY) is connected to the AXP192 PMIC.
    // We poll the AXP192 via I2C to detect button presses.
    // Short press the side power button to toggle display backlight with fade effect.
    // Backlight fade constants (defined above during AXP192 init)
    info!(
        "Power save feature enabled - press side PWR button to toggle display (with fade effect)"
    );

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
                // MPU6886 IMU input
                player_input::dispatch_accelerometer_input::<
                    embedded_hal_bus::i2c::RefCellDevice<'static, I2c<'static, Blocking>>,
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
    let mut fade_delay = Delay::new();
    let mut debug_counter: u32 = 0;
    loop {
        // Check for power button press event via AXP192 PMIC register 0x46
        // Returns: 0=no press, 1=long press, 2=short press, 3=both
        let button_press_type = unsafe {
            if let Some(ref mut axp) = AXP192_REF {
                axp.get_power_key_press_event().unwrap_or(0)
            } else {
                0
            }
        };

        // Debug logging every 100 iterations (~30 seconds)
        debug_counter += 1;
        if debug_counter >= 100 {
            debug_counter = 0;
            info!("Reg 0x46 value: 0x{:02x}", button_press_type);
        }

        // Check for long press (value 1) or both (value 3) - enter light sleep
        // Check for short press (value 2) - toggle display backlight
        if button_press_type == 1 || button_press_type == 3 {
            info!("Long press detected - entering light sleep!");
            enter_light_sleep();
        } else if button_press_type == 2 {
            info!("Short press detected - toggling display");
            // Short press toggles display backlight
            let current_state = DISPLAY_ON.load(Ordering::Relaxed);
            let new_state = !current_state;

            // Toggle backlight via AXP192 DCDC3 with fade effect
            unsafe {
                if let Some(ref mut axp) = AXP192_REF {
                    if new_state {
                        // FADE IN: gradually increase brightness
                        info!("Display backlight fading IN...");
                        for step in 0..=BACKLIGHT_FADE_STEPS {
                            let voltage = BACKLIGHT_VOLTAGE_MIN
                                + (BACKLIGHT_VOLTAGE_MAX - BACKLIGHT_VOLTAGE_MIN) * step
                                    / BACKLIGHT_FADE_STEPS;
                            let _ = axp.set_dcdc3_voltage(voltage);
                            // Small delay between steps for smooth fade
                            fade_delay.delay_ms(50u32);
                        }
                        let _ = axp.set_dcdc3_on(true);
                        info!("Display backlight ON (button press)");
                    } else {
                        // FADE OUT: gradually decrease brightness
                        info!("Display backlight fading OUT...");
                        for step in 0..=BACKLIGHT_FADE_STEPS {
                            let voltage = BACKLIGHT_VOLTAGE_MAX
                                - (BACKLIGHT_VOLTAGE_MAX - BACKLIGHT_VOLTAGE_MIN) * step
                                    / BACKLIGHT_FADE_STEPS;
                            let _ = axp.set_dcdc3_voltage(voltage);
                            // Small delay between steps for smooth fade
                            fade_delay.delay_ms(50u32);
                        }
                        let _ = axp.set_dcdc3_on(false);
                        info!("Display backlight OFF (button press)");
                    }
                }
            }
            DISPLAY_ON.store(new_state, Ordering::Relaxed);

            // Wait a bit to avoid multiple toggles from the same press
            fade_delay.delay_ms(500u32);
        }

        // Only run game updates if display is on
        if DISPLAY_ON.load(Ordering::Relaxed) {
            app.update();
        }

        loop_delay.delay_ms(300u32);
    }
}
