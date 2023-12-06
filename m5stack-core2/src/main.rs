#![no_std]
#![no_main]

// https://shop.m5stack.com/products/m5stack-core2-esp32-iot-development-kit

// use accel_device::Mpu6886Wrapper;

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

use spooky_embedded::{
    app::app_loop,
    controllers::{
        accel::{
            AccelMovementController,
            Mpu6886Wrapper
        },
        composites::accel_composite::AccelCompositeController
    },
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
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
use shared_bus::BusManagerSimple;

use embedded_hal::{
    digital::v2::OutputPin,
    blocking::i2c::{ Read, Write, WriteRead }
};

use axp192::{ Axp192 };

pub struct Universe<D> {
    pub engine: Engine<D>,
}

// Source: https://github.com/rcloran/axp192-rs/blob/main/examples/m5stack-core2.rs
fn m5sc2_init<I2C, E>(axp: &mut axp192::Axp192<I2C>, delay: &mut Delay) -> Result<(), E>
where
    I2C: Read<Error = E>
        + Write<Error = E>
        + WriteRead<Error = E>,
{
    // Default setup for M5Stack Core 2
    axp.set_dcdc1_voltage(3350)?; // Voltage to provide to the microcontroller (this one!)

    axp.set_ldo2_voltage(3300)?; // Peripherals (LCD, ...)
    axp.set_ldo2_on(true)?;

    axp.set_ldo3_voltage(2000)?; // Vibration motor
    axp.set_ldo3_on(false)?;

    axp.set_dcdc3_voltage(2800)?; // LCD backlight
    axp.set_dcdc3_on(true)?;

    axp.set_gpio1_mode(axp192::GpioMode12::NmosOpenDrainOutput)?; // Power LED
    axp.set_gpio1_output(false)?; // In open drain modes, state is opposite to what you might
                                  // expect

    axp.set_gpio2_mode(axp192::GpioMode12::NmosOpenDrainOutput)?; // Speaker
    axp.set_gpio2_output(true)?;

    axp.set_key_mode(
        // Configure how the power button press will work
        axp192::ShutdownDuration::Sd4s,
        axp192::PowerOkDelay::Delay64ms,
        true,
        axp192::LongPress::Lp1000ms,
        axp192::BootTime::Boot512ms,
    )?;

    axp.set_gpio4_mode(axp192::GpioMode34::NmosOpenDrainOutput)?; // LCD reset control

    axp.set_battery_voltage_adc_enable(true)?;
    axp.set_battery_current_adc_enable(true)?;
    axp.set_acin_current_adc_enable(true)?;
    axp.set_acin_voltage_adc_enable(true)?;

    // Actually reset the LCD
    axp.set_gpio4_output(false)?;
    axp.set_ldo3_on(true)?; // Buzz the vibration motor while intializing ¯\_(ツ)_/¯
    delay.delay_ms(100u32);
    axp.set_gpio4_output(true)?;
    axp.set_ldo3_on(false)?;
    delay.delay_ms(100u32);

    Ok(())
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

    let bus = BusManagerSimple::new(i2c_bus);

    let mut axp = axp192::Axp192::new(bus.acquire_i2c());

    // M5Stack CORE 2 - https://docs.m5stack.com/en/core/core2
    m5sc2_init(&mut axp, &mut delay).unwrap();

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

    let di = spi_dma_displayinterface::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(LCD_H_RES, LCD_V_RES)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(lcd_reset))
        .unwrap();

    Text::new(
        "Initializing...",
        Point::new(80, 110),
        MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
    )
    .draw(&mut display)
    .unwrap();


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

    let accel_movement_controller = AccelMovementController::new(icm, 0.3);
    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = AccelCompositeController::new(demo_movement_controller, accel_movement_controller);

    app_loop(&mut display, seed_buffer, movement_controller);
    loop {}

}
