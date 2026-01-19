use bevy::prelude::*;

pub struct ModelDialog {
    pub root: Entity,
    pub container: Entity,
}

impl ModelDialog {
    pub fn new<S: States>(commands: &mut Commands, state: S, width: f32) -> Self {
        let root = commands
            .spawn((
                DespawnOnExit(state),
                Node {
                    width: Val::Percent(width),
                    height: Val::Auto,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    border: UiRect::all(Val::Px(2.)),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb_u8(43, 44, 47)),
                BackgroundColor(Color::NONE),
            )).id();

        // 子节点
        let container = commands
            .spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    border: UiRect::all(Val::Px(3.)),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                BorderColor::all(Color::srgb_u8(76, 69, 113)),
                BackgroundColor(Color::srgb_u8(43, 44, 47)),
            )).id();

        // 关联父子关系
        commands.entity(root).add_child(container);
        
        Self { root, container }
    }
}