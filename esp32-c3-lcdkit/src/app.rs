
use embedded_graphics::pixelcolor::Rgb565;
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics::prelude::RgbColor;
use embedded_hal::digital::v2::OutputPin;
use display_interface::WriteOnlyDataCommand;
use mipidsi::models::Model;

pub fn app_loop<DI, M, RST>(
    display: &mut mipidsi::Display<DI, M, RST>,
    lcd_h_res:u16,
    lcd_v_res:u16,
    seed_buffer: [u8; 32],
    // icm: impl Accelerometer // You'll need to pass your accelerometer device here
) where
    DI: WriteOnlyDataCommand,
    M: Model<ColorFormat = Rgb565>,
    RST: OutputPin,
{
    // let accel_movement_controller = AccelMovementController::new(icm, 0.2);

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    // let movement_controller = S3BoxCompositeController::new(demo_movement_controller, accel_movement_controller);

    let mut data = [Rgb565::BLACK; 240 * 240];
    let fbuf = FrameBuf::new(&mut data, 240, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, Some(seed_buffer));
    // engine.switch_game_state(spooky_core::engine::GameState::Playing);

    let mut universe = Universe::new_with_movement_controller(engine, demo_movement_controller);

    universe.initialize();

    loop {
        let pixel_iterator = universe.render_frame().get_pixel_iter();
        let _ = display.set_pixels(0, 0, lcd_v_res, lcd_h_res, pixel_iterator);
    }
}
