use bevy::{
    app::AppExit,
    color::palettes::css::CRIMSON,
    ecs::spawn::{SpawnIter, SpawnWith},
    prelude::*,
};
use super::*;

pub fn menu_plugin(app: &mut App) {
    app.insert_resource(Players::default());
}