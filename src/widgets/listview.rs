use std::cmp::PartialEq;
use bevy::color::Color;
use bevy::input_focus::InputFocus;
use bevy::math::Vec2;
use bevy::prelude::*;
use crate::widgets::TextConfig;

const ENTRY_HOVERED_COLOR: Color = Color::srgb_u8(161, 67, 246);

pub enum ListItem {
    Image(Handle<Image>, Vec2),
    Text(String),
    Blank,
    Indicator(Handle<Image>, Vec2),
    Command(Vec<(String, Handle<Image>)>)
}

#[derive(PartialEq)]
enum ListItemType {
    Image,
    Text,
    Indicator,
    Command
}

#[derive(Component)]
pub struct ListItemMarker(ListItemType);

pub struct ListEntry {
    pub value: String,
    pub entities: Vec<Entity>
}

#[derive(Component, Default)]
pub struct ListViewMarker {
    pub entries: Vec<ListEntry>,
    pub selected: Option<usize>,
    pub hovered: Option<usize>,
    pub text_config: TextConfig,
}

pub struct ListView {
    pub entity: Entity,
    pub text_config: TextConfig,
    pub indicator: Option<(Handle<Image>, Vec2)>,
}

impl ListView {
    pub fn new<C: Component>(builder: &mut ChildSpawnerCommands,
                             marker: C,
                             text: TextConfig,
                             columns: Vec<RepeatedGridTrack>,
                             rows: Vec<RepeatedGridTrack>,
                             indicator: Option<(Handle<Image>, Vec2)>,
                             size: Option<Vec2>,
                             margin: Option<UiRect>) -> ListView {
        let (width, height) = if let Some(Vec2{x, y }) = size {
            (Val::Px(x), Val::Px(y))
        } else {
            (Val::Percent(100.), Val::Auto)
        };
        let columns = if indicator.is_some() {
            let mut cols: Vec<RepeatedGridTrack> = vec![GridTrack::min_content()];
            cols.extend(columns);
            cols
        } else {
            columns
        };
        let entity = builder.spawn((
            Node {
                display: Display::Grid,
                width,
                height,
                margin: margin.unwrap_or_default(),
                grid_template_columns: columns,
                grid_template_rows: rows,
                row_gap: px(10),
                column_gap: px(10),
                ..default()
            },
            Button,
            ListViewMarker {
                entries: Vec::new(),
                selected: None,
                hovered: None,
                text_config: text.clone(),
            },
            marker,
            BackgroundColor(Color::NONE),
        )).id();

        ListView{
            entity,
            text_config: text,
            indicator
        }
    }

    pub fn append(&self, commands: &mut Commands, value: String, items: Vec<ListItem>) {
        let entity = self.entity;
        let text_config = self.text_config.clone();
        let mut entry = ListEntry {
            value,
            entities: Vec::new()
        };
        // 如果设定了选择指示器图标，则增加一个列定义
        let items = if let Some(ref indicator) = self.indicator {
            std::iter::once(ListItem::Indicator(indicator.0.clone(), indicator.1)).chain(items).collect::<Vec<_>>()
        } else {
            items
        };
        commands.queue(move |world: &mut World| {
            for item in items {
                let item_id = match item {
                    ListItem::Text(_) | ListItem::Blank => {
                        let text = match item {
                            ListItem::Text(text) => text,
                            _ => " ".to_string()
                        };
                        world.spawn((
                            Text::new(text),
                            TextFont {
                                font: text_config.font.clone(),
                                font_size: text_config.font_size,
                                ..default()
                            },
                            TextColor(text_config.color),
                            text_config.to_shadow(),
                            ListItemMarker(ListItemType::Text),
                        )).id()
                    },
                    ListItem::Image(image, size) => {
                        world.spawn((
                            ImageNode::new(image.clone()),
                            Node {
                                width: Val::Px(size.x),
                                height: Val::Px(size.y),
                                ..default()
                            },
                            ListItemMarker(ListItemType::Image),
                        )).id()
                    },
                    ListItem::Indicator(image, size) => {
                        world.spawn((
                            ImageNode::new(image.clone()),
                            Node {
                                width: Val::Px(size.x),
                                height: Val::Px(size.y),
                                ..default()
                            },
                            Visibility::Hidden,
                            ListItemMarker(ListItemType::Indicator),
                        )).id()
                    },
                    ListItem::Command(_) => {
                        !unreachable!()
                    },
                };
                entry.entities.push(item_id);
            };

            if let Ok(mut listview) = world.get_entity_mut(entity) {
                entry.entities.iter().for_each(|id| {listview.add_child(*id); });
            }

            if let Some(mut listview) = world.get_mut::<ListViewMarker>(entity) {
                listview.entries.push(entry);
            };
        });
    }
}

