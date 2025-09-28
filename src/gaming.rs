pub mod common;
mod playing;
mod splash;
mod spawn;
mod paused;
mod exiting;
mod checkpoint;
mod upgrade;
mod failed;

use rand::Rng;
use bevy::app::App;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::window::WindowResized;
use crate::{GamePlayer, GameRoutes, GameLetters, GameFonts, GameState, PlayState, Route, GameSettings, Players, save_game_users, Player, MAX_PLAYER_LEVELS};
use crate::{DEFAULT_ROUTE_HEIGHT, GAME_INFO_AREA_HEIGHT, GAME_INFO_AREA_MARGIN, MAX_ROUTE_COUNT};
use crate::ui::*;
use common::*;
use crate::gaming::spawn::{AircraftSpawnState, BombSpawnState, HealthPackSpawnState, ShieldSpawnState};

pub fn play_game_plugin(app: &mut App) {
    app
        .init_resource::<GameRoutes>()
        .init_resource::<GameLetters>()
        .init_resource::<AircraftSpawnState>()
        .init_resource::<BombSpawnState>()
        .init_resource::<ShieldSpawnState>()
        .init_resource::<HealthPackSpawnState>()
        .init_resource::<FlyingUnitCounter>()
        .insert_resource(GameSaveTimer(Timer::from_seconds(10.0, TimerMode::Repeating)))
        .add_observer(playing::on_bomb_exploded)
        .add_observer(playing::on_update_health_bar)
        .add_observer(playing::on_shield_activated)
        .add_observer(playing::on_health_pack_apply)
        .add_systems(OnEnter(GameState::Gaming), playing_game_setup)
        .add_systems(OnExit(GameState::Gaming), playing_game_exit)
        .add_systems(OnEnter(PlayState::Splash), splash::game_splash_setup)
        .add_systems(OnEnter(PlayState::Playing), playing::playground_setup)
        .add_systems(OnEnter(PlayState::Paused), paused::paused_setup)
        .add_systems(OnEnter(PlayState::Exiting), exiting::confirm_exit_setup)
        .add_systems(OnEnter(PlayState::Checkpoint), checkpoint::checkpoint_setup)
        .add_systems(OnEnter(PlayState::Upgrading), upgrade::upgrading_setup)
        .add_systems(OnEnter(PlayState::Failed), failed::player_failed_setup)
        .add_systems(Update, update_game_time)
        .add_systems(Update, on_window_resized.run_if(on_message::<WindowResized>
            .and(in_state(GameState::Gaming))))
        .add_systems(Update, (move_space_stars,
                              twinkle_space_stars,
                              save_game_data).run_if(in_state(GameState::Gaming)))
        .add_systems(Update, restart_game.run_if(in_state(GameState::Restart)))
        .add_systems(Update, splash::fade_tip_messages.run_if(in_state(PlayState::Splash)))
        .add_systems(Update, (spawn::spawn_aircraft,
                              spawn::spawn_equipment::<Bomb>,
                              spawn::spawn_equipment::<Shield>,
                              spawn::spawn_equipment::<HealthPack>,
                              playing::move_fly_unit,
                              playing::animate_miss_text,
                              playing::on_player_char_input,
                              playing::on_keyboard_input,
                              playing::update_aircraft_flames,
                              playing::update_player_status,
                              playing::animate_explosion_sheet).run_if(in_state(PlayState::Playing)))
        .add_systems(Update, playing::update_missiles_for_aircraft.run_if(in_state(PlayState::Playing).and(|res: Option<Res<WarshipSentence>>| res.is_none())))
        .add_systems(Update, playing::update_missiles_for_warship.run_if(in_state(PlayState::Playing).and(resource_exists::<WarshipSentence>)))
        .add_systems(Update, playing::equipment_effect.run_if(in_state(PlayState::Playing).and(|q: Query<(), With<EquipmentEffect>>| !q.is_empty())))
        .add_systems(Update, playing::switch_checkpoint_state.run_if(resource_exists::<CheckpointTimer>))
        .add_systems(Update, paused::on_resume_game.run_if(in_state(PlayState::Paused)))
        .add_systems(Update, exiting::on_exit_game_button.run_if(in_state(PlayState::Exiting)))
        .add_systems(Update, exiting::on_cancel_exit_button.run_if(in_state(PlayState::Exiting)))
        .add_systems(Update, checkpoint::on_exit_game_button.run_if(in_state(PlayState::Checkpoint)))
        .add_systems(Update, checkpoint::on_continue_game_button.run_if(in_state(PlayState::Checkpoint)))
        .add_systems(Update, upgrade::on_continue_game_button.run_if(in_state(PlayState::Upgrading)))
        .add_systems(Update, failed::on_exit_game_button.run_if(in_state(PlayState::Failed)))
        .add_systems(Update, failed::on_continue_game_button.run_if(in_state(PlayState::Failed)))
    ;
}

