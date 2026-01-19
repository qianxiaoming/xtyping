#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![doc = include_str!("../README.md")]

mod gaming;
mod register;
mod startup;
mod ui;
mod widgets;

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

const GAME_APP_TITLE: &str = "超级打字练习";
const GAME_APP_NAME: &str = "xtyping";

const PLAYERS_DATA_FILE: &str = "players.json";

const WARSHIP_SENTENCES_FILE: &str = "sentences.json";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: GAME_APP_TITLE.to_string(),
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            file_path: resolve_assets_path(),
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
            widgets::widgets_plugin,
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
    Restart,
}

/// 玩游戏过程中的可能状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::Gaming)]
#[states(scoped_entities)]
enum PlayState {
    #[default]
    Splash, // 初始提示
    Playing,    // 玩家交互
    Paused,     // 暂停游戏
    Exiting,    // 确认退出
    Checkpoint, // 游戏过关
    Upgrading,  // 升级祝贺
    Failed,     // 玩家失败
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
    level: u32,
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
    pub choosed_letters: Vec<char>,
}

#[derive(Resource, Default)]
struct ExplosionTexture {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

fn get_app_data_dir(app_name: &str) -> PathBuf {
    let mut base_dir = dirs::data_dir().expect("无法获取用户数据目录");
    base_dir.push(app_name);

    fs::create_dir_all(&base_dir).expect("创建应用数据目录失败");
    base_dir
}

fn sync_sentences_with_file(data: &mut Vec<Vec<String>>, file_path: &Path) -> io::Result<()> {
    if !file_path.exists() {
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut file = fs::File::create(file_path)?;
        file.write_all(json.as_bytes())?;
    } else {
        let content = fs::read_to_string(file_path)?;
        match serde_json::from_str::<Vec<Vec<String>>>(&content) {
            Ok(parsed) => {
                *data = parsed; // 替换原内容
            }
            Err(e) => {
                error!("解析英语句子资源文件是比啊：{}", e);
            }
        }
    }

    Ok(())
}

#[derive(Resource)]
struct GameSettings {
    // 不同用户级别对应的字母
    pub level_letters: Vec<Vec<char>>,
    // 不同用户级别对应的战舰句子
    pub level_sentences: Vec<Vec<String>>,
    // 不同用户级别对应的飞行速度区间
    pub level_speeds: Vec<(f32, f32)>,
    // 不同级别用户战舰的发射时间间隔
    pub warship_fire_interval: Vec<f32>,
    // 战舰单个火炮发射间隔
    pub warship_gun_interval: Vec<f32>,
    // 升级的分数
    pub upgrade_scores: Vec<u32>,
    // 不同用户级别每一关的敌机数量
    pub aircraft_count: Vec<usize>,
    // 敌机出现的时间间隔
    pub aircraft_intervals: Vec<(f32, f32)>,
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
            "[]<>@_|'",
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