#[derive(BufferedEvent)]
pub struct ListViewSelectionChanged {
    pub entity: Entity,
    pub value: String,
}

pub fn listview_interaction_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut ListViewMarker,
            &UiGlobalTransform,
            &Children
        ),
        Changed<Interaction>,
    >,
    mut item_query: Query<
        (
            &mut ListItemMarker,
            &mut Visibility,
            Option<&mut TextColor>,
        ),
        With<ListItemMarker>>,
    mut writer: EventWriter<ListViewSelectionChanged>,
    window: Single<&Window>
) {
    for (entity, interaction,
        mut listview, transform, children)
        in &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                if let Some(selected) = listview.hovered {
                    writer.write(ListViewSelectionChanged{
                        entity,
                        value: listview.entries[selected].value.clone(),
                    });
                };
            },
            Interaction::Hovered => {
                input_focus.set(entity);
            },
            Interaction::None => {
                input_focus.clear();
                if let Some(hovered) = listview.hovered {
                    for e in listview.entries[hovered].entities.iter() {
                        let (marker, mut vis, color)
                            = item_query.get_mut(*e).unwrap();
                        if let Some(mut color) = color {
                            *color = TextColor(listview.text_config.color);
                        }
                        if marker.0 == ListItemType::Indicator {
                            *vis = Visibility::Hidden;
                        }
                    }
                }
                listview.hovered = None;
            }
        }
    }
}

pub fn listview_cursor_move_system(
    input_focus: Res<InputFocus>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut listview_query: Query<&mut ListViewMarker>,
    mut item_query: Query<
        (
            &UiGlobalTransform,
            &ComputedNode,
            &mut ListItemMarker,
            &mut Visibility,
            Option<&mut TextColor>,
        ),
        With<ListItemMarker>>,
    window: Single<&Window>
) {
    if let Some(entity) = input_focus.0
        && let Ok(mut listview) = listview_query.get_mut(entity) {
        if let Some(event) = cursor_moved_events.read().last() {
            let pos = event.position * window.scale_factor();
            let mut hovered_entry: Option<usize> = None;
            for (index, entry) in listview.entries.iter().enumerate() {
                let (transform, node, _, _, _) =
                    item_query.get(entry.entities[1]).unwrap();

                let y1 = transform.translation.y - node.size.y / 2.;
                let y2 = transform.translation.y + node.size.y / 2.;
                if pos.y > y1 && pos.y < y2 {
                    hovered_entry = Some(index);
                    break;
                }
            }

            if listview.hovered != hovered_entry {
                // 如果之前有 hovered，就先重置
                if let Some(last_hovered) = listview.hovered {
                    for e in listview.entries[last_hovered].entities.iter() {
                        let (_, _, marker, mut vis, color)
                            = item_query.get_mut(*e).unwrap();
                        if let Some(mut color) = color {
                            *color = TextColor(listview.text_config.color);
                        }
                        if marker.0 == ListItemType::Indicator {
                            *vis = Visibility::Hidden;
                        }
                    }
                }

                // 如果现在有新的 hovered，就设置
                if let Some(current) = hovered_entry {
                    for e in listview.entries[current].entities.iter() {
                        let (_, _, marker, mut vis, color)
                            = item_query.get_mut(*e).unwrap();
                        if let Some(mut color) = color {
                            *color = TextColor(ENTRY_HOVERED_COLOR);
                        }
                        if marker.0 == ListItemType::Indicator {
                            *vis = Visibility::Visible;
                        }
                    }
                }

                // 更新状态
                listview.hovered = hovered_entry;
            }
        }
    }
}