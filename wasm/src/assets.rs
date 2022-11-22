
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*
};
use tinybmp::Bmp;

pub struct Assets<'a> {
    pub ground: Option<Bmp<'a, Rgb565>>,
    pub wall: Option<Bmp<'a, Rgb565>>,
    pub empty: Option<Bmp<'a, Rgb565>>,
    pub ghost1: Option<Bmp<'a, Rgb565>>,
    pub ghost2: Option<Bmp<'a, Rgb565>>,
}

impl Assets<'static> {
    pub fn new() -> Assets<'static> {
        Assets {
            ground: None,
            wall: None,
            empty: None,
            ghost1: None,
            ghost2: None,
        }
    }

    pub fn load(&mut self) {
        self.ground = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/ground.bmp")).unwrap());
        self.wall = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/wall.bmp")).unwrap());
        self.empty = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/empty.bmp")).unwrap());
        self.ghost1 = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/ghost1.bmp")).unwrap());
        self.ghost2 = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/ghost2.bmp")).unwrap());
    }
}
