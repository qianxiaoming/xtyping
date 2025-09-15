use bevy::prelude::*;
use bevy::app::AppExit;
use super::*;
use ui::*;
use widgets;
use crate::widgets::{ListItem, ListView, TextConfig};

const MAX_PLAYERS_COUNT: usize = 7;

pub fn startup_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Startup), startup_setup)
        .add_systems(OnExit(GameState::Startup), cleanup_entities::<StartupEntity>)
        .add_systems(Update, on_create_user_button.run_if(in_state(GameState::Startup)))
        .add_systems(Update, on_exit_game_button.run_if(in_state(GameState::Startup)))
        .add_systems(Update, on_player_selected.run_if(in_state(GameState::Startup)));
}

#[derive(Component, Default)]
struct StartupEntity;

#[derive(Component)]
struct ListViewPlayer;

#[derive(Component)]
struct ButtonCreateUser;

#[derive(Component)]
struct ButtonExitGame;

fn startup_setup(mut commands: Commands, players: Res<Players>, fonts: Res<GameFonts>, asset_server: Res<AssetServer>) {
    spawn_startup_root::<StartupEntity>(&mut commands)
        .with_children(|builder| {
            spawn_game_title(builder, &fonts, 1., 20., 15., 20., true);
            if players.0.is_empty() {
                default_screen_setup(builder, fonts, asset_server);
            } else {
                player_list_setup(builder, players, fonts, asset_server);
            }
        });
}

