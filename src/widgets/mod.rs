use std::sync::OnceLock;
use bevy::prelude::*;

mod button;
pub use button::*;

pub const UI_FONT_SIZE: f32 = 18.0;

pub static UI_BUTTON_FONT: OnceLock<Handle<Font>> = OnceLock::new();

#[derive(Default, Component)]
pub struct Selected;

pub fn widgets_plugin(app: &mut App) {
    app
        .add_event::<ButtonClicked>()
        .add_systems(Update, (button_interaction_system,
                              button_style_selected_system, 
                              button_style_unselected_system));
}
