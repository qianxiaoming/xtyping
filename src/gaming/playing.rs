use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::color::palettes::css::RED;
use bevy::input::keyboard::Key;
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GameRoutes, GameLetters, GameSettings, Route, GamePlayer, GameFonts, PlayState};
use crate::gaming::common::*;
use crate::gaming::compute_route_count;

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
    game_fonts: Res<GameFonts>,
    game_player: Res<GamePlayer>,
    time: Res<Time>,
) {
    for (entity, unit, mut transform) in &mut query {
        // 沿着 -X 方向移动
        transform.translation.x -= unit.speed * time.delta_secs();

        // 到达销毁边界时，移除整个实体
        if transform.translation.x < game_player.safe_position {
            if let Some(pos) = game_routes.used_routes.iter().position(|r| r.id == unit.route) {
                // 如果只包含这一个 Entity，则将Route转移到未使用队列
                if game_routes.used_routes[pos].entities.len() == 1
                    && game_routes.used_routes[pos].entities[0] == entity {
                    let mut route = game_routes.used_routes.swap_remove(pos);
                    route.entities.clear();
                    game_routes.empty_routes.push(route);
                } else {
                    // 否则仅移除目标 Entity
                    game_routes.used_routes[pos].entities.retain(|&e| e != entity);
                }
            }

            if !game_letters.candidate_letters.contains(&unit.letter) {
                game_letters.candidate_letters.push(unit.letter);
            }
            commands.entity(entity).despawn();

            // 生成Miss文字动画
            commands.spawn((
                Text2d::new("MISS"),
                TextFont {
                    font: game_fonts.normal_font.clone(),
                    font_size: 24.,
                    ..Default::default()
                },
                TextColor(Color::srgb_u8(255, 100, 100)),
                Transform::from_translation(transform.translation),
                MissText(Timer::from_seconds(1., TimerMode::Once))
                ));
        }
    }
}

pub fn animate_miss_text(
    time: Res<Time>,
    mut query: Query<(&mut TextColor, &mut Transform, &mut MissText), With<MissText>>
) {
    for (mut color, mut transform, mut miss) in &mut query {
        miss.0.tick(time.delta());
        let t = miss.0.elapsed_secs();
        let alpha= if 1. - t < 0. { 0. } else { 1. - t };
        color.set_alpha(alpha);
        transform.translation.y += time.delta_secs() * 30.0;
    }
}
