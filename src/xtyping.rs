#![doc = include_str!("../README.md")]

mod startup;
mod game;
mod widgets;
mod ui;

use bevy::{prelude::*, dev_tools::states::*};
use bevy::input_focus::InputFocus;
use bevy::window::{WindowPlugin, WindowResolution};
use bevy::winit::WinitSettings;
use serde::Deserialize;

const GAME_APP_TITLE: &str = "超级打字练习";
const PLAYERS_DATA_FILE: &str = "players.dat";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: GAME_APP_TITLE.to_string(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<InputFocus>()
        .init_state::<GameState>()
        .init_resource::<GameFonts>()
        .init_resource::<Players>()
        .add_systems(OnEnter(GameState::InitResources), init_resources)
        .add_systems(Startup, setup_camera)
        .add_plugins((startup::startup_plugin, game::game_plugin, widgets::widgets_plugin))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    InitResources,
    Startup,
    NewPlayer,
    TypeShooting,
    GamePaused,
    ConfirmExit
}

#[derive(Resource, Default)]
struct GameFonts {
    title_font: Handle<Font>,
    normal_font: Handle<Font>,
    info_font: Handle<Font>,
    ui_font: Handle<Font>
}

#[derive(Deserialize, Default)]
struct Player {
    name: String,
    avatar: String,
    score: u32,
    level: u32
}

#[derive(Deserialize, Resource, Default)]
struct Players(Vec<Player>);

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 初始化全局的字体、用户以及图片资源
fn init_resources(mut players: ResMut<Players>,
                  mut fonts: ResMut<GameFonts>,
                  asset_server: Res<AssetServer>,
                  mut next: ResMut<NextState<GameState>>) {
    fonts.title_font = asset_server.load("fonts/sharphei.ttf");
    fonts.normal_font = asset_server.load("fonts/happyfont.ttf");
    fonts.info_font = asset_server.load("fonts/sans.ttf");
    fonts.ui_font = asset_server.load("fonts/cubehei.ttf");

    widgets::UI_BUTTON_FONT.set(fonts.ui_font.clone()).ok();

    if std::path::Path::new(PLAYERS_DATA_FILE).exists() {
        std::fs::read_to_string(PLAYERS_DATA_FILE)
            .and_then(|data| serde_json::from_str::<Vec<Player>>(&data).map_err(|err| err.into()))
            .map(|data| players.0.extend(data))
            .unwrap_or_else(|err| error!("Failed to parse player data: {}", err));
    } else {
        info!("No player data file found in current directory");
    }
    next.set(GameState::Startup);
}