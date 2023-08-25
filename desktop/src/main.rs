use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_graphics_simulator::{
    sdl2::Keycode, SimulatorDisplay, SimulatorEvent, Window, OutputSettingsBuilder,
};
use embedded_graphics_framebuf::FrameBuf;
use spooky_core::{ spritebuf::SpriteBuf, engine::Engine, universe::Universe, demo_movement_controller::DemoMovementController };
mod keyboard_movement_controller;
use keyboard_movement_controller::KeyboardMovementController;
use std::time::Duration;

fn get_seed_buffer() -> Option<[u8; 32]> {
    let mut seed_buffer = [0u8; 32];
    getrandom::getrandom(&mut seed_buffer).unwrap();
    Some(seed_buffer)
}

fn main() -> Result<(), core::convert::Infallible> {
    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("ESP32 Spooky Maze", &output_settings);

    let mut data = [Rgb565::BLACK ; 320*240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    // let mut movement_controller = KeyboardMovementController::new();
    let mut movement_controller = DemoMovementController::new(get_seed_buffer().unwrap());
    let engine = Engine::new(spritebuf, get_seed_buffer());

    let mut universe = Universe::new_with_movement_controller(engine, movement_controller);
    universe.initialize();
    let mut display = SimulatorDisplay::new(Size::new(320, 200));

    display.draw_iter(universe.render_frame().into_iter()).unwrap();
    window.update(&display);

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                // SimulatorEvent::KeyDown { keycode, .. } => {
                //     if let Some(controller) = universe.get_movement_controller_mut() {
                //         controller.handle_key(keycode);
                //     }
                // },
                // SimulatorEvent::KeyUp { .. } => {
                //     if let Some(controller) = universe.get_movement_controller_mut() {
                //         controller.stop_movement();
                //     }
                // },
                _ => {}
            }
        }
        display.draw_iter(universe.render_frame().into_iter()).unwrap();
        window.update(&display);
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
