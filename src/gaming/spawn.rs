use rand::Rng;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::{GamePlayer, GameRoutes, GameLetters, GameSettings, GameFonts, Route, GameState};
use crate::gaming::common::*;

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
        let speed = game_settings.level_speeds[level - 1];
        let id = commands.spawn((
            DespawnOnExit(GameState::Gaming),
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
                kind: FlyingUnitKind::Aircraft
            },
            Aircraft::default(),
            children![(
                Text2d::new(letter),
                TextFont {
                    font: game_fonts.letter_font.clone(),
                    font_size: TARGET_LETTER_SIZE / (FIGHTER_JET_SCALE * 0.6),
                    ..Default::default()
                },
                TextColor(TARGET_LETTER_COLOR),
                Transform::from_translation(Vec3::new(AIRCRAFT_SIZE/2.+TARGET_LETTER_SIZE/2.+18., 0.0, 0.0)),
            )]
        )).id();
        route.entities.push(id);
        state.count += 1;

        // 重置到新的随机时间
        if state.count < game_settings.aircraft_count[game_player.player.level as usize - 1] {
            let range = game_settings.aircraft_intervals[level - 1];
            let next_duration = rng.random_range(range.0..=range.1);
            state.timer = Timer::from_seconds(next_duration, TimerMode::Once);
        }
    }
}

#[derive(Default)]
pub struct SpawnState {
    pub timer: Timer,
    pub count: usize,
    pub spawn: bool,
    pub texture: String,
    pub speeds: Vec<(f32, f32)>,
    pub intervals: Vec<(f32, f32)>,
}

#[derive(Resource)]
pub struct BombSpawnState(pub SpawnState);

impl Default for BombSpawnState {
    fn default() -> Self {
        BombSpawnState(SpawnState {
            texture: "images/bomb.png".into(),
            ..default()
        })
    }
}

impl AsMut<SpawnState> for BombSpawnState {
    fn as_mut(&mut self) -> &mut SpawnState {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct ShieldSpawnState(pub SpawnState);

impl Default for ShieldSpawnState {
    fn default() -> Self {
        ShieldSpawnState(SpawnState {
            texture: "images/shield.png".into(),
            ..default()
        })
    }
}

impl AsMut<SpawnState> for ShieldSpawnState {
    fn as_mut(&mut self) -> &mut SpawnState {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct HealthPackSpawnState(pub SpawnState);

impl Default for HealthPackSpawnState {
    fn default() -> Self {
        HealthPackSpawnState(SpawnState {
            texture: "images/first-aid-kit.png".into(),
            ..default()
        })
    }
}

impl AsMut<SpawnState> for HealthPackSpawnState {
    fn as_mut(&mut self) -> &mut SpawnState {
        &mut self.0
    }
}

pub fn spawn_equipment<Marker: Default+Component+FlyingUnitTrait>(
    mut commands: Commands,
    mut spawn_state: ResMut<Marker::SpawnState>,
    mut game_routes: ResMut<GameRoutes>,
    mut game_letters: ResMut<GameLetters>,
    sentence: Option<Res<WarshipSentence>>,
    game_player: Res<GamePlayer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    game_fonts: Res<GameFonts>,
    window: Single<&Window>
)
where
    Marker::SpawnState: AsMut<SpawnState> + Resource
{
    if sentence.is_some() {
        return;
    }

    let state = spawn_state.as_mut().as_mut();
    if state.timer.tick(time.delta()).just_finished() {
        // 达到了创建的时间
        let mut rng = rand::rng();
        let level = game_player.player.level as usize;
        if state.spawn {
            // 随机选择一个将要使用的航道
            let route = random_route(&mut game_routes, &mut rng);
            let letter = random_letter(&mut game_letters, &mut rng);
            // 生成装备
            let texture = asset_server.load(state.texture.clone());
            let speed = state.speeds[level - 1];
            let id = commands.spawn((
                DespawnOnExit(GameState::Gaming),
                Sprite {
                    image: texture.clone(),
                    image_mode: SpriteImageMode::Auto,
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(window.width() / 2., route.get_position(window.height()), 0.))
                    .with_scale(Vec3::splat(FIGHTER_JET_SCALE * 0.6)),
                FlyingUnit {
                    route: route.id,
                    letter,
                    speed: rng.random_range(speed.0..=speed.1),
                    kind: Marker::kind(),
                },
                Marker::default(),
                children![(
                    Text2d::new(letter),
                    TextFont {
                        font: game_fonts.letter_font.clone(),
                        font_size: TARGET_LETTER_SIZE / (FIGHTER_JET_SCALE * 0.6),
                        ..Default::default()
                    },
                    TextColor(TARGET_LETTER_COLOR),
                    Transform::from_translation(Vec3::new(0., -30., 0.)).with_scale(Vec3::splat(0.8)),
                )]
            )).id();
            route.entities.push(id);
            state.count += 1;
        }
        state.spawn = true;

        // 重置到新的随机时间
        let range = state.intervals[level - 1];
        let next_duration = rng.random_range(range.0..=range.1);
        state.timer = Timer::from_seconds(next_duration, TimerMode::Once);
    }
}
