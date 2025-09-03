use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use crate::ui::TextConfig;
use crate::widgets::{Selected, UI_FONT_SIZE};

pub struct Button;

#[derive(BufferedEvent)]
pub struct ButtonClicked {
    pub entity: Entity,
}

const BUTTON_BORDER_WIDTH: f32 = 1.0;
const BUTTON_NORMAL_COLOR: Color = Color::srgb_u8(147, 51, 234);
const BUTTON_HOVERED_COLOR: Color = Color::srgb_u8(161, 67, 246);
const BUTTON_PRESSED_COLOR: Color = Color::srgb_u8(133, 35, 222);

#[derive(Clone, Copy)]
pub enum CornerStyle {
    Sharp,             // 尖角矩形
    Rounded,      // 可配置圆角半径
    FullRounded,       // 四角全圆
}

impl CornerStyle {
    pub fn to_border_radius(&self) -> BorderRadius {
        match self {
            CornerStyle::Sharp => BorderRadius::ZERO,
            CornerStyle::Rounded => BorderRadius::all(Val::Px(5.0)),
            CornerStyle::FullRounded => BorderRadius::MAX,
        }
    }
}

#[derive(Clone, Component)]
pub struct ButtonConfig {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub border: Color,
    pub width: f32,
    pub height: f32
}

impl Button {
    pub fn new<C: Component>(marker: C,
                             text: TextConfig,
                             size: Vec2,
                             margin: UiRect,
                             config: ButtonConfig,
                             corner: CornerStyle) -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin,
                ..default()
            },
            children![(
                bevy::ui::prelude::Button,
                marker,
                Node {
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(config.border),
                corner.to_border_radius(),
                BackgroundColor(config.normal),
                config,
                children![(
                    Text::new(text.text.clone()),
                    TextFont {
                        font: text.font.clone(),
                        font_size: text.font_size,
                        ..default()
                    },
                    TextColor(text.color),
                    text.to_shadow(),
                )]
            )],
        )
    }
}

pub struct PushButton;

impl PushButton {
    pub fn new<C: Component>(marker: C, text: &str, size: Vec2, margin: UiRect) -> impl Bundle {
        Button::new(
            marker,
            TextConfig {
                text: text.to_string(),
                font: super::UI_BUTTON_FONT.get().unwrap().clone(),
                ..default()
            },
            size,
            margin,
            ButtonConfig {
                normal: BUTTON_NORMAL_COLOR,
                border: BUTTON_NORMAL_COLOR,
                hovered: BUTTON_HOVERED_COLOR,
                pressed: BUTTON_PRESSED_COLOR,
                width: size.x,
                height: size.y
            },
            CornerStyle::FullRounded
        )
    }
}

pub struct IconButton;

#[derive(Clone, Component)]
pub struct ButtonValue(pub String);

impl IconButton {
    pub fn new<C: Component>(marker: C, value: String, image: Handle<Image>, size: Vec2,
                             background_color: Color, border_color: Color, margin: UiRect) -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin,
                ..default()
            },
            children![(
                bevy::ui::prelude::Button,
                marker,
                Node {
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(border_color),
                BorderRadius::all(Val::Px(5.0)),
                BackgroundColor(background_color),
                ButtonConfig {
                    normal: background_color,
                    border: border_color,
                    hovered: BUTTON_HOVERED_COLOR,
                    pressed: BUTTON_PRESSED_COLOR,
                    width: size.x,
                    height: size.y
                },
                ButtonValue(value),
                children![(
                    ImageNode::new(image),
                    Node {
                        width: Val::Px(size.x),
                        height: Val::Px(size.y),
                        margin,
                        ..default()
                    }
                )]
            )],
        )
    }
}

pub fn button_interaction_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut bevy::prelude::Button,
            &ButtonConfig,
            Option<&Selected>,
            &Children,
        ),
        Changed<Interaction>,
    >,
    //mut text_query: Query<&mut Text>,
    mut font_query: Query<&mut TextFont>,
    mut icon_query: Query<(&mut Node), With<ImageNode>>,
    mut writer: EventWriter<ButtonClicked>,
) {
    for (entity, interaction, mut color, mut border_color,
        mut button, config, selected, children) in &mut interaction_query
    {
        //let mut text = text_query.get_mut(children[0]).unwrap();
        let font = font_query.get_mut(children[0]);
        let icon = icon_query.get_mut(children[0]);
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *color = config.pressed.into();
                *border_color = BorderColor::all(config.pressed);
                if font.is_ok() {
                    font.unwrap().font_size = UI_FONT_SIZE + 2.0;
                }
                if let Ok(mut node) = icon {
                    node.width = Val::Px(config.width + 2.0);
                    node.height = Val::Px(config.height + 2.0);
                }
                button.set_changed();
                writer.write(ButtonClicked { entity });
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                *color = config.hovered.into();
                *border_color = BorderColor::all(config.hovered);
                if font.is_ok() {
                    font.unwrap().font_size = UI_FONT_SIZE + 2.0;
                }
                if let Ok(mut node) = icon {
                    node.width = Val::Px(config.width + 2.0);
                    node.height = Val::Px(config.height + 2.0);
                }
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                if selected.is_some() {
                    *color = config.pressed.into();
                } else {
                    *color = config.normal.into();
                }
                *border_color = BorderColor::all(config.normal);
                if font.is_ok() {
                    font.unwrap().font_size = UI_FONT_SIZE;
                }
                if let Ok(mut node) = icon {
                    node.width = Val::Px(config.width);
                    node.height = Val::Px(config.height);
                }
            }
        }
    }
}

pub fn button_style_selected_system(
    mut query: Query<(Entity, &mut BackgroundColor, &ButtonConfig, Option<&Selected>),
        (With<bevy::prelude::Button>, Changed<Selected>)>
) {
    for (entity, mut color, config, selected) in &mut query {
        if selected.is_some() {
            *color = config.pressed.into();
        }
    }
}

pub fn button_style_unselected_system(
    mut removed: RemovedComponents<Selected>,
    mut q_button: Query<(&mut BackgroundColor, &ButtonConfig), With<bevy::prelude::Button>>,
) {
    for entity in removed.read() {
        if let Ok((mut color, config)) = q_button.get_mut(entity) {
            *color = config.normal.into();
        }
    }
}