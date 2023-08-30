use crate::types::ConfiguredPins;
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::InputPin;
use crate::setup::{setup_movement_controller};
use hal::{adc::{AdcConfig, Attenuation, ADC, ADC1, AdcPin}, gpio::{GpioPin, Analog}};
use embedded_graphics::prelude::RgbColor;
use crate::ladder_movement_controller::LadderMovementController;
use embedded_hal::adc::OneShot;

pub fn app_loop<DISP>(
    adc: AdcPin<GpioPin<Analog, 6>, ADC1>,
    display: &mut DISP,
    seed_buffer: [u8; 32],
)
where
    DISP: DrawTarget<Color = Rgb565>,
{
    let ladder_movement_controller = LadderMovementController::new(adc);  // Assuming your LadderMovementController takes AdcType

    let movement_controller = setup_movement_controller(seed_buffer, ladder_movement_controller);

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
