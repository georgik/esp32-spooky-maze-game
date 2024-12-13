use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_graphics_framebuf::FrameBuf;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use spooky_core::{engine::Engine, spritebuf::SpriteBuf, universe::Universe};
use std::time::{Duration, Instant};

mod desktop_movement_controller;
use desktop_movement_controller::DesktopMovementControllerBuilder;

mod keyboard_movement_controller;
use keyboard_movement_controller::KeyboardMovementController;

use spooky_core::demo_movement_controller::DemoMovementController;
use spooky_core::movement_controller::MovementController;

fn get_seed_buffer() -> Option<[u8; 32]> {
    let mut seed_buffer = [0u8; 32];
    getrandom::getrandom(&mut seed_buffer).unwrap();
    Some(seed_buffer)
}

fn main() -> Result<(), core::convert::Infallible> {
    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("ESP32 Spooky Maze", &output_settings);

    let mut data = [Rgb565::BLACK; 320 * 240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let mut last_user_activity = Instant::now();
    let user_inactivity_timeout = Duration::from_secs(60); // Switch to demo mode after 1 minute of inactivity

    let engine = Engine::new(spritebuf, get_seed_buffer());

    let demo_movement_controller = DemoMovementController::new(get_seed_buffer().unwrap());
    let keyboard_movement_controller = KeyboardMovementController::new();
    let desktop_movement_controller = DesktopMovementControllerBuilder::new(
        demo_movement_controller,
        keyboard_movement_controller,
    );

    let mut universe = Universe::new_with_movement_controller(engine, desktop_movement_controller);
    universe.initialize();

    let mut display = SimulatorDisplay::new(Size::new(320, 200));
    display
        .draw_iter(universe.render_frame().into_iter())
        .unwrap();
    window.update(&display);

    let mut is_demo = true;
    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    last_user_activity = Instant::now(); // Record the time of the last user activity

                    let main_controller = universe.get_movement_controller_mut();
                    main_controller.handle_key(keycode);

                    if is_demo {
                        main_controller.set_active(1);
                        is_demo = false;
                    }
                }
                SimulatorEvent::KeyUp {
                    keycode: _keycode,
                    keymod: _keymod,
                    repeat: _repeat,
                } => {
                    let main_controller = universe.get_movement_controller_mut();
                    main_controller.stop_movement();
                }
                _ => {}
            }
        }

        // Check for user inactivity and switch back to demo mode if necessary
        if last_user_activity.elapsed() > user_inactivity_timeout {
            universe.set_active(0);
            is_demo = true;
        }

        display
            .draw_iter(universe.render_frame().into_iter())
            .unwrap();

        window.update(&display);
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
