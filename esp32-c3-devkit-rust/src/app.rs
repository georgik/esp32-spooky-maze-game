use spooky_embedded::embedded_display::{LCD_V_RES, LCD_H_RES, LCD_PIXELS};
use crate::s3box_composite_controller::S3BoxCompositeController;
use embedded_graphics::pixelcolor::Rgb565;
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics::prelude::RgbColor;
use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital::v2::OutputPin;
use mipidsi::models::Model;
use crate::accel_movement_controller::AccelMovementController;
use crate::Accelerometer;

pub fn app_loop<DI, M, RST>(
    display: &mut mipidsi::Display<DI, M, RST>,
    seed_buffer: [u8; 32],
    icm: impl Accelerometer
) where
    DI: WriteOnlyDataCommand,
    M: Model<ColorFormat = Rgb565>,
    RST: OutputPin,
{
    let accel_movement_controller = AccelMovementController::new(icm, 0.2);

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    let movement_controller = S3BoxCompositeController::new(demo_movement_controller, accel_movement_controller);

    let mut data = [Rgb565::BLACK; LCD_PIXELS];
    let fbuf = FrameBuf::new(&mut data, LCD_H_RES as usize, LCD_V_RES as usize);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);

    universe.initialize();

    loop {
        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0, 0, LCD_H_RES, LCD_V_RES, pixel_iterator);
    }
}
