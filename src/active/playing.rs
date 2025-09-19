use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::input::keyboard::Key;
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GameRoutes, GameLetters, GameSettings, Route, GamePlayer};
use crate::active::common::*;
use crate::active::compute_route_count;

pub fn playground_setup(mut commands: Commands,
                        mut game_routes: ResMut<GameRoutes>,
                        mut game_letters: ResMut<GameLetters>,
                        mut game_player: ResMut<GamePlayer>,
                        game_settings: Res<GameSettings>,
                        asset_server: Res<AssetServer>,
                        window: Single<&Window>) {
    // 玩家的战斗机
    let fighter_jet_path = format!("images/fighter_jet_{}.png", game_player.player.level);
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
    game_letters.candidate_letters = game_settings.level_letters[game_player.player.level as usize - 1].clone();

    // 计算玩家的安全距离
    game_player.safe_position = -(window.width() / 2. - FIGHTER_JET_MARGIN - FIGHTER_JET_SIZE * FIGHTER_JET_SCALE - 100.);
}

pub fn move_fly_unit(
    mut commands: Commands,
    mut query: Query<(Entity, &FlyingUnit, &mut Transform)>,
    mut game_routes: ResMut<GameRoutes>,
    mut game_letters: ResMut<GameLetters>,
    game_player: Res<GamePlayer>,
    time: Res<Time>,
) {
    for (entity, unit, mut transform) in &mut query {
        // 沿着 -X 方向移动
        transform.translation.x -= unit.speed * time.delta_secs();

        // 到达销毁边界时，移除整个实体
        if transform.translation.x < game_player.safe_position {
            if let Some(pos) = game_routes.used_routes.iter().position(|r| r.id == unit.route) {
                // 如果只包含这一个 Entity，则移除整个 Route
                if game_routes.used_routes[pos].entities.len() == 1
                    && game_routes.used_routes[pos].entities[0] == entity {
                    let mut route = game_routes.used_routes[pos].clone();
                    route.entities.clear();
                    game_routes.empty_routes.push(route);
                    game_routes.used_routes.remove(pos);
                } else {
                    // 否则仅移除目标 Entity
                    game_routes.used_routes[pos].entities.retain(|&e| e != entity);
                }
            }

            if !game_letters.candidate_letters.contains(&unit.letter) {
                game_letters.candidate_letters.push(unit.letter);
            }
            commands.entity(entity).despawn();
        }
    }
}
