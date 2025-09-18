use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GameRoutes, GameLetters, GameSettings, Route, GamePlayer};
use crate::active::common::*;
use crate::active::compute_route_count;

pub fn playground_setup(mut commands: Commands,
                        mut game_routes: ResMut<GameRoutes>,
                        mut game_letters: ResMut<GameLetters>,
                        game_player: Res<GamePlayer>,
                        game_settings: Res<GameSettings>,
                        asset_server: Res<AssetServer>,
                        window: Single<&Window>) {
    // 玩家的战斗机
    let fighter_jet_path = format!("images/fighter_jet_{}.png", game_player.0.level);
    let texture = asset_server.load(fighter_jet_path);
    commands.spawn((
        Sprite {
            image: texture.clone(),
            image_mode: SpriteImageMode::Auto,
            color: Color::WHITE,
            ..default()
        },
        Transform::from_translation(Vec3::new(FIGHTER_JET_MARGIN - window.width()/2., 0., 0.))
            .with_scale(Vec3::splat(FIGHTER_JET_SCALE)),
        FighterJet,
    ));

    // 计算并创建敌机的航道
    let route_count = compute_route_count(window.height());
    game_routes.empty_routes = Vec::with_capacity(route_count);
    for i in 0..route_count {
        game_routes.empty_routes.push(Route {
            id: i as i32,
            entities: Vec::new(),
        })
    }

    // 加载玩家等级对应的字符
    game_letters.candidate_letters = game_settings.level_letters[game_player.0.level as usize - 1].clone();
}