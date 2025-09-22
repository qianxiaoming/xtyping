#![doc = include_str!("../README.md")]

mod startup;
mod widgets;
mod register;
mod gaming;
mod ui;

use bevy::prelude::*;
use bevy::input_focus::InputFocus;
use bevy::window::WindowPlugin;
use serde::{Deserialize, Serialize};

const GAME_APP_TITLE: &str = "超级打字练习";
const PLAYERS_DATA_FILE: &str = "players.json";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: GAME_APP_TITLE.to_string(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_sub_state::<PlayState>()
        .init_resource::<InputFocus>()
        .init_resource::<GameSettings>()
        .init_resource::<GameFonts>()
        .init_resource::<Players>()
        .init_resource::<ExplosionTexture>()
        .add_systems(OnEnter(GameState::Init), init_resources)
        .add_systems(Startup, setup_camera)
        .add_plugins((
            startup::startup_plugin,
            register::new_player_plugin,
            gaming::play_game_plugin,
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
    Gaming
}

/// 玩游戏过程中的可能状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::Gaming)]
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
    ui_font: Handle<Font>,
    letter_font: Handle<Font>,
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
const MAX_PLAYER_LEVELS: usize = 5;

impl Players {
    pub fn get(&self, name: &str) -> &Player {
        self.0.iter().find(|p| p.name == name).unwrap()
    }
}

const DEFAULT_ROUTE_HEIGHT: f32 = 40.;
const MAX_ROUTE_COUNT: usize = 64;
const GAME_INFO_AREA_HEIGHT: f32 = 70.;
const GAME_INFO_AREA_MARGIN: f32 = 30.;

#[derive(Clone, Default)]
struct Route {
    pub id: i32,
    pub entities: Vec<Entity>,
}

impl Route {
    pub fn get_position(&self, window_height: f32) -> f32 {
        let start = window_height / 2. - GAME_INFO_AREA_HEIGHT - GAME_INFO_AREA_MARGIN;
        start - self.id as f32 * DEFAULT_ROUTE_HEIGHT - DEFAULT_ROUTE_HEIGHT / 2.
    }
}

#[derive(Resource, Default)]
struct GamePlayer{
    pub player: Player,
    pub safe_position: f32
}

#[derive(Resource, Default)]
struct GameRoutes {
    pub empty_routes: Vec<Route>,
    pub used_routes: Vec<Route>,
}

#[derive(Resource, Default)]
struct GameLetters {
    pub candidate_letters: Vec<char>,
    pub choosed_letters: Vec<char>
}

#[derive(Resource, Default)]
struct ExplosionTexture {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>
}

#[derive(Resource)]
struct GameSettings {
    // 不同用户级别对应的字母
    pub level_letters: Vec<Vec<char>>,
    // 不同用户级别对应的飞行速度区间
    pub level_speeds: Vec<(f32, f32)>,
    // 不同用户级别每一关的敌机数量
    pub aircraft_count: Vec<usize>,
    // 敌机出现的时间间隔
    pub aircraft_intervals: Vec<(f32, f32)>,
    // 敌机的开火距离
    pub firing_distance: f32,
    // 炸弹出现的时间间隔
    pub bomb_intervals: Vec<(f32, f32)>,
    // 护盾出现的时间间隔
    pub shield_intervals: Vec<(f32, f32)>,
    // 血包出现的时间间隔
    pub health_pack_intervals: Vec<(f32, f32)>,
    // 玩家发射的导弹速度
    pub missile_speed: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        let letters: Vec<Vec<char>> = [
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "1234567890",
            "+-*/=?,.!;",
            ":\"#*<>%'()",
            "[]{}@_|"
        ]
            .iter()
            .map(|s| s.chars().collect())
            .collect();

        let mut level_letters: Vec<Vec<char>> = Vec::with_capacity(MAX_PLAYER_LEVELS);
        let mut current = Vec::new();
        for i in 0..MAX_PLAYER_LEVELS {
            if i < letters.len() {
                current.extend(&letters[i]);
            }
            level_letters.push(current.clone());
        }

        GameSettings {
            level_letters,
            level_speeds: vec![(15., 30.),(40., 80.),(80., 120.),(120., 150.),(150., 200.)],
            aircraft_count: vec![150, 200, 300, 400, 500],
            aircraft_intervals: vec![(4., 8.),(3., 5.),(1.2, 2.),(0.8, 1.5),(0.3, 1.2)],
            firing_distance: 200.,
            bomb_intervals: vec![(60., 90.),(90., 120.),(120., 150.),(150., 300.),(300., 500.)],
            shield_intervals: vec![(100., 150.),(150., 200.),(200., 250.),(250., 300.),(300., 350.)],
            health_pack_intervals: vec![(200., 300.),(300., 400.),(400., 500.),(500., 600.),(600., 700.)],
            missile_speed: 1000.
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 初始化全局的字体、用户以及图片资源
fn init_resources(
    mut players: ResMut<Players>,
    mut fonts: ResMut<GameFonts>,
    mut next: ResMut<NextState<GameState>>,
    mut texture: ResMut<ExplosionTexture>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    fonts.title_font = asset_server.load("fonts/sharphei.ttf");
    fonts.normal_font = asset_server.load("fonts/happyfont.ttf");
    fonts.info_font = asset_server.load("fonts/sans.ttf");
    fonts.ui_font = asset_server.load("fonts/cubehei.ttf");
    fonts.letter_font = asset_server.load("fonts/letter-bold.ttf");

    widgets::UI_BUTTON_FONT.set(fonts.ui_font.clone()).ok();

    texture.texture = asset_server.load("images/explosion.png");
    texture.layout = texture_atlas_layouts.add(
        TextureAtlasLayout::from_grid(UVec2::new(150, 129), 3, 3, None, None)
    );

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