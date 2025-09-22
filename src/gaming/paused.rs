use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::{GameFonts, GamePlayer, PlayState};
use crate::gaming::common::LastPlayState;
use crate::ui::{spawn_image_node, spawn_info_text};

pub fn paused_setup(mut commands: Commands,
                    game_player: Res<GamePlayer>,
                    game_fonts: Res<GameFonts>,
                    asset_server: Res<AssetServer>) {
    commands.spawn((
        DespawnOnExit(PlayState::Paused),
        Node {
            width: Val::Percent(60.),
            height: Val::Auto,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            border: UiRect::all(Val::Px(2.)),
            ..default()
        },
       BorderColor::all(Color::srgb_u8(43, 44, 47)),
       BorderRadius::all(Val::Px(5.0)),
       BackgroundColor(Color::NONE),
    )).with_children(|builder| {
        builder.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                border: UiRect::all(Val::Px(3.)),
                padding: UiRect::all(Val::Px(10.)),
                ..default()
            },
            BorderColor::all(Color::srgb_u8(76, 69, 113)),
            BorderRadius::all(Val::Px(5.0)),
            BackgroundColor(Color::srgb_u8(43, 44, 47)),
        )).with_children(|builder| {
            spawn_info_text(builder, "Game Paused", Color::srgb_u8(135, 201, 22),
                            game_fonts.normal_font.clone(), 48.);
            builder.spawn(
                Node {
                    width: Val::Percent(90.),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    margin: UiRect::all(Val::Px(30.0)),
                    ..default()
                }).with_children(|builder| {
                spawn_image_node(builder, &asset_server, "images/space.png", Vec2::splat(96.0), 30., 0.);
                spawn_info_text(builder, &format!("{}，按下空格键可以继续游戏哟！", game_player.player.name),
                                Color::srgb_u8(188, 190, 196), game_fonts.ui_font.clone(), 28.);
            });
        });
    });

    commands.insert_resource(LastPlayState(PlayState::Paused));
}

pub fn on_resume_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<PlayState>>
) {
    if keyboard_input.just_released(KeyCode::Space) {
        next_state.set(PlayState::Playing);
    }
}