fn playing_game_setup(mut commands: Commands, 
                      game_player: Res<GamePlayer>,
                      game_settings: Res<GameSettings>,
                      fonts: Res<GameFonts>, 
                      asset_server: Res<AssetServer>, 
                      time: Res<Time>, 
                      window: Single<&Window>,
                      mut aircraft_spawn_state: ResMut<AircraftSpawnState>,
                      mut bomb_spawn_state: ResMut<BombSpawnState>,
                      mut shield_spawn_state: ResMut<ShieldSpawnState>,
                      mut health_pack_spawn_state: ResMut<HealthPackSpawnState>,
                      mut flying_unit_counter: ResMut<FlyingUnitCounter>,
                      mut next_state: ResMut<NextState<PlayState>>) {
    commands.spawn((
        DespawnOnExit(GameState::Gaming),
        Node {
            width: Val::Percent(100.),
            height: Val::Px(GAME_INFO_AREA_HEIGHT),
            display: Display::Grid,
            margin: UiRect::top(Val::Px(10.0)),
            grid_template_columns: vec![
                GridTrack::flex(1.0),
                GridTrack::px(120.0),
                GridTrack::flex(1.0),
            ],
            grid_template_rows: vec![GridTrack::auto()],
            column_gap: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(Color::NONE)
    )).with_children(|builder| {
        // 左边的玩家头像及信息
        builder.spawn(
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    display: Display::Grid,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    grid_template_columns: vec![
                        GridTrack::px(64.0),
                        GridTrack::flex(1.8),
                        GridTrack::flex(0.8),
                        GridTrack::flex(1.2),
                    ],
                    grid_template_rows: vec![
                        GridTrack::flex(2.),
                        GridTrack::flex(1.)
                    ],
                    column_gap: Val::Px(4.0),
                    padding: UiRect::left(Val::Px(10.0)),
                    ..default()
                },
            )
            .with_children(|builder| {
                let player = &game_player.player;
                // 用户头像及升级进度条
                builder.spawn(
                        Node {
                            display: Display::Grid,
                            grid_row: GridPlacement::span(2),
                            ..default()
                        }
                    )
                    .with_children(|builder| {
                        builder.spawn(
                            Node {
                                width: Val::Percent(100.),
                                height: Val::Percent(100.),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            }
                        ).with_children(|builder| {
                            builder.spawn((
                                Node {
                                    width: Val::Percent(100.),
                                    height: Val::Px(5.),
                                    flex_direction: FlexDirection::Row,
                                    ..default()
                                },
                                BorderRadius::MAX,
                                BackgroundColor(Color::srgb_u8(70, 70, 70))
                            )).with_children(|builder| {
                                builder.spawn((
                                    Node {
                                        width: Val::Percent(calculate_upgrade_percent(&game_player.player, &game_settings)),
                                        height: Val::Percent(100.),
                                        ..default()
                                    },
                                    LevelProgress,
                                    BorderRadius::MAX,
                                    BackgroundColor(Color::srgb_u8(70, 139, 254))
                                ));
                            });
                            let avatar = format!("avatars/{}.png", player.avatar.as_str());
                            spawn_image_node(builder, &asset_server, &avatar, Vec2::splat(64.), 0., 3.);
                        });
                    });
                // 用户名称
                spawn_info_text(builder, &game_player.player.name, INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
                // 用户等级
                spawn_marked_image(builder, LevelStarImage, &asset_server, &format!("images/star-{}.png", player.level),
                                 Vec2::new(24.*(player.level as f32), 24.), 0., 0.);
                // 用户分数
                builder.spawn(Node {
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                }).with_children(|builder| {
                    spawn_marked_text(builder, PlayerScore, &format!("{}", player.score), INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
                });
                // 玩家血条
                spawn_health_bar(builder, HealthBar{role: GameRole::Player, value: HEALTH_MAX_VALUE}, 100, 3);
            });

        // 中间的对战图标及时间
        builder.spawn(
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ).with_children(|builder| {
            spawn_image_node(builder, &asset_server, "images/vs.png", Vec2::splat(56.), 0., 0.);
            builder.spawn((
                Text::new("00:00:00"),
                TextFont {
                    font: fonts.ui_font.clone(),
                    font_size: 20.,
                    ..default()
                },
                TextColor(INFO_TEXT_COLOR),
                GameTime {
                    start_time: time.elapsed_secs_f64(),
                    last_second: 0
                }
            ));
        });

        // 右边的敌人及其它资源信息
        builder.spawn(
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Grid,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                grid_template_columns: vec![
                    GridTrack::flex(1.5),
                    GridTrack::flex(1.),
                    GridTrack::flex(1.),
                    GridTrack::flex(1.),
                ],
                grid_template_rows: vec![
                    GridTrack::flex(2.),
                    GridTrack::flex(1.)
                ],
                column_gap: Val::Px(4.0),
                padding: UiRect::right(Val::Px(10.0)),
                ..default()
            },
        ).with_children(|builder| {
            builder.spawn(Node::default()).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/fighter-jet.png", Vec2::splat(30.), 2., 4.);
                spawn_marked_text(builder, FlyingUnitText(FlyingUnitKind::Aircraft), "0/0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            builder.spawn(Node::default()).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/bomb.png", Vec2::splat(30.), 2., 4.);
                spawn_marked_text(builder, FlyingUnitText(FlyingUnitKind::Bomb), "0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            builder.spawn(Node::default()).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/first-aid-kit.png", Vec2::splat(30.), 2., 4.);
                spawn_marked_text(builder, FlyingUnitText(FlyingUnitKind::HealthPack), "0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            builder.spawn(Node::default()).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/shield.png", Vec2::splat(30.), 2., 4.);
                spawn_marked_text(builder, FlyingUnitText(FlyingUnitKind::Shield), "0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            // 敌方血条
            spawn_health_bar(builder, HealthBar{role: GameRole::Enemy, value: HEALTH_MAX_VALUE}, 100, 4);
        });
    });
    spawn_space_stars(&mut commands, &asset_server, window);

    *aircraft_spawn_state = AircraftSpawnState::default();

    // 初始化炸弹的生成参数
    *bomb_spawn_state = BombSpawnState::default();
    let state = &mut bomb_spawn_state.as_mut().0;
    state.speeds = game_settings.level_speeds.clone();
    state.intervals = game_settings.bomb_intervals.clone();
    state.spawn = true;

    // 初始化护盾的生成参数
    *shield_spawn_state = ShieldSpawnState::default();
    let state = &mut shield_spawn_state.as_mut().0;
    state.speeds = game_settings.level_speeds.clone();
    state.intervals = game_settings.shield_intervals.clone();
    state.spawn = true;

    // 初始化血包的生成参数
    *health_pack_spawn_state = HealthPackSpawnState::default();
    let state = &mut health_pack_spawn_state.as_mut().0;
    state.speeds = game_settings.level_speeds.clone();
    state.intervals = game_settings.health_pack_intervals.clone();
    state.spawn = true;

    *flying_unit_counter = FlyingUnitCounter::default();

    next_state.set(PlayState::Splash);
}

pub fn calculate_upgrade_percent(player: &Player, settings: &GameSettings) -> f32 {
    if player.level == MAX_PLAYER_LEVELS {
        return 100.;
    }
    let mut scores = player.score;
    for i in 0..player.level {
        if scores >= settings.upgrade_scores[i as usize] {
            scores -= settings.upgrade_scores[i as usize];
        } else {
            break;
        }
    }

    100. * scores as f32 / settings.upgrade_scores[player.level as usize - 1] as f32
}

fn update_and_save_player(player: &Player, players: &mut Players) {
    for p in players.0.iter_mut() {
        if player.name == p.name {
            if player != p {
                p.level = player.level;
                p.score = player.score;
                save_game_users(players);
                break;
            }
        }
    }
}

fn playing_game_exit(mut players: ResMut<Players>, game_player: Res<GamePlayer>) {
    update_and_save_player(&game_player.player, &mut players);
}

fn restart_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Gaming);
}

