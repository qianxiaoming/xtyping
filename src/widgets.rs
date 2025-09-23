use std::sync::OnceLock;
use bevy::prelude::*;

mod button;
mod input;
mod listview;
mod dialog;

pub use button::*;
pub use input::*;
pub use listview::*;
pub use dialog::*;

pub const UI_FONT_SIZE: f32 = 18.0;

pub static UI_BUTTON_FONT: OnceLock<Handle<Font>> = OnceLock::new();

#[derive(Clone)]
pub struct TextConfig {
    pub text: String,
    pub font: Handle<Font>,
    pub font_size: f32,
    pub color: Color,
    pub shadow: bool
}

impl Default for TextConfig {
    fn default() -> Self {
        TextConfig {
            text: Default::default(),
            font: Default::default(),
            font_size: UI_FONT_SIZE,
            color: Color::WHITE,
            shadow: false
        }
    }
}

impl TextConfig {
    pub fn to_shadow(&self) -> TextShadow {
        if self.shadow {
            TextShadow::default()
        } else {
            TextShadow {
                offset: Vec2::splat(0.0),
                color: Color::BLACK
            }
        }
    }
}

#[derive(Default, Component)]
pub struct Selected;

#[derive(Component)]
pub struct Enabled(bool);

impl Default for Enabled {
    fn default() -> Self {
        Enabled(true)
    }
}

pub fn widgets_plugin(app: &mut App) {
    app
        .add_message::<ButtonClicked>()
        .add_message::<ListViewSelectionChanged>()
        .add_systems(Update, (button_interaction_system,
                              button_style_selected_system,
                              button_style_unselected_system))
        .add_systems(Update, (input_box_handle_focus.run_if(|q: Query<(), With<InputBox>>| !q.is_empty()),
                              (input_box_blink_cursor,
                               input_box_ime_events,
                               input_box_keyboard_events)
                                  .run_if(resource_exists::<InputFocused>)))
        .add_systems(Update, (listview_interaction_system.run_if(|q: Query<(), With<ListViewMarker>>| !q.is_empty()),
                              listview_cursor_move_system.run_if((|q: Query<(), With<ListViewMarker>>| !q.is_empty())
                                  .and(on_message::<CursorMoved>))));
}
