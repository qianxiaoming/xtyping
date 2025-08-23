#![doc = include_str!("../README.md")]
use bevy::{prelude::*, dev_tools::states::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup_game)
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

fn setup_game(mut commands: Commands) {
    commands.spawn(Camera2d);
}