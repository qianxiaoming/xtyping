use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::render::view::visibility;
use crate::ui::TextConfig;

#[derive(Resource)]
pub struct InputFocused(Entity);

#[derive(Component)]
pub struct InputBox {
    text_color: Color,
    hint_text: String,
    size: Vec2,
    cursor: Entity
}

const INPUT_BOX_SIDE_COLOR: Color = Color::srgb_u8(240, 240, 240);

const INPUT_BOX_BOTTOM_COLOR: Color = Color::srgb_u8(0, 105, 186);

const HINT_TEXT_COLOR: Color = Color::srgb_u8(90, 90, 90);

#[derive(Component)]
pub struct CursorMarker {
    timer: Timer
}

impl InputBox {
    pub fn spawn<C: Component>(builder: &mut ChildSpawnerCommands,
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
                    size: Vec2::new(size.x, size.y),
                    cursor
                });
            });
        });
    }
}

fn cursor_inside_node(cursor_pos: Vec2, transform: &UiGlobalTransform, size: Vec2) -> bool {
    let node_pos = transform.translation;
    let half_size = size / 2.0;

    cursor_pos.x >= node_pos.x - half_size.x &&
        cursor_pos.x <= node_pos.x + half_size.x &&
        cursor_pos.y >= node_pos.y - half_size.y &&
        cursor_pos.y <= node_pos.y + half_size.y
}

pub fn handle_input_box_focus(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut query_input_box: Query<
        (
            Entity,
            &mut Text,
            &mut TextColor,
            &InputBox,
            &UiGlobalTransform
        ),
        With<InputBox>
    >,
    mut cursor_query: Query<&mut Visibility, With<CursorMarker>>,
    input_focused: Option<Res<InputFocused>>,
    windows: Query<&Window>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let window = windows.single().unwrap();
        let cursor_pos = window.cursor_position().unwrap_or_default();
        for (entity, mut text, mut color,  input_box,
            transform) in &mut query_input_box {
            if cursor_inside_node(cursor_pos * window.scale_factor(), transform, input_box.size) {
                *text = Text::new("");
                *color = TextColor(input_box.text_color);
                let mut visibility = cursor_query.get_mut(input_box.cursor).unwrap();
                *visibility = Visibility::Visible;
                commands.insert_resource(InputFocused(entity));
            } else {
                if text.0.is_empty() {
                    *text = Text::new(input_box.hint_text.clone());
                    *color = TextColor(HINT_TEXT_COLOR);
                }
                let mut visibility = cursor_query.get_mut(input_box.cursor).unwrap();
                *visibility = Visibility::Hidden;
                if let Some(ref e) = input_focused && e.0 == entity {
                    commands.remove_resource::<InputFocused>();
                }
            }
        }
    }
}

pub fn blink_input_box_cursor(
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

// 系统：键盘输入写入到当前焦点的输入框
// fn text_input_system(
//     mut char_events: EventReader<ReceivedCharacter>,
//     keys: Res<ButtonInput<KeyCode>>,
//     input_focus: Res<InputFocus>,
//     children_q: Query<&Children, With<InputBox>>,
//     mut text_q: Query<&mut Text, With<InputText>>,
// ) {
//     if let Some(focused) = input_focus.focused() {
//         if let Ok(children) = children_q.get(focused) {
//             for &child in children.iter() {
//                 if let Ok(mut text) = text_q.get_mut(child) {
//                     // 输入字符
//                     for ev in char_events.read() {
//                         if !ev.char.is_control() {
//                             text.0.push(ev.char);
//                         }
//                     }
//                     // 退格删除
//                     if keys.just_pressed(KeyCode::Backspace) {
//                         text.0.pop();
//                     }
//                 }
//             }
//         }
//     }
// }