use crate::types::ConfiguredPins;
use embedded_graphics::pixelcolor::Rgb565;
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::v2::InputPin;
use display_interface::WriteOnlyDataCommand;
use mipidsi::models::Model;
use embedded_hal::digital::v2::OutputPin;
use crate::setup::{setup_movement_controller, setup_button_keyboard};
use embedded_graphics::prelude::RgbColor;

pub fn app_loop<UP, DP, LP, RP, DB, TP, DI, M, RST>(
    display: &mut mipidsi::Display<DI, M, RST>,
    lcd_h_res:u16,
    lcd_v_res:u16,
    configured_pins: ConfiguredPins<UP, DP, LP, RP, DB, TP>,
    seed_buffer: [u8; 32])
where
    UP: InputPin,
    DP: InputPin,
    LP: InputPin,
    RP: InputPin,
    DB: InputPin,
    TP: InputPin,
    DI: WriteOnlyDataCommand,
    M: Model<ColorFormat = Rgb565>,
    RST: OutputPin,
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
        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0, 0, lcd_v_res-1, lcd_h_res, pixel_iterator);
    }
}
