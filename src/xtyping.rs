#![doc = include_str!("../README.md")]

mod startup;
mod widgets;
mod register;
mod playing;
mod ui;

use bevy::prelude::*;
use bevy::dev_tools::states::*;
use bevy::input_focus::InputFocus;
use bevy::window::WindowPlugin;
use serde::{Deserialize, Serialize};

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
        .add_sub_state::<PlayState>()
        .init_resource::<GameFonts>()
        .init_resource::<Players>()
        .add_systems(OnEnter(GameState::Init), init_resources)
        .add_systems(Startup, setup_camera)
        .add_plugins((
            startup::startup_plugin,
            register::new_player_plugin,
            playing::play_game_plugin,
            widgets::widgets_plugin
        ))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Init,
    Startup,
    Register,
    Playing
}

/// 玩游戏过程中的可能状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::Playing)]
#[states(scoped_entities)]
enum PlayState {
    #[default]
    Splash,    // 初始提示
    Playing,   // 玩家交互
    Paused,    // 暂停游戏
    Exiting,   // 确认退出
    Upgrading, // 升级祝贺
    Failed     // 玩家失败
}

#[derive(Resource, Default)]
struct GameFonts {
    title_font: Handle<Font>,
    normal_font: Handle<Font>,
    info_font: Handle<Font>,
    ui_font: Handle<Font>
}

#[derive(Deserialize, Serialize, Clone, Default)]
struct Player {
    name: String,
    avatar: String,
    score: u32,
    level: u32
}

#[derive(Deserialize, Resource, Default)]
struct Players(Vec<Player>);

const MAX_PLAYERS_COUNT: usize = 7;

impl Players {
    pub fn get(&self, name: &str) -> &Player {
        self.0.iter().find(|p| p.name == name).unwrap()
    }
}

#[derive(Resource, Default)]
struct GameData {
    pub player: Player,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 初始化全局的字体、用户以及图片资源
fn init_resources(
    mut players: ResMut<Players>,
    mut fonts: ResMut<GameFonts>,
    mut next: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>
) {
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