        let mut level_sentences = vec![
            vec![
                "well done".to_owned(),
                "excuse me".to_owned(),
                "my best friend".to_owned(),
                "be quiet".to_owned(),
                "listen carefully".to_owned(),
                "Good morning".to_owned(),
                "Be happy".to_owned(),
                "Thank you".to_owned(),
                "Hello bird".to_owned(),
                "Come here".to_owned(),
            ],
            vec![
                "Knowledge is power".to_owned(),
                "The sun is bright today".to_owned(),
                "My cat is under the chair".to_owned(),
                "We run fast in the park".to_owned(),
                "Kind words cost nothing".to_owned(),
                "The stars shine in the sky".to_owned(),
                "My dog waits at the gate".to_owned(),
                "We play games after school".to_owned(),
                "Her smile makes me happy".to_owned(),
                "Hope is a waking dream".to_owned(),
            ],
            vec![
                "The teacher reads a story to us".to_owned(),
                "We play football after school today".to_owned(),
                "He drinks milk every morning".to_owned(),
                "The dog sleeps on the sofa".to_owned(),
                "They are singing in the classroom".to_owned(),
                "The gentle wind moves the green leaves".to_owned(),
                "We walk together under the blue sky".to_owned(),
                "My little brother laughs in the garden".to_owned(),
                "The moon shines softly on the lake".to_owned(),
                "I write a story about my dream".to_owned(),
            ],
            vec![
                "The little prince lives on a small star".to_owned(),
                "The little bird sings sweetly in the morning".to_owned(),
                "A magic flower blooms only under the moon".to_owned(),
                "The golden sun rises slowly over the ocean".to_owned(),
                "My mother cooks delicious food for me".to_owned(),
                "Where there is love, there is life".to_owned(),
                "Smile, and the world smiles with you".to_owned(),
                "Honesty is the best policy".to_owned(),
                "A gentle word can make a heavy heart light".to_owned(),
                "Stars can’t shine without darkness.".to_owned(),
            ],
            vec![
                "The teacher tells us a funny story".to_owned(),
                "I like to draw animals and flowers".to_owned(),
                "The world is brighter when you choose to care".to_owned(),
                "The shining stars brighten the dark night sky".to_owned(),
                "A small candle lights the entire dark room".to_owned(),
                "Light follows the darkest night".to_owned(),
                "Every flower blooms in its own time".to_owned(),
                "The morning sun paints the sky with golden light".to_owned(),
                "Kind hearts are the gardens where love grows".to_owned(),
                "Dreams are stars that guide us through the night".to_owned(),
            ],
        ];
        let mut sentence_file = get_app_data_dir(GAME_APP_NAME);
        sentence_file.push(WARSHIP_SENTENCES_FILE);
        let _ = sync_sentences_with_file(&mut level_sentences, sentence_file.as_path());

        GameSettings {
            level_letters,
            level_sentences,
            level_speeds: vec![
                (50., 80.),
                (80., 110.),
                (120., 150.),
                (150., 180.),
                (180., 220.),
            ],
            warship_fire_interval: vec![4., 3., 2., 1., 0.5],
            warship_gun_interval: vec![0.3, 0.2, 0.15, 0.08, 0.04],
            upgrade_scores: vec![2000, 10000, 28000, 50000],
            aircraft_count: vec![150, 300, 400, 500, 600],
            aircraft_intervals: vec![(3., 5.), (1.5, 3.), (1., 1.5), (0.8, 1.), (0.3, 1.)],
            bomb_intervals: vec![
                (150., 200.),
                (200., 250.),
                (250., 300.),
                (300., 400.),
                (400., 500.),
            ],
            shield_intervals: vec![
                (150., 200.),
                (200., 250.),
                (250., 300.),
                (300., 400.),
                (400., 500.),
            ],
            health_pack_intervals: vec![
                (150., 200.),
                (200., 250.),
                (250., 300.),
                (300., 400.),
                (400., 500.),
            ],
            shield_active_time: 30.,
            missile_speed: 1000.,
            flame_speed: 500.,
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
    texture.layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(150, 129),
        3,
        3,
        None,
        None,
    ));

    let mut data_file = get_app_data_dir(GAME_APP_NAME);
    data_file.push(PLAYERS_DATA_FILE);
    if Path::new(&data_file).exists() {
        fs::read_to_string(&data_file)
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
        if let Err(e) = fs::write(&data_file, json.as_bytes()) {
            error!("Failed to save player data: {}", e);
        }
    }
}

fn resolve_assets_path() -> String {
    let dev_path = PathBuf::from("assets");
    if dev_path.exists() {
        return dev_path.display().to_string();
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            // exe/.../MacOS/
            let candidate = parent
                .parent() // Contents
                .map(|c| c.join("Resources").join("assets"))
                .unwrap();
            if candidate.exists() {
                return candidate.display().to_string();
            }
        }
    }

    panic!("找不到 assets 目录！");
}