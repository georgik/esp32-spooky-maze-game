#![cfg_attr(not(feature = "std"), no_std)]
use core::ptr;

use embedded_graphics::prelude::Point;
use spooky_core::assets::Assets;

use core::ffi::c_void;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}

static mut ASSETS: Option<Assets<'static>> = None;

#[no_mangle]
pub extern "C" fn load_assets() {
    let mut assets = Assets::new();
    assets.load();
    unsafe {
        ASSETS = Some(assets);
    }
}

static mut IMAGE_DATA: [u8; 16 * 16 * 3] = [200; 16 * 16 * 3];


#[no_mangle]
pub extern "C" fn get_ghost1_image() -> *const u8 {
    use spooky_core::{engine::Engine, spritebuf::SpriteBuf};
    use embedded_graphics::{pixelcolor::Rgb565, prelude::DrawTarget};
    use embedded_graphics_framebuf::FrameBuf;
    use embedded_graphics::prelude::RgbColor;
    use embedded_graphics::image::Image;
    use embedded_graphics::Drawable;
    use embedded_graphics::pixelcolor::raw::ToBytes;

    let mut data = [Rgb565::BLACK; 16 * 16];
    let fbuf = FrameBuf::new(&mut data, 16, 16);
    let mut spritebuf = SpriteBuf::new(fbuf);

    unsafe {
        if let Some(assets) = &ASSETS {
            if let Some(ghost1) = &assets.ghost1 {
                let image = Image::new(ghost1, Point::new(0, 0));
                image.draw(&mut spritebuf).unwrap();
                for x in 0..16 {
                    for y in 0..16 {
                        let p = Point::new(x as i32, y as i32);
                        let color = spritebuf.get_color_at(p);
                        IMAGE_DATA[(x + y * 16) * 3] = color.r();
                        IMAGE_DATA[(x + y * 16) * 3 + 1] = color.g();
                        IMAGE_DATA[(x + y * 16) * 3 + 2] = color.b();
                    }
                }
            }
        }
        return IMAGE_DATA.as_ptr();
    }
}

// #[no_mangle]
// pub extern "C" fn get_ghost1_image_size() -> usize {
//     unsafe {
//         if let Some(assets) = &ASSETS {
//             if let Some(ghost1) = &assets.ghost1 {
//                 ghost1.image_data().len()
//             } else {
//                 0
//             }
//         } else {
//             0
//         }
//     }
// }
