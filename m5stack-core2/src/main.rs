#![no_std]
#![no_main]
`
// https://shop.m5stack.com/products/m5stack-core2-esp32-iot-development-kit

use accel_device::Mpu6886Wrapper;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use hal::{
    clock::{ClockControl, CpuClock},
    i2c,
    peripherals::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Delay, Rng, Rtc, IO,
};

use esp_backtrace as _;

#[cfg(feature = "mpu9250")]
use mpu9250::{ImuMeasurements, Mpu9250};

#[cfg(feature = "mpu6050")]
use mpu6050::Mpu6050;

#[cfg(feature = "mpu6886")]
use mpu6886::Mpu6886;

use embedded_graphics::pixelcolor::Rgb565;

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

    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();

    wdt0.disable();
    wdt1.disable();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // I2C
    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;

    let i2c_bus = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        400u32.kHz(),
        &mut system.peripheral_clock_control,
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
    let mut backlight = io.pins.gpio3.into_push_pull_output();

    #[cfg(feature = "esp32")]
    let spi = spi::Spi::new(
        peripherals.SPI3,
        io.pins.gpio18,   // SCLK
        io.pins.gpio23,   // MOSI
        io.pins.gpio38,   // MISO
        io.pins.gpio5,   // CS
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    backlight.set_high().unwrap();

    let reset = io.pins.gpio4.into_push_pull_output();
    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio15.into_push_pull_output());

    #[cfg(feature = "m5stack_core2")]
    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, Some(reset))
        .unwrap();

    #[cfg(feature = "wokwi")]
    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::Landscape(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
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

    let icm_inner = Mpu6886::new(bus.acquire_i2c());
    let mut icm = Mpu6886Wrapper::new(icm_inner);
    // let is_imu_enabled = match icm.init(&mut delay) {
    //     Ok(_) => true,
    //     Err(_) => false,
    // };


    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();
    let mut data = [Rgb565::BLACK; 320 * 240];

    app_loop( &mut display, seed_buffer, icm);
    loop {}

}
