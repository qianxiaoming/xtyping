#![doc = include_str!("../README.md")]

mod startup;
mod widgets;
mod register;
mod gaming;
mod ui;

use std::path::PathBuf;
use rand::prelude::SliceRandom;
use bevy::prelude::*;
use bevy::input_focus::InputFocus;
use bevy::window::WindowPlugin;
use serde::{Deserialize, Serialize};

const GAME_APP_TITLE: &str = "超级打字练习";
const GAME_APP_NAME: &str = "xtyping";

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
    Gaming,
    Restart
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
    Checkpoint, // 游戏过关
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

#[derive(Deserialize, Serialize, Clone, Default, PartialEq)]
struct Player {
    name: String,
    avatar: String,
    score: u32,
    level: u32
}

#[derive(Deserialize, Resource, Default)]
struct Players(Vec<Player>);

const MAX_PLAYERS_COUNT: usize = 7;
const MAX_PLAYER_LEVELS: u32 = 5;

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
struct GamePlayer {
    pub player: Player,
    pub safe_position: f32,
    pub health: u16,
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
    // 不同级别用户战舰的发射时间间隔
    pub warship_fire_interval: Vec<f32>,
    // 战舰单个火炮发射间隔
    pub warship_gun_interval: f32,
    // 升级的分数
    pub upgrade_scores: Vec<u32>,
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
    // 护盾防护的时间
    pub shield_active_time: f32,
    // 血包出现的时间间隔
    pub health_pack_intervals: Vec<(f32, f32)>,
    // 玩家发射的导弹速度
    pub missile_speed: f32,
    // 战舰发射的火焰速度
    pub flame_speed: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        let letters: Vec<Vec<char>> = [
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "1234567890",
            "+-*/:=%,.;",
            "?\"{}#!()",
            "[]<>@_|'"
        ]
            .iter()
            .map(|s| s.chars().collect())
            .collect();

        let mut level_letters: Vec<Vec<char>> = Vec::with_capacity(MAX_PLAYER_LEVELS as usize);
        let mut current = Vec::new();
        let mut rng = rand::rng();
        for i in 0..MAX_PLAYER_LEVELS as usize {
            if i < letters.len() {
                current.extend(&letters[i]);
                if i == 1 || i == 3 {
                    current.extend(&letters[0]);
                }
                current.shuffle(&mut rng);
            }
            level_letters.push(current.clone());
        }

        GameSettings {
            level_letters,
            level_speeds: vec![(50., 80.),(80., 110.),(120., 150.),(150., 180.),(180., 220.)],
            warship_fire_interval: vec![4., 3., 2., 1., 0.5],
            upgrade_scores: vec![2000, 10000, 28000, 50000],
            aircraft_count: vec![3, 300, 400, 500, 600],
            aircraft_intervals: vec![(3., 5.),(1.5, 3.),(1., 1.5),(0.8, 1.),(0.3, 1.)],
            firing_distance: 200.,
            bomb_intervals: vec![(150., 300.),(250., 350.),(300., 450.),(400., 500.),(450., 600.)],
            shield_intervals: vec![(200., 250.),(250., 300.),(300., 450.),(450., 500.),(500., 550.)],
            health_pack_intervals: vec![(300., 400.),(400., 500.),(500., 600.),(600., 700.),(600., 700.)],
            shield_active_time: 30.,
            missile_speed: 1000.,
            flame_speed: 500.,
            warship_gun_interval: 0.3,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn get_app_data_dir(app_name: &str) -> PathBuf {
    let mut base_dir = dirs::data_dir().expect("无法获取用户数据目录");
    base_dir.push(app_name);

    std::fs::create_dir_all(&base_dir).expect("创建应用数据目录失败");
    base_dir
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

    let mut data_file = get_app_data_dir(GAME_APP_NAME);
    data_file.push(PLAYERS_DATA_FILE);
    if std::path::Path::new(&data_file).exists() {
        std::fs::read_to_string(&data_file)
            .and_then(|data| serde_json::from_str::<Vec<Player>>(&data).map_err(|err| err.into()))
            .map(|data| players.0.extend(data))
            .unwrap_or_else(|err| error!("Failed to parse player data: {}", err));
    } else {
        info!("No player data file found in current directory");
    }

    next.set(GameState::Startup);
}

fn save_game_users(players: &Players) {
    if let Ok(json) = serde_json::to_string_pretty(&players.0) {
        let mut data_file = get_app_data_dir(GAME_APP_NAME);
        data_file.push(PLAYERS_DATA_FILE);
        if let Err(e) = std::fs::write(&data_file, json.as_bytes()) {
            error!("Failed to save player data: {}", e);
        }
    }
}