use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::{default, Commands, Res, Single, Sprite, SpriteImageMode, Time, Transform, Window};
use crate::{GameData, GameFonts, Players};

pub fn game_player_setup(mut commands: Commands,
                         game_data: Res<GameData>,
                         asset_server: Res<AssetServer>,
                         window: Single<&Window>) {
    let fighter_jet_path = format!("images/fighter_jet_{}.png", game_data.player.level);
    let texture = asset_server.load(fighter_jet_path);
    commands.spawn((
        Sprite {
            image: texture.clone(),
            image_mode: SpriteImageMode::Auto,
            color: Color::WHITE,
            ..default()
        },
        Transform::from_translation(Vec3::new(100. - window.width()/2., 0., 0.)).with_scale(Vec3::splat(0.1)),
    ));
}