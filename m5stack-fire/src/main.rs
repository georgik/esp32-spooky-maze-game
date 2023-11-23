#![no_std]
#![no_main]

// https://docs.makerfactory.io/m5stack/core/fire/

use spi_dma_displayinterface::spi_dma_displayinterface;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{DrawTarget, Point, RgbColor},
    text::Text,
    Drawable,
};

use hal::{
    clock::{ClockControl, CpuClock},
    i2c::I2C,
    peripherals::Peripherals,
    dma::DmaPriority,
    pdma::Dma,
    prelude::*,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay, Rng, IO,
};

// use panic_halt as _;
use esp_backtrace as _;

use mpu9250::{ImuMeasurements, Mpu9250};


use shared_bus::BusManagerSimple;

use embedded_hal::digital::v2::OutputPin;

use shared_bus::I2cProxy;
use embedded_hal::blocking::i2c::{Write, WriteRead};

use spooky_embedded::{
    app::app_loop,
    controllers::{
        accel::{
            AccelMovementController,
            Mpu9250Wrapper
        },
        composites::accel_composite::AccelCompositeController
    },
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
};


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();

    // With DMA we have sufficient throughput, so we can clock down the CPU to 160MHz
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let lcd_sclk = io.pins.gpio18;
    let lcd_mosi = io.pins.gpio23;
    let lcd_miso = io.pins.gpio19;
    let lcd_cs = io.pins.gpio14;
    let lcd_dc = io.pins.gpio27.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio32.into_push_pull_output();
    let lcd_reset = io.pins.gpio33.into_push_pull_output();

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

    let mut display = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(LCD_H_RES, LCD_V_RES)
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

    let button_b = io.pins.gpio38.into_pull_up_input();
    let button_c = io.pins.gpio37.into_pull_up_input();

    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;

    let i2c = I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &clocks,
    );

    let bus = BusManagerSimple::new(i2c);

    let icm_inner = Mpu9250::imu_default(bus.acquire_i2c(), &mut delay).unwrap();
    let icm = Mpu9250Wrapper::new(icm_inner);

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    let accel_movement_controller = AccelMovementController::new(icm, 1.0);
    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = AccelCompositeController::new(demo_movement_controller, accel_movement_controller);

    app_loop(&mut display, seed_buffer, movement_controller);
    loop {}

}
