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

pub fn spawn_aircraft(
    mut commands: Commands,
    mut state: ResMut<AircraftSpawnState>,
    mut game_routes: ResMut<GameRoutes>,
    mut game_letters: ResMut<GameLetters>,
    game_player: Res<GamePlayer>,
    speed_factor: Res<SpeedFactor>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
    game_fonts: Res<GameFonts>,
    window: Single<&Window>
) {
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
                speed: rng.random_range(speed.0..=speed.1) * speed_factor.speed_factor,
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
            state.timer = Timer::from_seconds(
                next_duration/1.15_f32.powi(speed_factor.factor_changes),
                TimerMode::Once
            );
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

pub fn launch_space_warship(
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    player: Res<GamePlayer>,
    counter: Res<FlyingUnitCounter>
) {
    let total = game_settings.aircraft_count[player.player.level as usize - 1];
    if counter.destroyed + counter.missed >= total {
        commands.insert_resource(SpaceWarshipTimer(Timer::from_seconds(1., TimerMode::Once)))
    }
}

pub fn spawn_space_warship(
    mut commands: Commands,
    mut timer: ResMut<SpaceWarshipTimer>,
    flying_unit: Query<Entity, With<FlyingUnit>>,
    time: Res<Time>,
    player: Res<GamePlayer>,
    settings: Res<GameSettings>,
    game_fonts: Res<GameFonts>,
    assets: Res<AssetServer>,
    window: Single<&Window>
) {
    if timer.0.tick(time.delta()).is_finished() {
        commands.remove_resource::<SpaceWarshipTimer>();

        for entity in flying_unit.iter() {
            commands.entity(entity).despawn();
        }

        let mut rng = rand::rng();
        let level_index = player.player.level as usize - 1;
        let sentence_index = rng.random_range(0..settings.level_sentences[level_index].len());
        let sentence_chars: Vec<_>= settings.level_sentences[level_index][sentence_index].chars().collect();
        let letter_count = sentence_chars.len() as f32;
        let mut index = 0_usize;
        let mut letters = sentence_chars.clone();
        letters.retain(|c| *c != ' ');
        commands.insert_resource(
            WarshipSentence{
                letters,
                current: 0,
            }
        );

        let font_ratio = 0.65;
        let start_x = -(letter_count * CHECKPOINT_LETTER_SIZE * font_ratio / 2.);
        let start_y = -window.height() / 2. + CHECKPOINT_LETTER_SIZE / 2. + 5.;
        let mut x = start_x;
        for letter in &sentence_chars {
            if *letter != ' ' {
                commands.spawn((
                    DespawnOnExit(GameState::Gaming),
                    Text2d::new(letter.to_string()),
                    TextFont {
                        font: game_fonts.letter_font.clone(),
                        font_size: CHECKPOINT_LETTER_SIZE,
                        ..Default::default()
                    },
                    TextColor(if index == 0 { CHECKPOINT_LETTER_TARGET } else { CHECKPOINT_LETTER_WAITING }),
                    Transform::from_translation(
                        Vec3 {
                            x,
                            y: start_y,
                            z: 1.
                        }),
                    WarshipLetter(index),
                ));
                index += 1;
            }
            x += CHECKPOINT_LETTER_SIZE * font_ratio;
        }
        commands.spawn((
            DespawnOnExit(GameState::Gaming),
            Sprite {
                image: assets.load("images/down-arrow.png"),
                image_mode: SpriteImageMode::Auto,
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new(
                start_x,
                start_y + CHECKPOINT_LETTER_SIZE/2. + 15.,
                0.)).with_scale(Vec3::splat(0.6)
            ),
            WarshipLetterArrow
        ));

        // 加载关卡boss
        let half_window = window.width() / 2.;
        let speed = settings.level_speeds[level_index].0 * 0.45;
        let texture = assets.load("images/space-warship.png");
        commands.spawn((
            DespawnOnExit(GameState::Gaming),
            Sprite {
                image: texture.clone(),
                image_mode: SpriteImageMode::Auto,
                color: Color::WHITE,
                ..default()
            },
            Transform::from_translation(Vec3::new((window.width() + WARSHIP_WIDTH) / 2. - 160., 0., 0.)),
            FlyingUnit {
                route: 0,
                letter: sentence_chars[0],
                speed,
                kind: FlyingUnitKind::Warship
            },
            SpaceWarship {
                timer: Timer::from_seconds(1., TimerMode::Repeating),
                fired: false,
                gun_count: 0,
                gun_state: [false; 12],
                gun_pos: [
                    Vec2::new(224. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 104.),
                    Vec2::new(224. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 260.),
                    Vec2::new(456. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 50.),
                    Vec2::new(456. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 315.),
                    Vec2::new(624. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 17.),
                    Vec2::new(624. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 349.),
                    Vec2::new(870. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 133.),
                    Vec2::new(870. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 230.),
                    Vec2::new(870. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 144.),
                    Vec2::new(870. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 219.),
                    Vec2::new(939. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 80.),
                    Vec2::new(939. - WARSHIP_WIDTH / 2., WARSHIP_HEIGHT / 2. - 286.)
                ],
                gun_dist: [
                    half_window + WARSHIP_WIDTH / 2. - 276.,
                    half_window + WARSHIP_WIDTH / 2. - 510.,
                    half_window + WARSHIP_WIDTH / 2. - 680.,
                    half_window + WARSHIP_WIDTH / 2. - 928.,
                    half_window + WARSHIP_WIDTH / 2. - 990.
                ],
                gun_fired: 0,
                cannon: false,
                cannon_pos: Vec2::new(661. - WARSHIP_WIDTH / 2., 0.),
                cannon_dist: half_window + WARSHIP_WIDTH / 2. - 792.,
            }
        ));
    }
}