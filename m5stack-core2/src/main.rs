#![no_std]
#![no_main]

// https://shop.m5stack.com/products/m5stack-core2-esp32-iot-development-kit

use accel_device::Mpu6886Wrapper;

// use display_interface_spi::SPIInterfaceNoCS;
use spi_dma_displayinterface::spi_dma_displayinterface::SPIInterfaceNoCS;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use hal::{
    clock::{ClockControl, CpuClock},
    dma::DmaPriority,
    pdma::Dma,
    i2c::I2C,
    peripherals::Peripherals,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay, Rng, IO,
};

use esp_backtrace as _;
use log::debug;

#[cfg(feature = "mpu9250")]
use mpu9250::{ImuMeasurements, Mpu9250};

#[cfg(feature = "mpu6050")]
use mpu6050::Mpu6050;

#[cfg(feature = "mpu6886")]
use mpu6886::Mpu6886;

use spooky_core::engine::Engine;

#[cfg(any(feature = "i2c"))]
use shared_bus::BusManagerSimple;

use embedded_hal::digital::v2::OutputPin;

mod app;
use app::app_loop;
mod accel_device;
mod accel_movement_controller;

mod m5stack_composite_controller;

#[cfg(any(feature = "axp192"))]
use axp192::{ I2CPowerManagementInterface, Axp192 };

pub struct Universe<D> {
    pub engine: Engine<D>,
}


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 160MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // I2C
    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;

    let i2c_bus = I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        400u32.kHz(),
        &clocks,
    );

    #[cfg(any(feature = "i2c"))]
    let bus = BusManagerSimple::new(i2c_bus);

    // Power management - AXP192
    #[cfg(any(feature = "axp192"))]
    {
        let axp_interface = I2CPowerManagementInterface::new(bus.acquire_i2c());
        let mut axp = Axp192::new(axp_interface);
        axp.init().unwrap();
    }

    // M5Stack CORE 2 - https://docs.m5stack.com/en/core/core2
    let lcd_h_res = 240;
    let lcd_v_res = 320;

    let lcd_sclk = io.pins.gpio18;
    let lcd_mosi = io.pins.gpio23;
    let lcd_miso = io.pins.gpio38;
    let lcd_cs = io.pins.gpio5;
    let lcd_dc = io.pins.gpio15.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio3.into_push_pull_output();
    let lcd_reset = io.pins.gpio4.into_push_pull_output();

    let dma = Dma::new(system.dma);
    let dma_channel = dma.spi2channel;

    let mut descriptors = [0u32; 8 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];


    let spi = Spi::new(
        peripherals.SPI2,
        lcd_sclk,
        lcd_mosi,
        lcd_miso,
        lcd_cs,
        60u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    ).with_dma(dma_channel.configure(
        false,
        &mut descriptors,
        &mut rx_descriptors,
        DmaPriority::Priority0,
    ));

    lcd_backlight.set_high().unwrap();

    let di = SPIInterfaceNoCS::new(spi, lcd_dc);

    #[cfg(feature = "m5stack_core2")]
    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(lcd_reset))
        .unwrap();

    #[cfg(feature = "wokwi")]
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::Landscape(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(lcd_reset))
        .unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();

    #[cfg(feature = "wokwi")]
    let button_b = io.pins.gpio34.into_pull_up_input();
    #[cfg(feature = "wokwi")]
    let button_c = io.pins.gpio35.into_pull_up_input();

    #[cfg(any(feature = "mpu9250"))]
    let mut icm = Mpu9250::imu_default(bus.acquire_i2c(), &mut delay).unwrap();

    #[cfg(any(feature = "mpu6050"))]
    let mut icm = Mpu6050::new(bus.acquire_i2c());

    let mut icm_inner = Mpu6886::new(bus.acquire_i2c());
    match icm_inner.init(&mut delay) {
        Ok(_) => {
            debug!("MPU6886 initialized");
        }
        Err(_) => {
            debug!("Failed to initialize MPU6886");
        }
    }
    let icm = Mpu6886Wrapper::new(icm_inner);


    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    app_loop( &mut display, lcd_h_res, lcd_v_res, seed_buffer, icm);
    loop {}

}
