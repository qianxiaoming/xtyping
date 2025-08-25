use bevy::{
    app::AppExit,
    prelude::*,
};
use bevy::ecs::relationship::RelatedSpawnerCommands;
use super::*;

pub fn startup_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Startup), startup_setup)
        .add_systems(Update, expand_title_line.run_if(in_state(GameState::Startup)));
}

const TITLE_FONT_SIZE: f32 = 60.0;
const NORMAL_FONT_SIZE : f32 = 20.0;

#[derive(Component)]
struct ExpandingTitleLine;

#[derive(Component)]
struct CreateUserTip;

fn startup_setup(mut commands: Commands, players: Res<Players>, fonts: Res<GameFonts>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
        .with_children(|parent| {
            spawn_game_title(parent, fonts.title_font.clone());
            parent.spawn((
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Px(2.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                ExpandingTitleLine,
            ));
            if players.0.len() == 0 {
                parent.spawn((
                    Text::new("为了开始游戏，首先需要创建一个自己的账户。"),
                    TextFont {
                        font: fonts.normal_font.clone(),
                        font_size: NORMAL_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(96, 211, 255)),
                    CreateUserTip,
                    Node {
                        margin: UiRect::top(Val::Px(100.0)),
                        ..default()
                    }
                ));
            }
        });
}

fn spawn_game_title(parent: &mut RelatedSpawnerCommands<ChildOf>, font: Handle<Font>) {
    let colors = vec![Color::srgb_u8(66, 133, 243),
                      Color::srgb_u8(234, 67, 53),
                      Color::srgb_u8(251, 188, 8),
                      Color::srgb_u8(66, 133, 243),
                      Color::srgb_u8(52, 168, 82),
                      Color::srgb_u8(234, 67, 53)];
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(TITLE_FONT_SIZE + 30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect::top(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|row| {
                    for (i, ch) in GAME_APP_TITLE.chars().enumerate() {
                        let color = colors[i % colors.len()];
                        row.spawn((
                            Text::new(ch.to_string()),
                            TextFont {
                                font: font.clone(),
                                font_size: TITLE_FONT_SIZE,
                                ..default()
                            },
                            TextColor(color),
                            Node {
                                margin: UiRect::right(Val::Px(20.0)),
                                ..default()
                            }
                        ));
                    }
                });
        });
}

fn expand_title_line(time: Res<Time>, mut query: Query<&mut Node, With<ExpandingTitleLine>>) {
    for mut node in &mut query {
        if let Val::Percent(w) = node.width {
            if w < 100.0 {
                let new_width = (w + time.delta_secs() * 100.0).min(100.0);
                node.width = Val::Percent(new_width);
            }
        }
    }
}