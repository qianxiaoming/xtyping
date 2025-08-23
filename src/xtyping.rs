#![doc = include_str!("../README.md")]

mod menu;
mod game;

use bevy::{prelude::*, dev_tools::states::*};
use bevy::window::{WindowMode, WindowPlugin};
use serde::Deserialize;

const GAME_APP_TITLE: &str = "超级打字练习";
const PLAYERS_DATA_FILE: &str = "players.dat";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: GAME_APP_TITLE.to_string(),
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_systems(Startup, setup_game)
        .add_plugins((menu::menu_plugin, game::game_plugin))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    StartMenu,
    NewPlayer,
    Countdown,
    TypeShooting,
    GamePaused,
    ConfirmExit
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

fn setup_game(mut commands: Commands, mut players: ResMut<Players>, mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.single_mut() {
        window.set_maximized(true);
    }

    commands.spawn(Camera2d);

    if std::path::Path::new(PLAYERS_DATA_FILE).exists() {
        std::fs::read_to_string(PLAYERS_DATA_FILE)
            .and_then(|data| serde_json::from_str::<Vec<Player>>(&data).map_err(|err| err.into()))
            .map(|data| players.0.extend(data))
            .unwrap_or_else(|err| error!("Failed to parse player data: {}", err));
    } else {
        info!("No player data file found in current directory");
    }
}