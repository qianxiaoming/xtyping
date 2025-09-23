use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::{widgets, GameFonts, GamePlayer, GameState, PlayState};
use crate::gaming::common::LastPlayState;
use crate::ui::{spawn_image_node, spawn_info_text};
use crate::widgets::ModelDialog;

#[derive(Component)]
pub struct ButtonExitGame;

#[derive(Component)]
pub struct ButtonCancel;

pub fn confirm_exit_setup(mut commands: Commands,
                          game_player: Res<GamePlayer>,
                          game_fonts: Res<GameFonts>,
                          asset_server: Res<AssetServer>) {
    let dialog = ModelDialog::new(&mut commands, PlayState::Exiting, 60.);
    commands.entity(dialog.container).with_children(|builder| {
        spawn_info_text(builder, "Confirm Exit", Color::srgb_u8(135, 201, 22),
                        game_fonts.normal_font.clone(), 48.);
        builder.spawn(
            Node {
                width: Val::Percent(90.),
                height: Val::Auto,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            }).with_children(|builder| {
            spawn_image_node(builder, &asset_server, "images/grimacing.png", Vec2::splat(96.0), 30., 0.);
            spawn_info_text(builder, &format!("{}，确定要退出游戏吗？", game_player.player.name),
                            Color::srgb_u8(188, 190, 196), game_fonts.ui_font.clone(), 28.);
        });
        builder.spawn(
            Node {
                width: Val::Percent(50.),
                height: Val::Auto,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            }).with_children(|builder| {
            builder.spawn(
                widgets::PushButton::new(ButtonExitGame,
                                         "退出",
                                         Vec2::new(80.0,40.0),
                                         true,
                                         UiRect::right(Val::Px(10.0))
                ));
            builder.spawn(
                widgets::PushButton::new(ButtonCancel,
                                         "取消",
                                         Vec2::new(80.0,40.0),
                                         true,
                                         UiRect::left(Val::Px(10.0))
                ));
        });
    });

    commands.insert_resource(LastPlayState(PlayState::Exiting));
}

pub fn on_cancel_exit_button(
    mut next_state: ResMut<NextState<PlayState>>,
    mut reader: MessageReader<widgets::ButtonClicked>,
    query: Query<(), With<ButtonCancel>>,
) {
    if let Some(event) = reader.read().last()
        && query.get(event.entity).is_ok() {
        next_state.set(PlayState::Playing);
    }
}

pub fn on_exit_game_button(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut reader: MessageReader<widgets::ButtonClicked>,
    query: Query<(), With<ButtonExitGame>>,
) {
    if let Some(event) = reader.read().last()
        && query.get(event.entity).is_ok()  {
        commands.remove_resource::<LastPlayState>();
        next_state.set(GameState::Startup);
    }
}