use crate::types::ConfiguredPins;
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::InputPin;
use crate::setup::{setup_movement_controller, setup_button_keyboard};
use embedded_graphics::prelude::RgbColor;

pub fn app_loop<UP, DP, LP, RP, DB, TP, DISP>(
    configured_pins: ConfiguredPins<UP, DP, LP, RP, DB, TP>,
    display: &mut DISP,
    seed_buffer: [u8; 32])
where
    UP: InputPin,
    DP: InputPin,
    LP: InputPin,
    RP: InputPin,
    DB: InputPin,
    TP: InputPin,
    DISP: DrawTarget<Color = Rgb565>,
{
    let button_keyboard = setup_button_keyboard(configured_pins);

    let movement_controller = setup_movement_controller(seed_buffer, button_keyboard);

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
