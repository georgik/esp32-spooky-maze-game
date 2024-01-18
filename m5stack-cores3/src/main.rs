#![no_std]
#![no_main]

// https://shop.m5stack.com/products/m5stack-cores3-esp32s3-lotdevelopment-kit

use spi_dma_displayinterface::spi_dma_displayinterface;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use hal::{
    clock::{ClockControl, CpuClock},
    dma::DmaPriority,
    gdma::Gdma,
    i2c,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay, Rng, IO, gpio::PushPull,
};

use esp_backtrace as _;
use log::info;

#[cfg(feature = "mpu9250")]
use mpu9250::{ImuMeasurements, Mpu9250};

#[cfg(feature = "mpu6050")]
use mpu6050::Mpu6050;

#[cfg(feature = "mpu6886")]
use mpu6886::Mpu6886;

use spooky_core::engine::Engine;

use shared_bus::BusManagerSimple;

use embedded_hal::digital::v2::OutputPin;

use spooky_embedded::{
    app::app_loop,
    controllers::{
        accel::AccelMovementController,
        composites::accel_composite::AccelCompositeController
    },
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
};

use axp2101::{ I2CPowerManagementInterface, Axp2101 };
use aw9523::I2CGpioExpanderInterface;

use esp_bsp::lcd_gpios;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    esp_println::logger::init_logger_from_env();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let (lcd_sclk, lcd_mosi, lcd_cs, lcd_miso, lcd_dc, _lcd_backlight, lcd_reset) = lcd_gpios!(BoardType::M5StackCoreS3, io);

    // I2C
    let sda = io.pins.gpio12;
    let scl = io.pins.gpio11;

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];

    let i2c_bus = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        400u32.kHz(),
        &clocks,
    );

    let bus = BusManagerSimple::new(i2c_bus);

    info!("Initializing AXP2101");
    let axp_interface = I2CPowerManagementInterface::new(bus.acquire_i2c());
    let mut axp = Axp2101::new(axp_interface);
    axp.init().unwrap();

    info!("Initializing GPIO Expander");
    let aw_interface = I2CGpioExpanderInterface::new(bus.acquire_i2c());
    let mut aw = aw9523::Aw9523::new(aw_interface);
    aw.init().unwrap();

    // M5Stack CORE 2 - https://docs.m5stack.com/en/core/core2
    // let mut backlight = io.pins.gpio3.into_push_pull_output();
    delay.delay_ms(500u32);
    info!("About to initialize the SPI LED driver");

    let spi = Spi::new(
        peripherals.SPI3,
        20u32.MHz(),
        SpiMode::Mode0,
        &clocks,
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

    delay.delay_ms(500u32);
    // backlight.set_high().unwrap();

    //https://github.com/m5stack/M5CoreS3/blob/main/src/utility/Config.h#L8
    let di = spi_dma_displayinterface::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(lcd_reset))
        .unwrap();
    delay.delay_ms(500u32);
    info!("Initializing...");
    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();


    // #[cfg(any(feature = "mpu9250"))]
    // let mut icm = Mpu9250::imu_default(bus.acquire_i2c(), &mut delay).unwrap();

    // #[cfg(any(feature = "mpu6050"))]
    // let mut icm = Mpu6050::new(bus.acquire_i2c());

    // let icm_inner = Mpu6886::new(bus.acquire_i2c());
    // let icm = Mpu6886Wrapper::new(icm_inner);
    // let is_imu_enabled = match icm.init(&mut delay) {
    //     Ok(_) => true,
    //     Err(_) => false,
    // };


    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = demo_movement_controller;

    info!("Entering main loop");
    app_loop(&mut display, seed_buffer, movement_controller);
    loop {}

}
