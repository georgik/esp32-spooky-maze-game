#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

// use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::Point,
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ClockControl, CpuClock},
    dma::DmaPriority,
    gdma::Gdma,
    i2c,
    peripherals::Peripherals,
    prelude::*,
    psram,
    spi::{master::{prelude::*, Spi}, SpiMode},
    Delay,
    Rng,
    IO
};

mod app;
use app::app_loop;

mod accel_movement_controller;
mod s3box_composite_controller;
// mod setup;

// mod types;

use esp_backtrace as _;

mod spi_dma_displayinterface;

// #[cfg(any(feature = "imu_controls"))]
use icm42670::{accelerometer::Accelerometer, Address, Icm42670};
// #[cfg(any(feature = "imu_controls"))]
use shared_bus::BusManagerSimple;

fn init_psram_heap() {
    unsafe {
        ALLOCATOR.init(psram::psram_vaddr_start() as *mut u8, psram::PSRAM_BYTES);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    psram::init_psram(peripherals.PSRAM);
    init_psram_heap();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    // let (unconfigured_pins, /*configured_pins, */mut configured_system_pins) = setup_pins(io.pins);

    let lcd_sclk = io.pins.gpio7;
    let lcd_mosi = io.pins.gpio6;
    let lcd_cs = io.pins.gpio5;
    let lcd_miso = io.pins.gpio2; // random unused pin
    let lcd_dc = io.pins.gpio4.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio45.into_push_pull_output();
    let lcd_reset = io.pins.gpio48.into_push_pull_output();

    let i2c_sda = io.pins.gpio8;
    let i2c_scl = io.pins.gpio18;

    let dma = Gdma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

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

    println!("SPI ready");

    // let di = SPIInterfaceNoCS::new(spi, configured_system_pins.dc);
    let di = spi_dma_displayinterface::SPIInterfaceNoCS::new(spi, lcd_dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);

    let mut display = match mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(lcd_reset)) {
        Ok(display) => display,
        Err(e) => {
            // Handle the error and possibly exit the application
            panic!("Display initialization failed");
        }
    };

    let _ = lcd_backlight.set_high();

    println!("Initializing...");
        Text::new(
            "Initializing...",
            Point::new(80, 110),
            MonoTextStyle::new(&FONT_8X13, RgbColor::WHITE),
        )
        .draw(&mut display)
        .unwrap();



    // #[cfg(any(feature = "imu_controls"))]
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        i2c_sda,
        i2c_scl,
        100u32.kHz(),
        &clocks,
    );

    // #[cfg(any(feature = "imu_controls"))]
    let bus = BusManagerSimple::new(i2c);
    // #[cfg(any(feature = "imu_controls"))]
    let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    // TODO: Figure out type for display which will have set_pixels
    use crate::s3box_composite_controller::S3BoxCompositeController;
    use embedded_graphics::pixelcolor::Rgb565;
    use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
    use embedded_graphics_framebuf::FrameBuf;
    use embedded_graphics::prelude::RgbColor;
    use crate::accel_movement_controller::AccelMovementController;
    let accel_movement_controller = AccelMovementController::new(icm, 0.2);

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = S3BoxCompositeController::new(demo_movement_controller, accel_movement_controller);

    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();
    println!("Entering main loop");
    // app_loop( &mut display, seed_buffer, icm);
    loop {
        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0,0,320,240, pixel_iterator);
    }

}