fn default_screen_setup(builder: &mut ChildSpawnerCommands, fonts: Res<GameFonts>, asset_server: Res<AssetServer>) {
    // 首次开始游戏时，提示建立自己的账户
    spawn_instructions(builder, "为了开始游戏，首先需要创建一个自己的账户。", &fonts, 80.0);
    // 创建新账户按钮
    builder.spawn(
        widgets::PushButton::new(ButtonCreateUser,
                                 "创建新的账号",
                                 Vec2::new(500.0,50.0),
                                 true,
                                 UiRect::top(Val::Px(30.0))
        ));
    // 退出游戏按钮
    builder.spawn(
        widgets::PushButton::new(ButtonExitGame,
                                 "退出游戏",
                                 Vec2::new(500.0,50.0),
                                 true,
                                 UiRect::top(Val::Px(20.0))
        ));
    // 快速入门说明
    builder.spawn((
        Node {
            display: Display::Grid,
            width: Val::Percent(60.0),
            height: Val::Auto,
            margin: UiRect::top(Val::Px(50.0)),
            grid_template_columns: vec![GridTrack::flex(1.0)],
            grid_template_rows: vec![
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto(),
                GridTrack::auto()
            ],
            ..default()
        },
        BackgroundColor(Color::NONE),
    )).with_children(|builder| {
        builder.spawn(
            Node {
                display: Display::Grid,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|builder| {
            spawn_info_text(builder, " 游 戏 快 速 入 门 ", INFO_TEXT_COLOR,
                            fonts.normal_font.clone(), INFO_FONT_SIZE + 4.0);
        });
        builder.spawn((
            Node {
                display: Display::Grid,
                width: Val::Percent(100.0),
                height: Val::Px(1.0),
                ..default()
            },
            BackgroundColor(Color::WHITE)
        ));
        builder.spawn(
            Node {
                display: Display::Grid,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            }).with_children(|builder| {
            spawn_info_text(builder, "超级简单！攻击你的小飞机带有字母或符号，按对应的按键就能消灭它！游戏中你会遇到：",
                            INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE);
        });
        builder.spawn(
            Node {
                display: Display::Grid,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|builder| {
            builder.spawn(
                Node {
                    display: Display::Grid,
                    width: Val::Percent(85.0),
                    height: Val::Auto,
                    margin: UiRect::left(Val::Px(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    grid_template_columns: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
                    grid_template_rows: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
                    ..default()
                }
            ).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/plane_3_yellow.png", Vec2::splat(60.), 10.0, 5.0);
                spawn_item_desc_node(builder)
                    .with_children(|builder| {
                        spawn_info_text(builder, "敌机", Color::srgb_u8(251, 188, 8),
                                        fonts.title_font.clone(), INFO_FONT_SIZE+2.0);
                        spawn_info_text(builder, "小飞机接近后就会发射飞弹攻击你，按下对应的字母击落它！",
                                        INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE-2.0);
                    });

                spawn_image_node(builder, &asset_server, "images/bomb.png", Vec2::splat(45.), 10.0, 10.0);
                spawn_item_desc_node(builder)
                    .with_children(|builder| {
                        spawn_info_text(builder, "炸弹", Color::srgb_u8(234, 67, 53),
                                        fonts.title_font.clone(), INFO_FONT_SIZE+2.0);
                        spawn_info_text(builder, "炸弹可以将当前所有的敌机一次性全部摧毁，相当于你的大招！",
                                        INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE-2.0);
                    });

                spawn_image_node(builder, &asset_server, "images/first-aid-kit.png", Vec2::splat(45.), 10.0, 10.0);
                spawn_item_desc_node(builder)
                    .with_children(|builder| {
                        spawn_info_text(builder, "急救包", Color::srgb_u8(52, 168, 82),
                                        fonts.title_font.clone(), INFO_FONT_SIZE+2.0);
                        spawn_info_text(builder, "急救包可以快速给你补血，提高生命值，在战斗中坚持的更久！",
                                        INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE-2.0);
                    });

                spawn_image_node(builder, &asset_server, "images/shield.png", Vec2::splat(45.), 10.0, 10.0);
                spawn_item_desc_node(builder)
                    .with_children(|builder| {
                        spawn_info_text(builder, "护盾", Color::srgb_u8(66, 133, 243),
                                        fonts.title_font.clone(), INFO_FONT_SIZE+2.0);
                        spawn_info_text(builder, "护盾赋予你长达15秒坚不可摧的保护，期间所有的攻击对你无效。",
                                        INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE - 2.0);
                    });
            });
        });
    });
}

fn player_list_setup(builder: &mut ChildSpawnerCommands, players: Res<Players>, fonts: Res<GameFonts>, asset_server: Res<AssetServer>) {
    spawn_instructions(builder, "欢迎回来，选择你的账户以继续游戏", &fonts, 80.0);
    builder.spawn((
        Node {
            width: Val::Px(700.),
            height: Val::Px(300.),
            border: UiRect::all(Val::Px(2.)),
            margin: UiRect::top(Val::Px(15.)),
            padding: UiRect::all(Val::Px(20.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Start,
            ..default()
        },
        BorderColor::all(Color::srgb_u8(76, 69, 113)),
        BorderRadius::all(Val::Px(8.0)),
        BackgroundColor(Color::NONE),
    )).with_children(|builder| {
        let icon_size = NORMAL_FONT_SIZE + 8.;
        let list = ListView::new(builder,
                                 ListViewPlayer,
                                 TextConfig {
                                     text: "players".to_owned(),
                                     font: fonts.ui_font.clone(),
                                     font_size: NORMAL_FONT_SIZE + 4.,
                                     color: Color::srgb_u8(188, 190, 196),
                                     shadow: false
                                 },
                                 vec![GridTrack::min_content(),
                                      GridTrack::flex(3.0),
                                      GridTrack::flex(2.0),
                                      GridTrack::flex(1.0)],
                                 RepeatedGridTrack::flex(players.0.len() as u16, 1.0),
                                 Some((asset_server.load("images/fighter-jet.png"), Vec2::splat(icon_size))),
                                 None,
                                 None);
        for player in &players.0 {
            list.append(builder.commands_mut(),
                        player.name.clone(),
                        vec![
                            ListItem::Image(asset_server.load(format!("avatars/{}.png", player.avatar)),
                                            Vec2::splat(icon_size)),
                            ListItem::Text(player.name.clone()),
                            ListItem::Text(format!("{}分", player.score)),
                            ListItem::Image(asset_server.load(format!("images/star-{}.png", player.level)),
                                            Vec2::new((icon_size-4.)*(player.level as f32), icon_size-4.)),
                        ]);
        }
    });
    builder.spawn((
        Node {
            display: Display::Grid,
            width: Val::Px(700.0),
            height: Val::Auto,
            grid_template_columns: vec![GridTrack::px(300.0), GridTrack::auto(), GridTrack::px(300.0)],
            grid_template_rows: vec![GridTrack::auto()],
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
    )).with_children(|builder| {
        builder.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Px(1.),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(76, 69, 113)),
            ));
        spawn_instructions(builder, "或者", &fonts, 0.0);
        builder.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Px(1.),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(76, 69, 113)),
        ));
    });
    info!("total players: {}", players.0.len());
    builder.spawn(
        widgets::PushButton::new(ButtonCreateUser,
                                 "创建新的账号",
                                 Vec2::new(500.0,50.0),
                                 players.0.len() < MAX_PLAYERS_COUNT,
                                 UiRect::top(Val::Px(20.0))
        ));
    builder.spawn(
        widgets::PushButton::new(ButtonExitGame,
                                 "退出游戏",
                                 Vec2::new(500.0,50.0),
                                 true,
                                 UiRect::top(Val::Px(20.0))
        ));
}

fn spawn_item_desc_node<'a>(builder: &'a mut ChildSpawnerCommands) -> EntityCommands<'a> {
    builder.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
}

fn on_create_user_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut reader: EventReader<widgets::ButtonClicked>,
    query: Query<(), With<ButtonCreateUser>>,
) {
    if let Some(event) = reader.read().last()
        && query.get(event.entity).is_ok() {
        next_state.set(GameState::NewPlayer);
    }
}

fn on_exit_game_button(
    mut reader: EventReader<widgets::ButtonClicked>,
    mut exit: EventWriter<AppExit>,
    query: Query<(), With<ButtonExitGame>>,
) {
    if let Some(event) = reader.read().last()
        && query.get(event.entity).is_ok()  {
        exit.write(AppExit::Success);
    }
}

fn on_player_selected(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut reader: EventReader<widgets::ListViewSelectionChanged>,
    query: Query<(), With<ListViewPlayer>>,
) {
    if let Some(event) = reader.read().last()
        && query.get(event.entity).is_ok() {
        info!("Player {} selected to continue game", event.value);
        commands.insert_resource(GameData {
            player: event.value.clone(),
            ..default()
        });
        next_state.set(GameState::PlayGame)
    }
}
