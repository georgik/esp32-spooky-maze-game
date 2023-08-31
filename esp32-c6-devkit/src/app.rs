use crate::{types::ConfiguredPins, devkitc6_composite_controller::DevkitC6CompositeController};
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::InputPin;
use embedded_graphics::prelude::RgbColor;
use crate::ladder_movement_controller::LadderMovementController;
use hal::{adc::{ADC1, AdcPin, ADC}, gpio::{GpioPin, Analog}};
use log::debug;

pub fn app_loop<DISP>(
    adc1: ADC<'_, ADC1>,
    adc_ladder_pin: AdcPin<GpioPin<Analog, 2>, ADC1>,
    display: &mut DISP,
    seed_buffer: [u8; 32]
)
where
    DISP: DrawTarget<Color = Rgb565>,
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
        let _ = display
            .draw_iter(universe.render_frame().into_iter());
    }
}
