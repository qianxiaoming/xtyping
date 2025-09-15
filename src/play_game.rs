use bevy::app::App;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use crate::{GameData, GameFonts, GameState, Players};
use crate::ui::*;

enum Character {
    Player,
    Enemy
}

#[derive(Component, Default)]
struct PlayGameEntity;

pub fn play_game_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::PlayGame), play_game_setup)
        .add_systems(OnExit(GameState::PlayGame), play_game_exit);
}

fn play_game_setup(mut commands: Commands,
                   players: Res<Players>,
                   game_data: Res<GameData>,
                   fonts: Res<GameFonts>,
                   asset_server: Res<AssetServer>) {
    commands.spawn((
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
                let player = players.get(&game_data.player);
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
                                    BorderRadius::MAX,
                                    BackgroundColor(Color::srgb_u8(70, 139, 254))
                                ));
                            });
                            let avatar = format!("avatars/{}.png", player.avatar.as_str());
                            spawn_image_node(builder, &asset_server, &avatar, Vec2::splat(64.), 0., 3.);
                        });
                    });
                // 用户名称
                spawn_info_text(builder, &game_data.player, INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
                // 用户等级
                spawn_image_node(builder, &asset_server, &format!("images/star-{}.png", player.level),
                                 Vec2::new(24.*(player.level as f32), 24.), 0., 0.);
                // 用户分数
                builder.spawn(Node {
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                }).with_children(|builder| {
                    spawn_info_text(builder, &format!("{}", player.score), INFO_TEXT_COLOR, fonts.ui_font.clone(), 28.);
                });
                // 玩家血条
                spawn_health_bar(builder, HealthBarMarker(Character::Player), 100, 3);
            });

        // 中间的对战图标
        builder.spawn(
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ).with_children(|builder| {
            spawn_image_node(builder, &asset_server, "images/vs.png", Vec2::splat(64.), 0., 0.);
        });

        // 右边的敌人及其它资源信息

    });
}

#[derive(Component)]
pub struct HealthBarMarker(Character);

const HEALTH_BAR_LEN: u16 = 100;

fn spawn_health_bar(
    builder: &mut ChildSpawnerCommands,
    health_bar: HealthBarMarker,
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
            match health_bar {
                HealthBarMarker(Character::Player) => {
                    for i in 0..HEALTH_BAR_LEN {
                        let color = if i < health {
                            gradient_health_bar_color(i)
                        } else {
                            Color::srgb_u8(70,70,70)
                        };
                        builder
                            .spawn((
                                Node::default(),
                                BackgroundColor(color),
                            ));
                    }
                },
                HealthBarMarker(Character::Enemy) => {

                }
            }
        });
    });
}

fn play_game_exit(mut commands: Commands, query: Query<Entity, With<PlayGameEntity>>) {
    cleanup_entities::<PlayGameEntity>(commands, query);
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