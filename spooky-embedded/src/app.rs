use crate::embedded_display::{LCD_H_RES, LCD_PIXELS, LCD_V_RES};
use display_interface::WriteOnlyDataCommand;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics_framebuf::FrameBuf;
use embedded_hal::digital::OutputPin;
use mipidsi::models::Model;
use spooky_core::{
    engine::Engine, movement_controller::MovementController, spritebuf::SpriteBuf,
    universe::Universe,
};

pub fn app_loop<DI, M, RST, MC>(
    display: &mut mipidsi::Display<DI, M, RST>,
    seed_buffer: [u8; 32],
    movement_controller: MC,
) -> !
where
    DI: WriteOnlyDataCommand,
    M: Model<ColorFormat = Rgb565>,
    RST: OutputPin,
    MC: MovementController,
{
    let mut data = [Rgb565::BLACK; LCD_PIXELS];
    let fbuf = FrameBuf::new(&mut data, LCD_H_RES as usize, LCD_V_RES as usize);
    let spritebuf = SpriteBuf::new(fbuf);
    // let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();

    loop {
        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0, 0, LCD_H_RES - 1, LCD_V_RES, pixel_iterator);
    }
}