fn spawn_health_bar(
    builder: &mut ChildSpawnerCommands,
    health_bar: HealthBar,
    health: u16,
    span: u16
) {
    builder.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(95.),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(4.0)),
            display: Display::Grid,
            grid_column: GridPlacement::span(span),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::srgb_u8(107, 162, 215)),
    )).with_children(|builder| {
        builder.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(HEALTH_MAX_VALUE, 1.),
                grid_template_rows: vec![GridTrack::flex(1.)],
                ..default()
            },
            health_bar.clone()
        )).with_children(|builder| {
            let index_fn: fn(u16) -> u16 =
                if health_bar.role == GameRole::Player { |i| i } else { |i| HEALTH_MAX_VALUE - i  };
            for i in 0..HEALTH_MAX_VALUE {
                let color = if i < health {
                    gradient_health_bar_color(index_fn(i))
                } else {
                    Color::srgb_u8(70,70,70)
                };
                builder
                    .spawn((
                        Node::default(),
                        BackgroundColor(color),
                    ));
            }
        });
    });
}

pub fn gradient_health_bar_color(value: u16) -> Color {
    let red    = Color::srgb_u8(234, 67, 53).to_linear();
    let yellow = Color::srgb_u8(251, 210, 8).to_linear();
    let green  = Color::srgb_u8(52, 168, 82).to_linear();

    let c = if value <= 50 {
        let t = value as f32 / 50.0;
        red.lerp(yellow, t)
    } else {
        let t = (value as f32 - 50.0) / 49.0;
        yellow.lerp(green, t)
    };

    Color::LinearRgba(c)
}

