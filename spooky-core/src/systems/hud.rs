use bevy::prelude::*;

/// A resource storing the current HUD values.
#[derive(Resource)]
pub struct HudState {
    pub coins_left: u32,
    pub teleport_countdown: u32,
    pub walker_timer: u32,
    pub dynamites: u32,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            coins_left: 100,
            teleport_countdown: 100,
            walker_timer: 0,
            dynamites: 0,
        }
    }
}
