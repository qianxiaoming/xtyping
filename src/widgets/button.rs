use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use crate::widgets::UI_FONT_SIZE;

pub struct Button;

#[derive(BufferedEvent)]
pub struct ButtonClicked {
    pub entity: Entity,
}

const BUTTON_BORDER_WIDTH: f32 = 1.0;
const BUTTON_NORMAL_COLOR: Color = Color::srgb_u8(147, 51, 234);
const BUTTON_HOVERED_COLOR: Color = Color::srgb_u8(161, 67, 246);
const BUTTON_PRESSED_COLOR: Color = Color::srgb_u8(133, 35, 222);

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct ButtonText {
    pub text: String,
    pub font: Handle<Font>,
    pub font_size: f32,
    pub color: Color,
    pub shadow: bool
}

impl Default for ButtonText {
    fn default() -> Self {
        ButtonText {
            text: Default::default(),
            font: Default::default(),
            font_size: super::UI_FONT_SIZE,
            color: Color::WHITE,
            shadow: false
        }
    }
}

impl ButtonText {
    pub fn to_shadow(&self) -> TextShadow {
        if self.shadow {
            TextShadow::default()
        } else {
            TextShadow {
                offset: Vec2::splat(0.0),
                color: Color::BLACK
            }
        }
    }
}

impl Button {
    pub fn new<C: Component>(marker: C,
                             text: ButtonText,
                             size: Vec2,
                             margin: UiRect,
                             color: Color,
                             border_color: Color,
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
                BorderColor::all(border_color),
                corner.to_border_radius(),
                BackgroundColor(color),
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
            ButtonText {
                text: text.to_string(),
                font: super::UI_BUTTON_FONT.get().unwrap().clone(),
                ..default()
            },
            size,
            margin,
            BUTTON_NORMAL_COLOR,
            BUTTON_NORMAL_COLOR,
            CornerStyle::FullRounded
        )
    }
}

pub fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut bevy::prelude::Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    //mut text_query: Query<&mut Text>,
    mut font_query: Query<&mut TextFont>,
    mut writer: EventWriter<ButtonClicked>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        //let mut text = text_query.get_mut(children[0]).unwrap();
        let mut font = font_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *color = BUTTON_PRESSED_COLOR.into();
                *border_color = BorderColor::all(BUTTON_PRESSED_COLOR);
                font.font_size = UI_FONT_SIZE + 2.0;
                button.set_changed();
                writer.write(ButtonClicked { entity });
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                *color = BUTTON_HOVERED_COLOR.into();
                *border_color = BorderColor::all(BUTTON_HOVERED_COLOR);
                font.font_size = UI_FONT_SIZE + 2.0;
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                *color = BUTTON_NORMAL_COLOR.into();
                *border_color = BorderColor::all(BUTTON_NORMAL_COLOR);
                font.font_size = UI_FONT_SIZE;
            }
        }
    }
}