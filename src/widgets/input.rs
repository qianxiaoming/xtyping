use std::mem;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use super::TextConfig;

#[derive(Resource)]
pub struct InputFocused(Entity);

#[derive(Component)]
pub struct InputBoxBorder(Vec2);

#[derive(Component)]
pub struct InputBox {
    pub text_color: Color,
    pub hint_text: String,
    pub value: String,
    pub cursor: Entity
}

const INPUT_BOX_SIDE_COLOR: Color = Color::srgb_u8(240, 240, 240);

const INPUT_BOX_BOTTOM_COLOR: Color = Color::srgb_u8(0, 105, 186);

const HINT_TEXT_COLOR: Color = Color::srgb_u8(90, 90, 90);

#[derive(Component)]
pub struct CursorMarker {
    timer: Timer
}

impl InputBox {
    pub fn new<C: Component>(builder: &mut ChildSpawnerCommands,
                               marker: C,
                               text: TextConfig,
                               hint: &str,
                               size: Vec2,
                               margin: UiRect) {
        let use_hint = text.text.is_empty();
        builder.spawn((
            // 构建底边蓝色，其它边白色的文本输入框外观
            Node {
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                margin,
                padding: UiRect::left(Val::Px(6.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                border: UiRect {
                    left: Val::Px(0.5),
                    right: Val::Px(0.5),
                    top: Val::Px(0.5),
                    bottom: Val::Px(3.)
                },
                ..default()
            },
            InputBoxBorder(size),
            BorderRadius::px(5.0, 5.0, 5.0, 5.0),
            BorderColor {
                top: INPUT_BOX_SIDE_COLOR,
                left: INPUT_BOX_SIDE_COLOR,
                right: INPUT_BOX_SIDE_COLOR,
                bottom: INPUT_BOX_BOTTOM_COLOR,
            })
        ).with_children(|builder| {
            builder.spawn(
                Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                }).with_children(|builder| {
                let input_text = builder.spawn((
                    Text::new(if use_hint { hint.to_owned() } else { text.text.clone() }),
                    TextFont {
                        font: text.font.clone(),
                        font_size: text.font_size,
                        ..default()
                    },
                    TextColor(if use_hint { HINT_TEXT_COLOR } else { text.color }),
                    text.to_shadow(),
                    marker
                )).id();
                let cursor = builder.spawn((
                    Node {
                        width: Val::Px(2.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    Visibility::Hidden,
                    BackgroundColor(INPUT_BOX_SIDE_COLOR),
                    CursorMarker {
                        timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                    }
                )).id();
                builder.commands_mut().entity(input_text).insert(InputBox {
                    text_color: text.color,
                    hint_text: hint.to_owned(),
                    value: "".to_owned(),
                    cursor
                });
            });
        });
    }
}

fn get_border_position(entity: Entity,
                       query_parent: Query<&ChildOf>,
                       input_border_query: Query<(&UiGlobalTransform, &InputBoxBorder), With<InputBoxBorder>>) -> (Vec2, Vec2) {
    let border_entity = query_parent.get(entity).ok().and_then(|p| query_parent.get(p.0).ok()).map(|p| p.0);
    let (transform, border) = input_border_query.get(border_entity.unwrap()).unwrap();
    (transform.translation, border.0)
}

fn is_cursor_in_border(cursor_pos: Vec2,
                       factor: f32,
                       entity: Entity,
                       query_parent: Query<&ChildOf>,
                       input_border_query: Query<(&UiGlobalTransform, &InputBoxBorder), With<InputBoxBorder>>) -> bool {
    let (translation, border_size) = get_border_position(entity, query_parent, input_border_query);
    let half_size = border_size / 2.0;
    let translation = translation / factor;
    cursor_pos.x >= translation.x - half_size.x &&
        cursor_pos.x <= translation.x + half_size.x &&
        cursor_pos.y >= translation.y - half_size.y &&
        cursor_pos.y <= translation.y + half_size.y
}

pub fn input_box_handle_focus(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    query_parent: Query<&ChildOf>,
    input_border_query: Query<(&UiGlobalTransform, &InputBoxBorder), With<InputBoxBorder>>,
    mut input_box_query: Query<
        (
            Entity,
            &mut Text,
            &mut TextColor,
            &InputBox
        ),
        With<InputBox>
    >,
    mut cursor_query: Query<&mut Visibility, With<CursorMarker>>,
    input_focused: Option<Res<InputFocused>>,
    mut window: Single<&mut Window>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let cursor_pos = window.cursor_position().unwrap_or_default();
        for (entity, mut text, mut color,  input_box) in &mut input_box_query {
            if is_cursor_in_border(cursor_pos, window.scale_factor(), entity, query_parent, input_border_query) {
                // 如果本身就在输入状态则什么也不做
                if input_focused.is_none() || input_focused.as_ref().unwrap().0 != entity {
                    *text = Text::new(input_box.value.clone());
                    *color = TextColor(input_box.text_color);
                    let mut visibility = cursor_query.get_mut(input_box.cursor).unwrap();
                    *visibility = Visibility::Visible;
                    commands.insert_resource(InputFocused(entity));

                    // toggle IME
                    window.ime_position = window.cursor_position().unwrap();
                    window.ime_enabled = true;
                }
            } else {
                if text.0.is_empty() {
                    *text = Text::new(input_box.hint_text.clone());
                    *color = TextColor(HINT_TEXT_COLOR);
                }
                let mut visibility = cursor_query.get_mut(input_box.cursor).unwrap();
                *visibility = Visibility::Hidden;
                if let Some(ref e) = input_focused && e.0 == entity {
                    commands.remove_resource::<InputFocused>();
                    window.ime_enabled = false;
                }
            }
        }
    }
}

pub fn input_box_blink_cursor(
    focused: Res<InputFocused>,
    input_query: Query<&InputBox, With<InputBox>>,
    mut cursor_query: Query<(Entity, &mut Visibility, &mut CursorMarker)>,
    time: Res<Time>,
) {
    // InputFocused里面保存了当前有输入焦点的InputBox，其cursor成员记录了对应的光标Node
    let current = input_query.get(focused.0).unwrap().cursor;
    for (entity, mut visibility, mut cursor)
        in cursor_query.iter_mut() {
        if cursor.timer.tick(time.delta()).just_finished() {
            if current == entity {
                *visibility = if *visibility == Visibility::Visible {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}

pub fn input_box_ime_events(
    mut events: EventReader<Ime>,
    mut texts: Query<(Entity, &mut Text, &TextFont, &mut InputBox), With<InputBox>>,
    focused: Res<InputFocused>,
    cursors: Query<&UiGlobalTransform, With<CursorMarker>>,
    input_borders: Query<(&UiGlobalTransform, &InputBoxBorder), With<InputBoxBorder>>,
    parents: Query<&ChildOf>,
    window: Single<&Window>,
) {
    let (entity, mut text, font, mut input_box) =
        texts.get_mut(focused.0).unwrap();
    // 分别获取输入文本框和光标的位置信息
    let (border_translation, border_size) = get_border_position(entity, parents, input_borders);
    let cursor_translation = cursors.get(input_box.cursor).unwrap().translation;
    // 根据光标位置判断是否还可以接受更多字符
    let accept_more =
        cursor_translation.x + font.font_size
            < border_translation.x + border_size.x * window.scale_factor() / 2.0;
    for event in events.read() {
        match event {
            Ime::Preedit { value, cursor, .. } if !cursor.is_none() => {
                if accept_more {
                    *text = Text::new(format!("{}{value}", input_box.value));
                }
            }
            Ime::Commit { value, .. } => {
                if accept_more {
                    input_box.value += value;
                    *text = Text::new(input_box.value.clone());
                }
            }
            _ => (),
        }
    }
}

pub fn input_box_keyboard_events(
    mut events: EventReader<KeyboardInput>,
    mut texts: Query<(Entity, &mut Text, &TextFont, &mut InputBox), With<InputBox>>,
    focused: Res<InputFocused>,
    cursors: Query<&UiGlobalTransform, With<CursorMarker>>,
    input_borders: Query<(&UiGlobalTransform, &InputBoxBorder), With<InputBoxBorder>>,
    parents: Query<&ChildOf>,
    window: Single<&Window>
) {
    let (entity, mut text, font, mut input_box) =
        texts.get_mut(focused.0).unwrap();
    // 分别获取输入文本框和光标的位置信息
    let (border_translation, border_size) = get_border_position(entity, parents, input_borders);
    let cursor_translation = cursors.get(input_box.cursor).unwrap().translation;
    // 根据光标位置判断是否还可以接受更多字符
    let accept_more =
        cursor_translation.x + font.font_size
            < border_translation.x + border_size.x * window.scale_factor() / 2.0;
    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match (&event.logical_key, &event.text) {
            (Key::Backspace, _) => {
                text.pop();
                input_box.value.pop();
            }
            (_, Some(inserted_text)) => {
                if accept_more && inserted_text.chars().all(is_printable_char) {
                    text.push_str(inserted_text);
                    input_box.value.push_str(inserted_text);
                }
            }
            _ => continue,
        }
    }
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}