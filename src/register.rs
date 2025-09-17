use std::fs::File;
use std::io::Write;
use bevy::prelude::*;
use super::*;
use ui::*;
use widgets::TextConfig;
use crate::widgets::InputBox;

const PLAYER_AVATARS: [&str; 28] = [
    "whale", "cat", "cool", "donatello", "dragon", "swordsman", "robot",
    "elephant", "ghost", "hero", "hero-boy", "hero-girl", "hero-girl2", "sun",
    "kitty", "monkey", "monkey-cool", "panda", "panda-sleep", "assasin", "spierman",
    "sea-turtle", "snake", "tiger", "angel", "rabbit", "smiling", "dog"];

pub fn new_player_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Register), new_player_setup)
        .add_systems(OnExit(GameState::Register), new_player_exit)
        .add_systems(Update, on_cancel_button.run_if(in_state(GameState::Register)))
        .add_systems(Update, on_avatar_button.run_if(in_state(GameState::Register)))
        .add_systems(Update, on_create_button.run_if(in_state(GameState::Register)));
}

#[derive(Component, Default)]
struct NewPlayerEntity;

#[derive(Component)]
struct ButtonCreate;

#[derive(Component)]
struct ButtonCancel;

#[derive(Component)]
struct ButtonAvatar;

#[derive(Component)]
struct PlayerNameText;

#[derive(Resource, Default)]
struct SelectedAvatar(Option<Entity>, Option<String>);

fn new_player_setup(mut commands: Commands, fonts: Res<GameFonts>, asset_server: Res<AssetServer>) {
    commands.insert_resource(SelectedAvatar::default());
    spawn_startup_root::<NewPlayerEntity>(&mut commands)
        .with_children(|parent| {
            spawn_game_title(parent, &fonts, 1., 20., 15., 20., true);
            spawn_instructions(parent, "1. 输入一个喜欢的名称作为账户名", &fonts, 100.0);
            InputBox::new(
                parent,
                PlayerNameText,
                TextConfig {
                    text: "".to_string(),
                    font: fonts.info_font.clone(),
                    font_size: 16.0,
                    color: Color::WHITE,
                    shadow: false
                },
                "在此输入名称",
                Vec2::new(300., 32.),
                UiRect::all(Val::Px(10.)));
            spawn_instructions(parent, "2. 选择一个喜欢的头像代表你自己", &fonts, 20.0);
            parent.spawn((
                    Node {
                        width: Val::Auto,
                        height: Val::Auto,
                        display: Display::Grid,
                        padding: UiRect::all(Val::Px(10.0)),
                        grid_template_columns: RepeatedGridTrack::flex(7, 1.0),
                        grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
                        row_gap: Val::Px(10.0),
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                )).with_children(|builder| {
                for avatar in PLAYER_AVATARS {
                    let path = format!("avatars/{}.png", avatar);
                    builder.spawn(
                        widgets::IconButton::new(ButtonAvatar,
                                                 avatar.to_owned(),
                                                 asset_server.load(path),
                                                 Vec2::new(64.0, 64.0),
                                                 Color::NONE,
                                                 Color::WHITE,
                                                 UiRect::all(Val::Px(12.)))
                    );
                }
            });
            parent.spawn(
                Node {
                    width: Val::Auto,
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                }
            ).with_children(|builder| {
                builder.spawn(
                    widgets::PushButton::new(ButtonCreate,
                                             "创建",
                                             Vec2::new(100.0,40.0),
                                             true,
                                             UiRect {
                                                 top: Val::Px(30.0),
                                                 right: Val::Px(5.0),
                                                 ..default()
                                             })
                    );
                builder.spawn(
                    widgets::PushButton::new(ButtonCancel,
                                             "取消",
                                             Vec2::new(100.0,40.0),
                                             true,
                                             UiRect {
                                                 top: Val::Px(30.0),
                                                 left: Val::Px(5.0),
                                                 ..default()
                                             })
                    );
            });
        });
}

fn new_player_exit(mut commands: Commands, query: Query<Entity, With<NewPlayerEntity>>) {
    commands.remove_resource::<SelectedAvatar>();
    cleanup_entities::<NewPlayerEntity>(commands, query);
}

fn on_avatar_button(
    mut commands: Commands,
    mut selected: ResMut<SelectedAvatar>,
    mut reader: MessageReader<widgets::ButtonClicked>,
    query: Query<&widgets::ButtonValue, With<ButtonAvatar>>,
) {
    for event in reader.read() {
        if let Ok(value) = query.get(event.entity) {
            if let Some(prev) = selected.0 {
                if prev != event.entity {
                    commands.entity(prev).remove::<widgets::Selected>();
                }
            }
            commands.entity(event.entity).insert(widgets::Selected);
            selected.0 = Some(event.entity);
            selected.1 = Some(value.0.clone());
        }
    }
}

fn on_create_button(
    selected: Res<SelectedAvatar>,
    mut players: ResMut<Players>,
    mut next_state: ResMut<NextState<GameState>>,
    mut reader: MessageReader<widgets::ButtonClicked>,
    query: Query<(), With<ButtonCreate>>,
    player_name: Single<&InputBox, With<PlayerNameText>>
) {
    for event in reader.read() {
        if query.get(event.entity).is_ok() {
            if !player_name.value.is_empty() && let Some(ref avatar) = selected.1 {
                players.0.push(Player {
                    name: player_name.value.clone(),
                    avatar: avatar.clone(),
                    score: 0,
                    level: 1
                });
                let json = serde_json::to_string_pretty(&players.0).unwrap();
                if let Ok(mut file) = File::create(PLAYERS_DATA_FILE) {
                    if let Err(e) = file.write_all(json.as_bytes()) {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                }
                next_state.set(GameState::Startup);
            }
        }
    }
}

fn on_cancel_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut reader: MessageReader<widgets::ButtonClicked>,
    query: Query<(), With<ButtonCancel>>,
) {
    for event in reader.read() {
        if query.get(event.entity).is_ok() {
            next_state.set(GameState::Startup);
        }
    }
}
