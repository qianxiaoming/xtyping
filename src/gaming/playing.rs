use rand::Rng;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GameRoutes, GameLetters, GameSettings, Route, GamePlayer, GameFonts, ExplosionTexture, PlayState, GameState};
use crate::gaming::common::*;
use crate::gaming::{compute_route_count, gradient_health_bar_color};

pub fn playground_setup(
    mut commands: Commands,
    mut game_routes: ResMut<GameRoutes>,
    mut game_letters: ResMut<GameLetters>,
    mut game_player: ResMut<GamePlayer>,
    game_settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
    window: Single<&Window>,
    last_state: Option<Res<LastPlayState>>,
) {
    if last_state.is_some() {
        commands.remove_resource::<LastPlayState>();
        return;
    }
    // 玩家的战斗机
    let fighter_jet_path = format!("images/fighter_jet_{}.png", game_player.player.level);
    let texture = asset_server.load(fighter_jet_path);
    commands.spawn((
        DespawnOnExit(GameState::Gaming),
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
    game_player.safe_position = -(window.width() / 2. - FIGHTER_JET_MARGIN - FIGHTER_JET_SIZE * FIGHTER_JET_SCALE - 50.);
}

pub fn move_fly_unit(
    mut commands: Commands,
    mut query: Query<(Entity, &FlyingUnit, &mut Transform)>,
    mut game_routes: ResMut<GameRoutes>,
    mut game_letters: ResMut<GameLetters>,
    mut counter: ResMut<FlyingUnitCounter>,
    mut aircraft: Query<&mut Aircraft>,
    mut counter_texts: Query<(&mut Text, &FlyingUnitText), With<FlyingUnitText>>,
    game_fonts: Res<GameFonts>,
    game_player: Res<GamePlayer>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    window: Single<&Window>
) {
    let mut text = counter_texts.iter_mut().find_map(
        |(txt, FlyingUnitText(kind))| {
            matches!(kind, FlyingUnitKind::Aircraft).then_some(txt)
        }
    ).unwrap();
    let mut rng = rand::rng();
    for (entity, unit, mut transform) in &mut query {
        // 沿着 -X 方向移动
        transform.translation.x -= unit.speed * time.delta_secs();
        let pos = transform.translation.xy();

        // 到达销毁边界时，移除整个实体
        if pos.x < game_player.safe_position {
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

            // 把字母放回候选队列中
            if !game_letters.candidate_letters.contains(&unit.letter) {
                game_letters.candidate_letters.push(unit.letter);
            }

            // 销毁实体
            commands.entity(entity).despawn();

            // 如果当前是敌机，出现一个“MISS”的文本提示
            if unit.kind == FlyingUnitKind::Aircraft {
                counter.missed += 1;
                *text = Text::new(format!("{}/{}", counter.destroyed, counter.missed));

                // 生成Miss文字动画
                commands.spawn((
                    DespawnOnExit(GameState::Gaming),
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
        } else if unit.kind == FlyingUnitKind::Aircraft && pos.x < 0. {
            // 当前实体是敌机，根据距离判断是准备进入发射火球状态还是可以发射了
            if let Ok(mut ac) = aircraft.get_mut(entity) {
                if ac.ready {
                    if ac.flame.is_none() && pos.x < ac.fire_pos {
                        // 达到了发射位置，现在发射火球
                        let texture = asset_server.load("images/flame.png");
                        let pos_y = if pos.y > 0. { -20. } else { 20. };
                        let flame = commands.spawn((
                            DespawnOnExit(GameState::Gaming),
                            Sprite {
                                image: texture,
                                image_mode: SpriteImageMode::Auto,
                                color: Color::WHITE,
                                ..default()
                            },
                            Transform::from_translation(Vec3::new(pos.x, pos.y + pos_y,0.))
                                .with_scale(Vec3::splat(FIGHTER_JET_SCALE)),
                            Flame { speed: unit.speed * 1.5 }
                        )).id();
                        ac.flame = Some(flame);
                    }
                } else {
                    ac.ready = true;
                    // 敌机的攻击位置设为中间向左窗口宽度四分之一内的随机位置
                    ac.fire_pos = pos.x - rng.random_range(5.0..window.width()/4.);
                }
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

pub fn on_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<PlayState>>
) {
    if keyboard_input.just_released(KeyCode::Space) {
        next_state.set(PlayState::Paused);
    } else if keyboard_input.just_released(KeyCode::Escape) {
        next_state.set(PlayState::Exiting);
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
                    DespawnOnExit(GameState::Gaming),
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

pub fn update_player_status(
    player: ResMut<GamePlayer>,
    mut health_bars: Query<(Entity, &mut HealthBar)>,
    mut score_text: Single<&mut Text, With<PlayerScore>>,
    children_query: Query<&Children>,
    mut color_query: Query<&mut BackgroundColor>
) {
    if !player.is_changed() {
        return;
    }
    **score_text = Text::new(format!("{}", player.player.score));

    for (entity, mut bar) in health_bars.iter_mut() {
        if bar.role == GameRole::Player && bar.value != player.health {
            if let Ok(children) = children_query.get(entity) {
                for (index, child) in children.iter().enumerate() {
                    let bg_color = if index < player.health as usize {
                        gradient_health_bar_color(index as u16)
                    } else {
                        Color::srgb_u8(70,70,70)
                    };
                    if let Ok(mut color) = color_query.get_mut(child) {
                        *color = BackgroundColor(bg_color);
                    }
                }
            }
            bar.value = player.health;
        }
    }
}

pub fn update_player_missiles(
    mut commands: Commands,
    mut missiles: Query<(Entity, &Missile, &mut Transform), Without<FlyingUnit>>,
    mut player: ResMut<GamePlayer>,
    mut counter: ResMut<FlyingUnitCounter>,
    mut counter_texts: Query<(&mut Text, &FlyingUnitText), With<FlyingUnitText>>,
    aircraft: Query<&Aircraft>,
    time: Res<Time>,
    explosion: ResMut<ExplosionTexture>,
    flying_units: Query<(&FlyingUnit, &Transform), (With<FlyingUnit>, Without<Missile>)>,
) {
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
                FlyingUnitKind::Aircraft => {
                    counter.destroyed += 1;
                    let mut text = counter_texts.iter_mut().find_map(
                        |(txt, FlyingUnitText(kind))| {
                            matches!(kind, FlyingUnitKind::Aircraft).then_some(txt)
                        }
                    ).unwrap();
                    *text = Text::new(format!("{}/{}", counter.destroyed, counter.missed));
                    player.player.score += 1;
                    // 销毁发射的火球
                    if let Ok(ac) = aircraft.get(missile.target) && ac.flame.is_some() {
                        commands.entity(ac.flame.unwrap()).despawn();
                    }
                    // 更新敌机的血条
                    commands.trigger(UpdateHealthBarEvent(counter.destroyed as u16));
                },
                FlyingUnitKind::Bomb => {
                    counter.bomb += 1;
                    let mut text = counter_texts.iter_mut().find_map(
                        |(txt, FlyingUnitText(kind))| {
                            matches!(kind, FlyingUnitKind::Bomb).then_some(txt)
                        }
                    ).unwrap();
                    *text = Text::new(format!("{}", counter.bomb));
                    commands.trigger(BombExplodedEvent);
                },
                FlyingUnitKind::Shield => {
                    counter.shield += 1;
                    let mut text = counter_texts.iter_mut().find_map(
                        |(txt, FlyingUnitText(kind))| {
                            matches!(kind, FlyingUnitKind::Shield).then_some(txt)
                        }
                    ).unwrap();
                    *text = Text::new(format!("{}", counter.shield));
                },
                FlyingUnitKind::HealthPack => {
                    counter.health_pack += 1;
                    let mut text = counter_texts.iter_mut().find_map(
                        |(txt, FlyingUnitText(kind))| {
                            matches!(kind, FlyingUnitKind::HealthPack).then_some(txt)
                        }
                    ).unwrap();
                    *text = Text::new(format!("{}", counter.health_pack));
                    player.health = (player.health + HEALTH_PACK_RESTORE).min(HEALTH_MAX_VALUE);
                }
            }

            // 产生爆炸动画
            commands.spawn((
                DespawnOnExit(GameState::Gaming),
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
    mut game_player: ResMut<GamePlayer>,
    mut flames: Query<(Entity, &Flame, &mut Transform), Without<FighterJet>>,
    time: Res<Time>,
    fighter_jet: Single<&Transform, With<FighterJet>>,
) {
    for (flame_entity, flame, mut transform) in &mut flames {
        // 获取目标玩家
        let target_pos = fighter_jet.translation.truncate();
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
            if game_player.health != 0 {
                game_player.health -= 1;
            }
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
            DespawnOnExit(GameState::Gaming),
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
                kind: FlyingUnitKind::Aircraft,
            }
        ));
    }
}

/// 用于更新敌方的血条
pub fn on_update_health_bar(
    event: On<UpdateHealthBarEvent>,
    mut health_bars: Query<(Entity, &mut HealthBar)>,
    game_settings: Res<GameSettings>,
    player: Res<GamePlayer>,
    children_query: Query<&Children>,
    mut color_query: Query<&mut BackgroundColor>
) {
    let total = game_settings.aircraft_count[(player.player.level-1) as usize] as u16;
    let health = ((total - event.0) as f32 / total as f32 * 100.).ceil() as u16;
    for (entity, mut bar) in health_bars.iter_mut() {
        if bar.role == GameRole::Enemy && bar.value != health {
            if let Ok(children) = children_query.get(entity) {
                for (index, child) in children.iter().enumerate() {
                    let bg_color = if (HEALTH_MAX_VALUE - index as u16) <= health {
                        gradient_health_bar_color(HEALTH_MAX_VALUE - index as u16)
                    } else {
                        Color::srgb_u8(70,70,70)
                    };
                    if let Ok(mut color) = color_query.get_mut(child) {
                        *color = BackgroundColor(bg_color);
                    }
                }
            }
            bar.value = player.health;
        }
    }
}