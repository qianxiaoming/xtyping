mod marker;
mod playing;
mod splash;

use rand::Rng;
use bevy::app::App;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::window::WindowResized;
use crate::{GameData, GameFonts, GameState, PlayState};
use crate::ui::*;
use marker::*;

enum Character {
    Player,
    Enemy
}

#[derive(Component)]
struct PlayingEntity;

pub fn play_game_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Playing), playing_game_setup)
        .add_systems(OnEnter(PlayState::Splash), splash::game_splash_setup)
        .add_systems(OnEnter(PlayState::Playing), playing::game_player_setup)
        .add_systems(Update, update_game_time)
        .add_systems(Update, on_window_resized.run_if(on_message::<WindowResized>
            .and(in_state(GameState::Playing))))
        .add_systems(Update, (move_space_stars, twinkle_space_stars).run_if(in_state(GameState::Playing)))
        .add_systems(Update, splash::fade_tip_messages.run_if(in_state(PlayState::Splash)));
}

fn playing_game_setup(mut commands: Commands, 
                      game_data: Res<GameData>, 
                      fonts: Res<GameFonts>, 
                      asset_server: Res<AssetServer>, 
                      time: Res<Time>, 
                      window: Single<&Window>, 
                      mut next_state: ResMut<NextState<PlayState>>) {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Node {
            width: Val::Percent(100.),
            height: Val::Px(70.0),
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
        BackgroundColor(Color::NONE),
        PlayingEntity
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
                let player = &game_data.player;
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
                                        width: Val::Percent(50.),
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
                spawn_info_text(builder, &game_data.player.name, INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
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
                spawn_health_bar(builder, HealthBar(Character::Player), 100, 3);
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
                spawn_marked_text(builder, EnemyCounter, "0/0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            builder.spawn(Node::default()).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/bomb.png", Vec2::splat(30.), 2., 4.);
                spawn_marked_text(builder, BombCounter, "0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            builder.spawn(Node::default()).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/first-aid-kit.png", Vec2::splat(30.), 2., 4.);
                spawn_marked_text(builder, BloodBagCounter, "0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            builder.spawn(Node::default()).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/shield.png", Vec2::splat(30.), 2., 4.);
                spawn_marked_text(builder, ShieldCounter, "0", INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
            });
            // 敌方血条
            spawn_health_bar(builder, HealthBar(Character::Enemy), 100, 4);
        });
    });

    spawn_space_stars(&mut commands, &asset_server, window);
    next_state.set(PlayState::Splash);
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
        builder.spawn(
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(HEALTH_BAR_LEN, 1.),
                grid_template_rows: vec![GridTrack::flex(1.)],
                ..default()
            }
        ).with_children(|builder| {
            let index_fn: fn(u16) -> u16 = match health_bar {
                HealthBar(Character::Player) => |i| i,
                HealthBar(Character::Enemy) => |i| HEALTH_BAR_LEN - i,
            };
            for i in 0..HEALTH_BAR_LEN {
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

            let h = total_seconds / 3600;
            let m = (total_seconds % 3600) / 60;
            let s = total_seconds % 60;

            *text = Text::new(format!("{:02}:{:02}:{:02}", h, m, s));
        }
    }
}

fn on_window_resized(
    mut resize_events: MessageReader<WindowResized>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stars: Query<Entity, With<SpaceStar>>,
    window: Single<&Window>
) {
    if let Some(e) = resize_events.read().last() {
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
