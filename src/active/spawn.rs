use rand::Rng;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GamePlayer, GameRoutes, GameLetters, GameSettings, GameFonts, Route};
use crate::active::common::*;

fn random_route<'a>(game_data: &'a mut GameRoutes, rng: &mut impl Rng) -> &'a mut Route {
    if game_data.empty_routes.is_empty() {
        let index = rng.random_range(0..game_data.used_routes.len());
        &mut game_data.used_routes[index]
    } else {
        let index = rng.random_range(0..game_data.empty_routes.len());
        let route = game_data.empty_routes.swap_remove(index);
        let index = game_data.used_routes.len();
        game_data.used_routes.push(route.clone());
        &mut game_data.used_routes[index]
    }
}

fn random_letter(game_data: &mut GameLetters, rng: &mut impl Rng) -> char {
    if game_data.candidate_letters.is_empty() {
        let index = rng.random_range(0..game_data.choosed_letters.len());
        game_data.choosed_letters[index]
    } else {
        let index = rng.random_range(0..game_data.candidate_letters.len());
        let letter = game_data.candidate_letters.swap_remove(index);
        if !game_data.choosed_letters.contains(&letter) {
            game_data.choosed_letters.push(letter);
        }
        letter
    }
}

#[derive(Resource, Default)]
pub struct AircraftSpawnState {
    pub timer: Timer,
    pub count: usize,
}

pub fn spawn_aircraft(mut commands: Commands,
                      mut state: ResMut<AircraftSpawnState>,
                      mut game_routes: ResMut<GameRoutes>,
                      mut game_letters: ResMut<GameLetters>,
                      game_player: Res<GamePlayer>,
                      time: Res<Time>,
                      asset_server: Res<AssetServer>,
                      game_settings: Res<GameSettings>,
                      game_fonts: Res<GameFonts>,
                      window: Single<&Window>) {
    if state.timer.tick(time.delta()).just_finished() {
        // 达到了创建新敌机的时间
        let level = game_player.player.level as usize;
        // 随机选择一个敌机将要使用的航道
        let mut rng = rand::rng();
        let route = random_route(&mut game_routes, &mut rng);
        let letter = random_letter(&mut game_letters, &mut rng);
        // 生成敌机
        let kind = rng.random_range(1..=AIRCRAFT_KIND);
        let texture = asset_server.load(format!("images/aircraft_{}.png", kind));
        let speed = game_settings.aircraft_speeds[level - 1];
        let id = commands.spawn((
            Sprite {
                image: texture.clone(),
                image_mode: SpriteImageMode::Auto,
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(window.width()/2., route.get_position(window.height()), 0.))
                .with_scale(Vec3::splat(FIGHTER_JET_SCALE*0.6)),
            FlyingUnit {
                route: route.id,
                letter,
                speed: rng.random_range(speed.0..=speed.1),
            },
            Aircraft,
            children![(
                Text2d::new(letter),
                TextFont {
                    font: game_fonts.letter_font.clone(),
                    font_size: TARGET_LETTER_SIZE * window.scale_factor(),
                    ..Default::default()
                },
                TextColor(Color::srgb_u8(88, 251, 254)),
                Transform::from_translation(Vec3::new(AIRCRAFT_SIZE/2.+TARGET_LETTER_SIZE/2.+10., 0.0, 0.0)),
            )]
        )).id();
        route.entities.push(id);
        state.count += 1;

        // 重置到新的随机时间
        let range = game_settings.aircraft_intervals[level - 1];
        let next_duration = rng.random_range(range.0..=range.1);
        state.timer = Timer::from_seconds(next_duration, TimerMode::Once);
    }
}

#[derive(Resource, Default)]
pub struct BombSpawnState {
    pub timer: Timer,
    pub count: usize,
    pub spawn: bool
}

pub fn spawn_bomb(mut commands: Commands,
                  mut state: ResMut<BombSpawnState>,
                  mut game_routes: ResMut<GameRoutes>,
                  mut game_letters: ResMut<GameLetters>,
                  game_player: Res<GamePlayer>,
                  time: Res<Time>,
                  asset_server: Res<AssetServer>,
                  game_settings: Res<GameSettings>,
                  game_fonts: Res<GameFonts>,
                  window: Single<&Window>) {
    if state.timer.tick(time.delta()).just_finished() {
        // 达到了创建新炸弹的时间（跳过第一次生成）
        let mut rng = rand::rng();
        let level = game_player.player.level as usize;
        if state.spawn {
            // 随机选择一个将要使用的航道
            let route = random_route(&mut game_routes, &mut rng);
            let letter = random_letter(&mut game_letters, &mut rng);
            // 生成炸弹
            let texture = asset_server.load("images/bomb.png");
            let speed = game_settings.aircraft_speeds[level - 1];
            let id = commands.spawn((
                Sprite {
                    image: texture.clone(),
                    image_mode: SpriteImageMode::Auto,
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(window.width()/2., route.get_position(window.height()), 0.))
                    .with_scale(Vec3::splat(FIGHTER_JET_SCALE*0.6)),
                FlyingUnit {
                    route: route.id,
                    letter,
                    speed: rng.random_range(speed.0..=speed.1),
                },
                Bomb,
                children![(
                    Text2d::new(letter),
                    TextFont {
                        font: game_fonts.letter_font.clone(),
                        font_size: TARGET_LETTER_SIZE * window.scale_factor(),
                        ..Default::default()
                    },
                    TextColor(Color::srgb_u8(88, 251, 254)),
                    Transform::from_translation(Vec3::new(AIRCRAFT_SIZE/2.+TARGET_LETTER_SIZE/2., 0., 0.)),
                )]
            )).id();
            route.entities.push(id);
            state.count += 1;
        }
        state.spawn = true;

        // 重置到新的随机时间
        let range = game_settings.bomb_intervals[level - 1];
        let next_duration = rng.random_range(range.0..=range.1);
        state.timer = Timer::from_seconds(next_duration, TimerMode::Once);
    }
}