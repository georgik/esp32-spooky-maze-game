// desktop_systems/hud.rs
use bevy::prelude::*;
use spooky_core::systems::hud::HudState;

/// Marker component for our HUD text node.
#[derive(Component)]
pub struct HudText;

pub fn setup_hud(mut commands: Commands) {
    // Spawn the HUD root node.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            Name::new("HUD Root"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text("Coins: 0  Teleport: 100  Walker: 0  Dynamite: 0".to_string()),
                HudText,
                Name::new("HUD Text"),
            ));
        });
}

/// Updates the HUD text based on the current HudState resource.
pub fn update_hud(hud_state: Res<HudState>, mut query: Query<&mut Text, With<HudText>>) {
    if hud_state.is_changed() {
        for mut text in query.iter_mut() {
            // Overwrite the text with the new HUD values.
            *text = Text(format!(
                "Coins: {}  Teleport: {}  Walker: {}  Dynamite: {}",
                hud_state.coins_left,
                hud_state.teleport_countdown,
                hud_state.walker_timer,
                hud_state.dynamites,
            ));
        }
    }
}
