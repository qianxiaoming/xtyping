use bevy::prelude::*;
use bevy::app::AppExit;
use super::*;
use ui::*;
use widgets;

pub fn startup_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Startup), startup_setup)
        .add_systems(OnExit(GameState::Startup), cleanup_entities::<StartupEntity>)
        .add_systems(Update, on_create_user_button.run_if(in_state(GameState::Startup)))
        .add_systems(Update, on_exit_game_button.run_if(in_state(GameState::Startup)));
}

#[derive(Component, Default)]
struct StartupEntity;

#[derive(Component)]
struct ButtonCreateUser;

#[derive(Component)]
struct ButtonExitGame;

fn startup_setup(mut commands: Commands, players: Res<Players>, fonts: Res<GameFonts>, asset_server: Res<AssetServer>) {
    spawn_startup_root::<StartupEntity>(&mut commands)
        .with_children(|parent| {
            spawn_game_title(parent, &fonts);
            if players.0.len() == 0 {
                // 首次开始游戏时，提示建立自己的账户。
                spawn_instructions(parent, "为了开始游戏，首先需要创建一个自己的账户。", &fonts, 100.0);
                // 创建新账户按钮
                parent.spawn(
                    widgets::PushButton::new(ButtonCreateUser,
                                            "创建新的账号",
                                            Vec2::new(500.0,50.0),
                                            UiRect::top(Val::Px(30.0))
                    ));
                // 退出游戏按钮
                parent.spawn(
                    widgets::PushButton::new(ButtonExitGame,
                                            "退出游戏",
                                            Vec2::new(500.0,50.0),
                                            UiRect::top(Val::Px(20.0))
                    ));
                // 快速入门说明
                parent.spawn((
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
                            spawn_image_node(builder, &asset_server, "images/plane_3_yellow.png", 60.0, 10.0, 5.0);
                            spawn_item_desc_node(builder)
                                .with_children(|builder| {
                                    spawn_info_text(builder, "敌机", Color::srgb_u8(251, 188, 8),
                                                    fonts.title_font.clone(), INFO_FONT_SIZE+2.0);
                                    spawn_info_text(builder, "小飞机接近后就会发射飞弹攻击你，按下对应的字母击落它！",
                                                    INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE-2.0);
                                });

                            spawn_image_node(builder, &asset_server, "images/bomb.png", 45.0, 10.0, 10.0);
                            spawn_item_desc_node(builder)
                                .with_children(|builder| {
                                    spawn_info_text(builder, "炸弹", Color::srgb_u8(234, 67, 53),
                                                    fonts.title_font.clone(), INFO_FONT_SIZE+2.0);
                                    spawn_info_text(builder, "炸弹可以将当前所有的敌机一次性全部摧毁，相当于你的大招！",
                                                    INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE-2.0);
                                });

                            spawn_image_node(builder, &asset_server, "images/first-aid-kit.png", 45.0, 10.0, 10.0);
                            spawn_item_desc_node(builder)
                                .with_children(|builder| {
                                    spawn_info_text(builder, "急救包", Color::srgb_u8(52, 168, 82),
                                                    fonts.title_font.clone(), INFO_FONT_SIZE+2.0);
                                    spawn_info_text(builder, "急救包可以快速给你补血，提高生命值，在战斗中坚持的更久！",
                                                    INFO_TEXT_COLOR, fonts.normal_font.clone(), INFO_FONT_SIZE-2.0);
                                });

                            spawn_image_node(builder, &asset_server, "images/shield.png", 45.0, 10.0, 10.0);
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
        });
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
    for event in reader.read() {
        if query.get(event.entity).is_ok() {
            next_state.set(GameState::NewPlayer)
        }
    }
}

fn on_exit_game_button(
    mut reader: EventReader<widgets::ButtonClicked>,
    mut exit: EventWriter<AppExit>,
    query: Query<(), With<ButtonExitGame>>,
) {
    for event in reader.read() {
        if query.get(event.entity).is_ok() {
            exit.write(AppExit::Success);
        }
    }
}
