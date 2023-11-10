use crate::devkitc6_composite_controller::DevkitC6CompositeController;
use display_interface::WriteOnlyDataCommand;
use embedded_graphics::pixelcolor::Rgb565;
use mipidsi::models::Model;
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics::prelude::RgbColor;
use crate::ladder_movement_controller::LadderMovementController;
use embedded_hal::digital::v2::OutputPin;
use hal::{adc::{ADC1, AdcPin, ADC}, gpio::{GpioPin, Analog}};

pub fn app_loop<DI, M, RST>(
    display: &mut mipidsi::Display<DI, M, RST>,
    lcd_h_res:u16,
    lcd_v_res:u16,
    adc1: ADC<'_, ADC1>,
    adc_ladder_pin: AdcPin<GpioPin<Analog, 2>, ADC1>,
    seed_buffer: [u8; 32]
)
where
    DI: WriteOnlyDataCommand,
    M: Model<ColorFormat = Rgb565>,
    RST: OutputPin,
{
    let ladder_movement_controller = LadderMovementController::new(adc1, adc_ladder_pin);

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);

    let movement_controller = DevkitC6CompositeController::new(demo_movement_controller, ladder_movement_controller);

    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();

    loop {
        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0, 0, lcd_v_res, lcd_h_res, pixel_iterator);
    }
}
