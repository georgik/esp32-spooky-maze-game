#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

use spi_dma_displayinterface::spi_dma_displayinterface;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use esp_println::println;

use hal::{
    clock::{ClockControl, CpuClock},
    dma::DmaPriority,
    gdma::Gdma,
    i2c,
    i2s::{DataFormat, I2s, I2s0New, I2sWriteDma, MclkPin, PinsBclkWsDout, Standard},
    peripherals::Peripherals,
    prelude::*,
    psram,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    Delay, Rng, IO,
};

use spooky_embedded::{
    app::app_loop,
    controllers::{
        accel::AccelMovementController,
        composites::accel_composite::AccelCompositeController
    },
    embedded_display::{LCD_H_RES, LCD_V_RES, LCD_MEMORY_SIZE},
};


use esp_backtrace as _;

use icm42670::{Address, Icm42670};
use shared_bus::BusManagerSimple;

use es8311::{Config, Resolution, SampleFreq};

use log::info;

const TELEPORT_SAMPLE: &[u8] = include_bytes!("../../assets/raw/teleport.raw");

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
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut delay = Delay::new(&clocks);

    println!("About to initialize the SPI LED driver");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let lcd_sclk = io.pins.gpio7;
    let lcd_mosi = io.pins.gpio6;
    let lcd_cs = io.pins.gpio5;
    let lcd_miso = io.pins.gpio3; // random unused pin
    let lcd_dc = io.pins.gpio4.into_push_pull_output();
    let mut lcd_backlight = io.pins.gpio45.into_push_pull_output();
    let lcd_reset = io.pins.gpio48.into_push_pull_output();

    let i2c_sda = io.pins.gpio8;
    let i2c_scl = io.pins.gpio18;

    let dma = Gdma::new(peripherals.DMA);
    let dma_graphics_channel = dma.channel0;

    let mut graphics_descriptors = [0u32; 8 * 3];
    let mut graphics_rx_descriptors = [0u32; 8 * 3];

    let spi = Spi::new(
        peripherals.SPI2,
        lcd_sclk,
        lcd_mosi,
        lcd_miso,
        lcd_cs,
        60u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    )
    .with_dma(dma_graphics_channel.configure(
        false,
        &mut graphics_descriptors,
        &mut graphics_rx_descriptors,
        DmaPriority::Priority0,
    ));

    println!("SPI ready");

    let di = spi_dma_displayinterface::new_no_cs(LCD_MEMORY_SIZE, spi, lcd_dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_ms(500u32);

    let mut display = match mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(LCD_H_RES, LCD_V_RES)
        .with_orientation(mipidsi::Orientation::PortraitInverted(false))
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(lcd_reset))
    {
        Ok(display) => display,
        Err(_e) => {
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

    let i2c = i2c::I2C::new(peripherals.I2C0, i2c_sda, i2c_scl, 100u32.kHz(), &clocks);

    // let bus = BusManagerSimple::new(i2c);
    // let icm = Icm42670::new(bus.acquire_i2c(), Address::Primary).unwrap();

    let mut rng = Rng::new(peripherals.RNG);
    let mut seed_buffer = [0u8; 32];
    rng.read(&mut seed_buffer).unwrap();

    // let accel_movement_controller = AccelMovementController::new(icm, 0.2);
    // let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    // let movement_controller = AccelCompositeController::new(demo_movement_controller, accel_movement_controller);
    let movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);


    info!("Initializing audio");
    let mut pa_ctrl = io.pins.gpio46.into_push_pull_output();
    pa_ctrl.set_high().unwrap();

    let mut es8311 = es8311::Es8311::new(i2c, es8311::Address::Primary);

    let cfg = Config {
        sample_frequency: SampleFreq::Freq44KHz,
        mclk: Some(es8311::MclkFreq::Freq2822KHz),
        res_in: Resolution::Resolution16,
        res_out: Resolution::Resolution16,
        mclk_inverted: false,
        sclk_inverted: true,
    };

    es8311.init(delay, &cfg).unwrap();
    info!("init done");
    es8311.voice_mute(false).unwrap();
    es8311.set_voice_volume(180).unwrap();

    let mut audio_tx_descriptors = [0u32; 20 * 3];
    let mut audio_rx_descriptors = [0u32; 8 * 3];

    let dma_audio_channel = dma.channel1;

    let i2s = I2s::new(
        peripherals.I2S0,
        MclkPin::new(io.pins.gpio2),
        Standard::Philips,
        DataFormat::Data16Channel16,
        44100u32.Hz(),
        dma_audio_channel.configure(
            false,
            &mut audio_tx_descriptors,
            &mut audio_rx_descriptors,
            DmaPriority::Priority0,
        ),
        &clocks,
    );

    let i2s_tx = i2s.i2s_tx.with_pins(PinsBclkWsDout::new(
        io.pins.gpio17,
        io.pins.gpio47,
        io.pins.gpio15,
    ));

    let data =
    unsafe { core::slice::from_raw_parts(TELEPORT_SAMPLE as *const _ as *const u8, TELEPORT_SAMPLE.len()) };

    let buffer = dma_buffer();
    let mut idx = 0;
    for i in 0..usize::min(data.len(), buffer.len()) {
        buffer[i] = data[idx];

        idx += 1;

        if idx >= data.len() {
            idx = 0;
        }
    }

    let mut filler = [0u8; 10000];

    let mut transfer = i2s_tx.write_dma_circular(buffer).unwrap();
    let avail = transfer.available();
    if avail > 0 {
        let avail = usize::min(10000, avail);
        for bidx in 0..avail {
            filler[bidx] = data[idx];
            idx += 1;

            if idx >= data.len() {
                idx = 0;
            }
        }
        transfer.push(&filler[0..avail]).unwrap();
    }

    info!("Entering main loop");
    app_loop(&mut display, seed_buffer, movement_controller);
    loop {}
}


fn dma_buffer() -> &'static mut [u8; 32000] {
    static mut BUFFER: [u8; 32000] = [0u8; 32000];
    unsafe { &mut BUFFER }
}
