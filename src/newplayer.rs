use bevy::prelude::*;
use super::*;
use ui::*;

const PLAYER_AVATARS: [&str; 28] = [
    "whale", "cat", "cool", "donatello", "dragon", "swordsman", "robot",
    "elephant", "ghost", "hero", "hero-boy", "hero-girl", "owl", "sun",
    "kitty", "monkey", "monkey-cool", "panda", "panda-sleep", "assasin", "detective",
    "sea-turtle", "snake", "tiger", "angel", "rabbit", "smiling", "animal"];

pub fn new_player_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::NewPlayer), new_player_setup)
        .add_systems(OnExit(GameState::NewPlayer), new_player_exit)
        .add_systems(Update, on_cancel_button.run_if(in_state(GameState::NewPlayer)))
        .add_systems(Update, on_avatar_button.run_if(in_state(GameState::NewPlayer)));
}

#[derive(Component, Default)]
struct NewPlayerEntity;

#[derive(Component)]
struct ButtonCreate;

#[derive(Component)]
struct ButtonCancel;

#[derive(Component)]
struct ButtonAvatar;

#[derive(Resource, Default)]
struct SelectedAvatar(Option<Entity>, Option<String>);

fn new_player_setup(mut commands: Commands, players: Res<Players>, fonts: Res<GameFonts>, asset_server: Res<AssetServer>) {
    commands.insert_resource(SelectedAvatar::default());
    spawn_startup_root::<NewPlayerEntity>(&mut commands)
        .with_children(|parent| {
            spawn_game_title(parent, &fonts);
            spawn_instructions(parent, "1. 输入一个喜欢的名称作为账户名", &fonts, 100.0);
            parent.spawn((
                Node {
                    width: Val::Px(300.),
                    height: Val::Px(32.),
                    margin: UiRect::all(Val::Px(10.)),
                    padding: UiRect::left(Val::Px(10.)),
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
                    top: Color::srgb_u8(240, 240, 240),
                    bottom: Color::srgb_u8(0, 105, 186),
                    left: Color::srgb_u8(240, 240, 240),
                    right: Color::srgb_u8(240, 240, 240),
                }
            )).with_children(|builder| {
                spawn_info_text(builder, "在此输入名称", Color::srgb_u8(90, 90, 90),
                                fonts.info_font.clone(), 16.0);
            });
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
    mut reader: EventReader<widgets::ButtonClicked>,
    query: Query<(&widgets::ButtonValue), With<ButtonAvatar>>,
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

fn on_cancel_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut reader: EventReader<widgets::ButtonClicked>,
    query: Query<(), With<ButtonCancel>>,
) {
    for event in reader.read() {
        if query.get(event.entity).is_ok() {
            next_state.set(GameState::Startup);
        }
    }
}
