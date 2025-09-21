use bevy::color::Color;
use bevy::prelude::{Component, Timer};
use crate::gaming::spawn::{AircraftSpawnState, BombSpawnState, HealthPackSpawnState, ShieldSpawnState};

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

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum UnitKind {
    FighterJet,
    Aircraft,
    Bomb,
    Shield,
    HealthPack
}

pub trait FlyingUnitTrait {
    type SpawnState;
    fn kind() -> UnitKind;
}

#[derive(Component)]
pub struct FlyingUnit {
    pub route: i32,
    pub letter: char,
    pub speed: f32,
    pub kind: UnitKind,
}

#[derive(Component)]
pub struct Aircraft;

impl FlyingUnitTrait for Aircraft {
    type SpawnState = AircraftSpawnState;
    fn kind() -> UnitKind { UnitKind::Aircraft }
}

pub const AIRCRAFT_KIND: i32 = 3;
pub const AIRCRAFT_SIZE: f32 = 300.;

#[derive(Component, Default)]
pub struct Bomb;

impl FlyingUnitTrait for Bomb {
    type SpawnState = BombSpawnState;
    fn kind() -> UnitKind { UnitKind::Bomb }
}

#[derive(Component, Default)]
pub struct HealthPack;

impl FlyingUnitTrait for HealthPack {
    type SpawnState = HealthPackSpawnState;
    fn kind() -> UnitKind { UnitKind::HealthPack }
}

#[derive(Component, Default)]
pub struct Shield;

impl FlyingUnitTrait for Shield {
    type SpawnState = ShieldSpawnState;
    fn kind() -> UnitKind { UnitKind::Shield }
}

#[derive(Component)]
pub struct MissText(pub Timer);

#[derive(Component)]
pub struct Missile;