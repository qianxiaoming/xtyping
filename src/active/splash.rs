use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::prelude::*;
use crate::{GameFonts, PlayState};
use crate::active::common::SplashTextRow;
use crate::ui::{spawn_image_node, spawn_info_text};

pub fn game_splash_setup(mut commands: Commands,
                         asset_server: Res<AssetServer>,
                         fonts: Res<GameFonts>) {
    commands.spawn((
        DespawnOnExit(PlayState::Splash),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        }
    ))
        .with_children(|builder| {
            builder.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                SplashTextRow { timer: Timer::from_seconds(4.0, TimerMode::Once) }
            ))
                .with_children(|builder| {
                    spawn_info_text(builder, "Ready Go!", Color::srgba_u8(135, 201, 22, 0),
                                    fonts.normal_font.clone(), 48.);
                });

            let text_color = Color::srgba_u8(188, 190, 196, 0);
            let key_color = Color::srgba_u8(255, 100, 100, 0);
            builder.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                SplashTextRow { timer: Timer::from_seconds(4.0, TimerMode::Once) }
            ))
                .with_children(|builder| {
                    spawn_image_node(builder, &asset_server, "images/esc.png", Vec2::splat(50.0), 5., 0.);
                    spawn_info_text(builder, "按下", text_color, fonts.ui_font.clone(), 32.);
                    spawn_info_text(builder, "ESC键", key_color, fonts.ui_font.clone(), 32.);
                    spawn_info_text(builder, "可以退出游戏", text_color, fonts.ui_font.clone(), 32.);
                });

            builder.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                SplashTextRow { timer: Timer::from_seconds(4.0, TimerMode::Once) }
            ))
                .with_children(|builder| {
                    spawn_image_node(builder, &asset_server, "images/space.png", Vec2::splat(50.0), 5., 0.);
                    spawn_info_text(builder, "按下", text_color, fonts.ui_font.clone(), 32.);
                    spawn_info_text(builder, "空格键", key_color, fonts.ui_font.clone(), 32.);
                    spawn_info_text(builder, "暂停继续游戏", text_color, fonts.ui_font.clone(), 32.);
                });
        });
}

pub fn fade_tip_messages(
    time: Res<Time>,
    mut query: Query<(&mut SplashTextRow, &Children)>,
    mut text_query: Query<&mut TextColor>,
    mut image_query: Query<&mut ImageNode>,
    mut next_state: ResMut<NextState<PlayState>>
) {
    for (mut fade, children) in &mut query {
        fade.timer.tick(time.delta());
        let t = fade.timer.elapsed_secs();
        let (alpha, finished) = if t < 1. {
            // 淡入
            (t / 1., false)
        } else if t < 3. {
            // 保持
            (1.0, false)
        } else if t < 4.0 {
            // 淡出
            (1.0 - (t - 3.) / 1., false)
        } else {
            (0.0, true)
        };

        for child in children.iter() {
            if let Ok(mut color) = text_query.get_mut(child) {
                color.set_alpha(alpha);
            }
            if let Ok(mut img) = image_query.get_mut(child) {
                img.color.set_alpha(alpha);
            }
        }

        if finished {
            next_state.set(PlayState::Playing);
        }
    }
}