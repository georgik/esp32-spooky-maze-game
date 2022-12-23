
use embedded_graphics::{
    pixelcolor::Rgb565,
};
use tinybmp::Bmp;

pub struct Assets<'a> {
    pub ground: Option<Bmp<'a, Rgb565>>,
    pub wall: Option<Bmp<'a, Rgb565>>,
    pub empty: Option<Bmp<'a, Rgb565>>,
    pub ghost1: Option<Bmp<'a, Rgb565>>,
    pub ghost2: Option<Bmp<'a, Rgb565>>,
    pub coin: Option<Bmp<'a, Rgb565>>,
    pub dynamite: Option<Bmp<'a, Rgb565>>,
    pub explosion1: Option<Bmp<'a, Rgb565>>,
    pub explosion2: Option<Bmp<'a, Rgb565>>,
    pub scorched: Option<Bmp<'a, Rgb565>>,
    pub npc: Option<Bmp<'a, Rgb565>>,
    pub teleport: Option<Bmp<'a, Rgb565>>,
    pub walker: Option<Bmp<'a, Rgb565>>,
    pub smiley: Option<Bmp<'a, Rgb565>>,
}

impl Assets<'static> {
    pub fn new() -> Assets<'static> {
        Assets {
            ground: None,
            wall: None,
            empty: None,
            ghost1: None,
            ghost2: None,
            coin: None,
            dynamite: None,
            explosion1: None,
            explosion2: None,
            scorched: None,
            npc: None,
            teleport: None,
            walker: None,
            smiley: None,
        }
    }

    pub fn load(&mut self) {
        self.ground = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/ground.bmp")).unwrap());
        self.wall = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/wall.bmp")).unwrap());
        self.empty = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/empty.bmp")).unwrap());
        self.ghost1 = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/ghost1.bmp")).unwrap());
        self.ghost2 = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/ghost2.bmp")).unwrap());
        self.coin = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/coin.bmp")).unwrap());
        self.dynamite = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/dynamite.bmp")).unwrap());
        self.explosion1 = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/explosion1.bmp")).unwrap());
        self.explosion2 = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/explosion2.bmp")).unwrap());
        self.scorched = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/scorched.bmp")).unwrap());
        self.npc = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/npc.bmp")).unwrap());
        self.teleport = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/teleport.bmp")).unwrap());
        self.walker = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/walker.bmp")).unwrap());
        self.smiley = Some(Bmp::<Rgb565>::from_slice(include_bytes!("../../assets/img/smiley.bmp")).unwrap());
    }
}
