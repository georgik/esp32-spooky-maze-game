use crate::s3box_composite_controller::S3BoxCompositeController;
use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics::prelude::RgbColor;
use crate::accel_movement_controller::AccelMovementController;
use crate::Accelerometer;

pub fn app_loop<DISP>(
    display: &mut DISP,
    seed_buffer: [u8; 32],
    // icm: impl Accelerometer // You'll need to pass your accelerometer device here
)
where
    DISP: DrawTarget<Color = Rgb565>,
{
    // let accel_movement_controller = AccelMovementController::new(icm, 0.2);

    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    // let movement_controller = S3BoxCompositeController::new(demo_movement_controller, accel_movement_controller);

    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, Some(seed_buffer));

    let mut universe = Universe::new_with_movement_controller(engine, demo_movement_controller);

    universe.initialize();

    loop {
        let _ = display
            .draw_iter(universe.render_frame().into_iter());
    }
}
