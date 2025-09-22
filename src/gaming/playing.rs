use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GameRoutes, GameLetters, GameSettings, Route, GamePlayer, GameFonts, ExplosionTexture};
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
    aircraft_counter: Single<(&mut Text, &mut AircraftCounter)>,
    game_fonts: Res<GameFonts>,
    game_player: Res<GamePlayer>,
    time: Res<Time>,
) {
    let (mut text, mut counter) = aircraft_counter.into_inner();
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

            if unit.kind == UnitKind::Aircraft {
                counter.miss += 1;
                *text = Text::new(format!("{}/{}", counter.hit, counter.miss));

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

pub fn on_player_char_input(
    mut commands: Commands,
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut query: Query<(Entity, &FlyingUnit)>,
    game_settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
    window: Single<&Window>
) {
    for event in keyboard_inputs.read() {
        if !event.state.is_pressed() {
            continue;
        }
        if let Key::Character(character) = &event.logical_key
            && let Some(c) = character.chars().next() {
            // 找到玩家输入字符对应的飞行单元（可能多个）
            let missile = asset_server.load("images/missile.png");
            let missile_pos = FIGHTER_JET_MARGIN - window.width()/2. + FIGHTER_JET_SIZE*FIGHTER_JET_SCALE/2.;
            for (entity, unit) in &mut query {
                if unit.letter != c.to_ascii_uppercase() {
                    continue;
                }
                commands.spawn((
                    Sprite {
                        image: missile.clone(),
                        image_mode: SpriteImageMode::Auto,
                        color: Color::WHITE,
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(missile_pos, -20., 0.))
                        .with_scale(Vec3::splat(FIGHTER_JET_SCALE)),
                    Missile {
                        speed: game_settings.missile_speed,
                        target: entity,
                        kind: unit.kind,
                    }
                ));
            }
        }
    }
}

pub fn update_player_score(
    player: ResMut<GamePlayer>,
    mut score_text: Single<&mut Text, With<PlayerScore>>,
) {
    if player.is_changed() {
        **score_text = Text::new(format!("{}", player.player.score));
    }
}

pub fn update_player_missiles(
    mut commands: Commands,
    mut missiles: Query<(Entity, &Missile, &mut Transform), Without<FlyingUnit>>,
    mut player: ResMut<GamePlayer>,
    time: Res<Time>,
    explosion: ResMut<ExplosionTexture>,
    aircraft_counter: Single<(&mut Text, &mut AircraftCounter)>,
    bomb_counter: Single<(&mut Text, &mut BombCounter), Without<AircraftCounter>>,
    shield_counter: Single<(&mut Text, &mut ShieldCounter), (Without<AircraftCounter>, Without<BombCounter>)>,
    health_pack_counter: Single<(&mut Text, &mut HealthPackCounter), (Without<AircraftCounter>, Without<BombCounter>, Without<ShieldCounter>)>,
    flying_units: Query<(&FlyingUnit, &Transform), (With<FlyingUnit>, Without<Missile>)>,
) {
    let (mut aircraft_text, mut aircraft_counter) = aircraft_counter.into_inner();
    let (mut bomb_text, mut bomb_counter) = bomb_counter.into_inner();
    let (mut shield_text, mut shield_counter) = shield_counter.into_inner();
    let (mut health_pack_text, mut health_pack_counter) = health_pack_counter.into_inner();
    for (entity, missile, mut transform) in &mut missiles {
        // 获取目标
        let (unit, target_transform) = if let Ok(t) = flying_units.get(missile.target) {
            t
        } else {
            // 目标不存在则移除导弹
            commands.entity(entity).despawn();
            continue;
        };

        let target_pos = target_transform.translation.truncate();
        let current_pos = transform.translation.truncate();

        // 方向
        let dir = (target_pos - current_pos).normalize_or_zero();

        // 移动
        transform.translation += (dir * missile.speed * time.delta_secs()).extend(0.0);

        // 玩家导弹默认朝 +X
        let angle = dir.y.atan2(dir.x);
        transform.rotation = Quat::from_rotation_z(angle);

        // 命中检测
        if current_pos.distance(target_pos) < 30.0 {
            commands.entity(entity).despawn();
            commands.entity(missile.target).despawn();

            // 更新统计信息
            match unit.kind {
                UnitKind::Aircraft => {
                    aircraft_counter.hit += 1;
                    *aircraft_text = Text::new(format!("{}/{}", aircraft_counter.hit, aircraft_counter.miss));
                    player.player.score += 1;
                },
                UnitKind::Bomb => {
                    bomb_counter.0 += 1;
                    *bomb_text = Text::new(format!("{}", bomb_counter.0));
                    commands.trigger(BombExplodedEvent);
                },
                UnitKind::Shield => {
                    shield_counter.0 += 1;
                    *shield_text = Text::new(format!("{}", shield_counter.0));
                },
                UnitKind::HealthPack => {
                    health_pack_counter.0 += 1;
                    *health_pack_text = Text::new(format!("{}", health_pack_counter.0));
                }
            }

            // 产生爆炸动画
            commands.spawn((
                Sprite::from_atlas_image(
                    explosion.texture.clone(),
                    TextureAtlas {
                        layout: explosion.layout.clone(),
                        index: 0,
                    },
                ),
                Transform::from_translation(target_transform.translation),
                Explosion(Timer::from_seconds(0.05, TimerMode::Repeating)),
            ));
        }
    }
}

pub fn animate_explosion_sheet(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Explosion, &mut Sprite)>,
) {
    for (entity, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() && let Some(atlas) = &mut sprite.texture_atlas {
            if atlas.index < EXPLOSION_SHEET_MAX_INDEX {
                atlas.index += 1;
            } else {
                // 爆炸动画结束，删除爆炸效果
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn update_aircraft_flames(
    mut commands: Commands,
    time: Res<Time>,
    mut flames: Query<(Entity, &Flame, &mut Transform)>,
    fighters: Query<&Transform, With<FighterJet>>,
) {
    for (flame_entity, flame, mut transform) in &mut flames {
        // 获取目标玩家
        let target_transform = if let Ok(t) = fighters.get(flame.target) {
            t
        } else {
            // 玩家不存在则移除导弹
            commands.entity(flame_entity).despawn();
            continue;
        };

        let target_pos = target_transform.translation.truncate();
        let current_pos = transform.translation.truncate();

        // 方向
        let dir = (target_pos - current_pos).normalize_or_zero();

        // 移动
        transform.translation += (dir * flame.speed * time.delta_secs()).extend(0.0);

        // 敌机导弹默认朝 -X
        let angle = dir.y.atan2(dir.x) + std::f32::consts::PI;
        transform.rotation = Quat::from_rotation_z(angle);

        // 命中检测
        if current_pos.distance(target_pos) < 30.0 {
            commands.entity(flame_entity).despawn();
        }
    }
}

pub fn on_bomb_exploded(
    _: On<BombExplodedEvent>,
    mut commands: Commands,
    aircraft: Query<Entity, With<Aircraft>>,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
    window: Single<&Window>
) {
    let missile = asset_server.load("images/missile.png");
    let missile_pos = FIGHTER_JET_MARGIN - window.width()/2. + FIGHTER_JET_SIZE*FIGHTER_JET_SCALE/2.;
    for entity in aircraft.iter() {
        commands.spawn((
            Sprite {
                image: missile.clone(),
                image_mode: SpriteImageMode::Auto,
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(missile_pos, -20., 0.))
                .with_scale(Vec3::splat(FIGHTER_JET_SCALE)),
            Missile {
                speed: game_settings.missile_speed,
                target: entity,
                kind: UnitKind::Aircraft,
            }
        ));
    }
}