#[derive(Component)]
struct SpaceStar {
    speed: f32,
    phase: f32,
    rate: f32,
}

fn spawn_space_stars(commands: &mut Commands, asset_server: &AssetServer, window: Single<&Window>) {
    let mut rng = rand::rng();
    let texture = asset_server.load("images/space_star.png");

    let layers = vec![
        (30, 15.0_f32..25.0_f32, 0.10_f32..0.20_f32, -1.0_f32),  // 远（慢）
        (35, 30.0_f32..45.0_f32, 0.15_f32..0.25_f32, -2.0_f32), // 中
        (40, 50.0_f32..75.0_f32, 0.20_f32..0.35_f32, -3.0_f32),// 近（快）
    ];

    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
    for (count, speed_range, scale_range, z) in layers {
        for _ in 0..count {
            let x = rng.random_range(-half_w..half_w);
            let y = rng.random_range(-half_h..half_h);
            let speed = rng.random_range(speed_range.clone());
            let scale = rng.random_range(scale_range.clone());

            // 随机初始化闪烁相位和速率
            let phase = rng.random_range(0.0..std::f32::consts::TAU);
            let rate = rng.random_range(1.0..3.0);

            commands.spawn((
                DespawnOnExit(GameState::Gaming),
                Sprite {
                    image: texture.clone(),
                    image_mode: SpriteImageMode::Auto,
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(Vec3::new(x, y, z)).with_scale(Vec3::splat(scale)),
                SpaceStar {
                    speed,
                    phase,
                    rate,
                },
            ));
        }
    }
}

pub fn update_game_time(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut GameTime)>,
) {
    if let Ok((mut text, mut clock)) = query.single_mut() {
        let elapsed = time.elapsed_secs_f64() - clock.start_time;
        let total_seconds = elapsed as u64;

        if total_seconds != clock.last_second {
            clock.last_second = total_seconds;
            *text = Text::new(format!("{:02}:{:02}:{:02}",
                                      total_seconds / 3600,
                                      (total_seconds % 3600) / 60,
                                      total_seconds % 60));
        }
    }
}

