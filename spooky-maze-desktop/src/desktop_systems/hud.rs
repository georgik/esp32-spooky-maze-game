// desktop_systems/hud.rs
use bevy::prelude::*;
use spooky_core::systems::hud::HudState;

/// Marker component for our HUD text node.
#[derive(Component)]
pub struct HudText;

pub fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load a font. (If you want to use a built‐in default, you could try to provide one here.)
    // let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Spawn the HUD root node.
    commands
        .spawn((
            Node {
                // Set the size: full width, fixed height (e.g. 60 px)
                // size: Size::new(Val::Percent(100.0), Val::Px(60.0)),
                // Make the node use absolute positioning at the top‐left
                position_type: PositionType::Absolute,
                // position: UiRect {
                //     top: Val::Px(0.0),
                //     left: Val::Px(0.0),
                //     ..default()
                // },
                // Layout: left‐aligned with some padding
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            // BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
            Name::new("HUD Root"),
        ))
        .with_children(|parent| {
            // Spawn the text child that displays the HUD values.
            parent.spawn((
                // In Bevy 0.16, Text is a tuple struct wrapping a String.
                // (For more advanced styling you could add a TextStyle component.)
                Text("Coins: 0  Teleport: 100  Walker: 0  Dynamite: 0".to_string()),
                // Optionally add a style for the text (e.g. margin).
                // Style {
                //     margin: UiRect::all(Val::Px(5.0)),
                //     ..default()
                // },
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
