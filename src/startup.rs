use bevy::{
    app::AppExit,
    prelude::*,
};
use super::*;

pub fn startup_plugin(app: &mut App) {
    app
        .insert_resource(Players::default())
        .add_systems(OnEnter(GameState::Startup), startup_setup);
}

const GAME_TITLE_SIZE: f32 = 60.0;

fn startup_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建游戏标题
    let colors = vec![Color::srgb_u8(66, 133, 243),
                      Color::srgb_u8(234, 67, 53),
                      Color::srgb_u8(251, 188, 8),
                      Color::srgb_u8(66, 133, 243),
                      Color::srgb_u8(52, 168, 82),
                      Color::srgb_u8(234, 67, 53)];
    let font: Handle<Font> = asset_server.load("fonts/SimHei.ttf");
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(GAME_TITLE_SIZE + 30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            padding: UiRect::top(Val::Px(50.0)),
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
                            Text::new(format!("{} ", ch)),
                            TextFont {
                                font: font.clone(),
                                font_size: GAME_TITLE_SIZE,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                });
        });
}