fn compute_route_count(window_height: f32) -> usize {
    let total_height = window_height - GAME_INFO_AREA_HEIGHT - GAME_INFO_AREA_MARGIN;
    let mut route_count = (total_height / DEFAULT_ROUTE_HEIGHT) as usize;
    if route_count > MAX_ROUTE_COUNT { MAX_ROUTE_COUNT } else { route_count }
}

fn on_window_resized(
    mut resize_events: MessageReader<WindowResized>,
    mut commands: Commands,
    mut game_routes: ResMut<GameRoutes>,
    mut game_player: ResMut<GamePlayer>,
    asset_server: Res<AssetServer>,
    stars: Query<Entity, With<SpaceStar>>,
    mut fighter_jet: Single<&mut Transform, With<FighterJet>>,
    window: Single<&Window>
) {
    if let Some(_) = resize_events.read().last() {
        // 调整玩家战斗机位置
        fighter_jet.translation.x = FIGHTER_JET_MARGIN - window.width()/2.;

        // 重新计算航道信息
        let route_count = compute_route_count(window.height());
        let last_route_count = game_routes.empty_routes.len() + game_routes.used_routes.len();
        if route_count > last_route_count {
            for i in 0..route_count-last_route_count {
                game_routes.empty_routes.push(Route {
                    id: (i+last_route_count) as i32,
                    entities: Vec::new(),
                })
            }
        }

        game_player.safe_position = -(window.width() / 2. - FIGHTER_JET_MARGIN - FIGHTER_JET_SIZE * FIGHTER_JET_SCALE - 100.);

        // 重新生成星空
        for entity in &stars {
            commands.entity(entity).despawn();
        }
        spawn_space_stars(&mut commands, &asset_server, window);
    }
}

fn move_space_stars(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &SpaceStar)>,
    window: Single<&Window>,
) {
    let left_bound = -window.width() / 2.0;
    let right_bound = window.width() / 2.0;

    for (mut transform, star) in &mut query {
        transform.translation.x -= star.speed * time.delta_secs();
        if transform.translation.x < left_bound {
            transform.translation.x = right_bound;
        }
    }
}

/// 星星闪烁
fn twinkle_space_stars(time: Res<Time>, mut query: Query<(&mut Sprite, &mut SpaceStar)>) {
    let dt = time.delta_secs();
    for (mut sprite, mut star) in &mut query {
        star.phase += star.rate * dt;
        let brightness = 0.5 + 0.5 * (star.phase.sin());
        sprite.color.set_alpha(brightness as f32);
    }
}

fn save_game_data (
    time: Res<Time>,
    mut timer: ResMut<GameSaveTimer>,
    mut players: ResMut<Players>,
    game_player: Res<GamePlayer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        update_and_save_player(&game_player.player, &mut players);
    }
}