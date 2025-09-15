use bevy::color::Color;
use bevy::prelude::*;
use crate::{GameFonts, GAME_APP_TITLE};

pub fn cleanup_entities<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub const TITLE_FONT_SIZE: f32 = 60.0;
pub const NORMAL_FONT_SIZE: f32 = 20.0;
pub const INFO_FONT_SIZE: f32 = 18.0;
pub const INFO_TEXT_COLOR: Color = Color::srgb_u8(188, 190, 196);

pub fn spawn_startup_root<'a, T: Component + Default>(commands: &'a mut Commands) -> EntityCommands<'a> {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        T::default()
    ))
}

pub fn spawn_game_title(builder: &mut ChildSpawnerCommands,
                        fonts: &GameFonts,
                        scale: f32,
                        margin: f32,
                        padding: f32,
                        spacing: f32, 
                        signature: bool) {
    let colors = vec![Color::srgb_u8(66, 133, 243),
                      Color::srgb_u8(234, 67, 53),
                      Color::srgb_u8(251, 188, 8),
                      Color::srgb_u8(66, 133, 243),
                      Color::srgb_u8(52, 168, 82),
                      Color::srgb_u8(234, 67, 53)];
    builder.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(TITLE_FONT_SIZE*scale + padding*2.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            margin: UiRect::top(Val::Px(margin)),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
        .with_children(|builder| {
            builder
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
                                font: fonts.title_font.clone(),
                                font_size: TITLE_FONT_SIZE*scale,
                                ..default()
                            },
                            TextColor(color),
                            Node {
                                margin: UiRect::right(Val::Px(spacing)),
                                ..default()
                            }
                        ));
                    }
                });
        });
    if signature {
        builder.spawn((
            Text::new("v0.1  kyleqian@gmail.com"),
            TextFont {
                font: fonts.info_font.clone(),
                font_size: INFO_FONT_SIZE,
                ..default()
            },
            TextColor(INFO_TEXT_COLOR),
        ));
    }
}

pub fn spawn_image_node(builder: &mut ChildSpawnerCommands,
                    asset_server: &AssetServer,
                    path: &str,
                    size: Vec2,
                    h_margin: f32,
                    v_margin: f32) {
    builder.spawn((
        ImageNode::new(asset_server.load(path.to_string())),
        Node {
            width: Val::Px(size.x),
            height: Val::Px(size.y),
            margin: UiRect {
                left: Val::Px(h_margin),
                right: Val::Px(h_margin),
                top: Val::Px(v_margin),
                bottom: Val::Px(v_margin),
            },
            ..default()
        },
    ));
}

pub fn spawn_marked_image<Marker: Component>(
    builder: &mut ChildSpawnerCommands,
    marker: Marker,
    asset_server: &AssetServer,
    path: &str,
    size: Vec2,
    h_margin: f32,
    v_margin: f32) {
    builder.spawn((
        ImageNode::new(asset_server.load(path.to_string())),
        marker,
        Node {
            width: Val::Px(size.x),
            height: Val::Px(size.y),
            margin: UiRect {
                left: Val::Px(h_margin),
                right: Val::Px(h_margin),
                top: Val::Px(v_margin),
                bottom: Val::Px(v_margin),
            },
            ..default()
        },
    ));
}

pub fn spawn_info_text(builder: &mut ChildSpawnerCommands,
                   text: &str,
                   color: Color,
                   font: Handle<Font>,
                   font_size: f32) {
    builder.spawn((
        Text::new(text),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(color)
    ));
}

pub fn spawn_marked_text<Marker: Component>(
    builder: &mut ChildSpawnerCommands,
    marker: Marker,
    text: &str,
    color: Color,
    font: Handle<Font>,
    font_size: f32) {
    builder.spawn((
        Text::new(text),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(color),
        marker
    ));
}

pub fn spawn_instructions(builder: &mut ChildSpawnerCommands, text: &str, fonts: &GameFonts, margin: f32) {
    builder.spawn((
        Text::new(text),
        TextFont {
            font: fonts.normal_font.clone(),
            font_size: NORMAL_FONT_SIZE,
            ..default()
        },
        TextColor(Color::srgb_u8(96, 211, 255)),
        Node {
            margin: UiRect::top(Val::Px(margin)),
            ..default()
        }
    ));
}