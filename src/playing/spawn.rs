use rand::Rng;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GameData, GameFonts, GameSettings, Route};
use crate::playing::components::{Aircraft, FIGHTER_JET_SCALE};

fn random_route<'a>(game_data: &'a mut GameData, rng: &mut impl Rng) -> &'a mut Route {
    if game_data.empty_routes.is_empty() {
        let index = rng.random_range(0..game_data.used_routes.len());
        &mut game_data.used_routes[index]
    } else {
        let index = rng.random_range(0..game_data.empty_routes.len());
        let route = game_data.empty_routes.swap_remove(index);
        let index = game_data.used_routes.len();
        game_data.used_routes.push(route);
        &mut game_data.used_routes[index]
    }
}

const AIRCRAFT_KIND: i32 = 3;
const AIRCRAFT_SIZE: f32 = 300.;
const AIRCRAFT_COLORS: [Color; 3] = [
    Color::srgb_u8(66, 201, 36),
    Color::srgb_u8(220, 31, 11),
    Color::srgb_u8(255, 222, 0)
];

pub struct AircraftSpawnState {
    pub timer: Timer,
}

impl Default for AircraftSpawnState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

const LETTER_SIZE: f32 = 80.;

pub fn spawn_aircraft(mut commands: Commands,
                      mut state: Local<AircraftSpawnState>,
                      mut game_data: ResMut<GameData>,
                      time: Res<Time>,
                      asset_server: Res<AssetServer>,
                      game_settings: Res<GameSettings>,
                      game_fonts: Res<GameFonts>,
                      ui_scale: Res<UiScale>,
                      window: Single<&Window>) {
    if state.timer.tick(time.delta()).just_finished() {
        // 达到了创建新敌机的时间
        // 随机选择一个敌机将要使用的航道
        let mut rng = rand::rng();
        let route = random_route(&mut game_data, &mut rng);
        // 生成敌机
        let kind = rng.random_range(1..=AIRCRAFT_KIND);
        let texture = asset_server.load(format!("images/aircraft_{}.png", kind));
        let id = commands.spawn((
            Sprite {
                image: texture.clone(),
                image_mode: SpriteImageMode::Auto,
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(window.width()/2., route.get_position(window.height()), 0.))
                .with_scale(Vec3::splat(FIGHTER_JET_SCALE*0.6)),
            Aircraft,
            children![(
                Text2d::new("B"),
                TextFont {
                    font: game_fonts.letter_font.clone(),
                    font_size: LETTER_SIZE * window.scale_factor(),
                    ..Default::default()
                },
                TextColor(AIRCRAFT_COLORS[kind as usize - 1].into()),
                Transform::from_translation(Vec3::new(AIRCRAFT_SIZE/2.+LETTER_SIZE/2.+10., 0.0, 0.0)),
            )]
        )).id();
        route.entities.push(id);

        // 重置到新的随机时间
        let range = game_settings.aircraft_intervals[game_data.player.level as usize + 1];
        let next_duration = rng.random_range(range.0..=range.1);
        state.timer = Timer::from_seconds(next_duration, TimerMode::Once);
    }
}