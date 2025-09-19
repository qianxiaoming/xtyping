use bevy::color::Color;
use bevy::prelude::{Component, Timer};

/// 游戏时间显示
#[derive(Component)]
pub struct GameTime {
    pub start_time: f64,
    pub last_second: u64,
}

/// 玩家等级提升进度条
#[derive(Component)]
pub struct LevelProgress;

/// 玩家等级星星图片
#[derive(Component)]
pub struct LevelStarImage;

/// 玩家当前得分文本
#[derive(Component)]
pub struct PlayerScore;

/// 玩家/敌方的血条
#[derive(Component)]
pub struct HealthBar(pub bool);

pub const HEALTH_BAR_LEN: u16 = 100;

/// 敌方数量统计文本
#[derive(Component)]
pub struct EnemyCounter;

/// 炸弹数量统计文本
#[derive(Component)]
pub struct BombCounter;

/// 血包数量统计文本
#[derive(Component)]
pub struct HealthPackCounter;

/// 护盾数量统计文本
#[derive(Component)]
pub struct ShieldCounter;

/// Splash动画元素
#[derive(Component)]
pub struct SplashTextRow {
    pub timer: Timer,
}

/// 玩家的战斗机
#[derive(Component)]
pub struct FighterJet;

pub const FIGHTER_JET_MARGIN: f32 = 80.0;
pub const FIGHTER_JET_SCALE: f32 = 0.3;
pub const FIGHTER_JET_SIZE: f32 = 300.;

pub const TARGET_LETTER_SIZE: f32 = 88.;
pub const TARGET_LETTER_COLOR: Color = Color::srgb_u8(88, 251, 254);

#[derive(Component)]
pub struct FlyingUnit {
    pub route: i32,
    pub letter: char,
    pub speed: f32,
}

#[derive(Component)]
pub struct Aircraft;

pub const AIRCRAFT_KIND: i32 = 3;
pub const AIRCRAFT_SIZE: f32 = 300.;
// pub const AIRCRAFT_COLORS: [Color; 3] = [
//     Color::srgb_u8(66, 201, 36),
//     Color::srgb_u8(220, 31, 11),
//     Color::srgb_u8(255, 222, 0)
// ];

#[derive(Component, Default)]
pub struct Bomb;

#[derive(Component, Default)]
pub struct HealthPack;

#[derive(Component, Default)]
pub struct Shield;