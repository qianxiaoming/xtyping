use bevy::color::Color;
use bevy::prelude::{Component, Deref, DerefMut, Entity, Event, Resource, Timer};
use crate::gaming::spawn::{AircraftSpawnState, BombSpawnState, HealthPackSpawnState, ShieldSpawnState};
use crate::PlayState;

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

#[derive(PartialEq, Clone, Copy)]
pub enum GameRole {
    Player,
    Enemy
}

/// 玩家/敌方的血条
#[derive(Component, Clone)]
pub struct HealthBar {
    pub role: GameRole,
    pub value: u16,
}

pub const HEALTH_MAX_VALUE: u16 = 100;

#[derive(Resource, Default)]
pub struct FlyingUnitCounter {
    pub destroyed: usize,
    pub missed: usize,
    pub bomb: usize,
    pub shield: usize,
    pub health_pack: usize,
}

/// Splash动画元素
#[derive(Component)]
pub struct SplashTextRow {
    pub timer: Timer,
}

/// 玩家的战斗机
#[derive(Component)]
pub struct FighterJet {
    pub protected: bool,
    pub protect_since: f32,
}

pub const FIGHTER_JET_MARGIN: f32 = 80.0;
pub const FIGHTER_JET_SCALE: f32 = 0.3;
pub const FIGHTER_JET_SIZE: f32 = 300.;

pub const TARGET_LETTER_SIZE: f32 = 32.;
pub const TARGET_LETTER_COLOR: Color = Color::srgb_u8(88, 251, 254);

pub const CHECKPOINT_LETTER_SIZE: f32 = 52.;
pub const CHECKPOINT_LETTER_COLOR: Color = Color::srgb_u8(245, 53, 53);

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum FlyingUnitKind {
    Aircraft,
    Bomb,
    Shield,
    HealthPack
}

#[derive(Component)]
pub struct FlyingUnitText(pub FlyingUnitKind);

pub trait FlyingUnitTrait {
    type SpawnState;
    fn kind() -> FlyingUnitKind;
}

#[derive(Component)]
pub struct FlyingUnit {
    pub route: i32,
    pub letter: char,
    pub speed: f32,
    pub kind: FlyingUnitKind,
}

#[derive(Component, Default)]
pub struct Aircraft {
    pub ready: bool,   // 是否准备发射
    pub fire_pos: f32, // 发射的坐标位置
    pub flame: Option<Entity> // 发射的火球
}

impl FlyingUnitTrait for Aircraft {
    type SpawnState = AircraftSpawnState;
    fn kind() -> FlyingUnitKind { FlyingUnitKind::Aircraft }
}

pub const AIRCRAFT_KIND: i32 = 3;
pub const AIRCRAFT_SIZE: f32 = 300.;

#[derive(Component, Default)]
pub struct Bomb;

impl FlyingUnitTrait for Bomb {
    type SpawnState = BombSpawnState;
    fn kind() -> FlyingUnitKind { FlyingUnitKind::Bomb }
}

#[derive(Component, Default)]
pub struct HealthPack;

impl FlyingUnitTrait for HealthPack {
    type SpawnState = HealthPackSpawnState;
    fn kind() -> FlyingUnitKind { FlyingUnitKind::HealthPack }
}

pub const HEALTH_PACK_RESTORE: u16 = 10;

#[derive(Component)]
pub struct EquipmentEffect {
    pub timer: Timer,
    pub duration: f32,
}

#[derive(Event)]
pub struct HealthPackApplyEvent;

#[derive(Component, Default)]
pub struct Shield;

impl FlyingUnitTrait for Shield {
    type SpawnState = ShieldSpawnState;
    fn kind() -> FlyingUnitKind { FlyingUnitKind::Shield }
}

#[derive(Component)]
pub struct MissText(pub Timer);

/// 玩家发射的导弹
#[derive(Component)]
pub struct Missile {
    pub speed: f32,
    pub target: Entity,
    pub kind: FlyingUnitKind
}

/// 敌机发射的火焰武器
#[derive(Component)]
pub struct Flame {
    pub speed: f32
}

#[derive(Component, Deref, DerefMut)]
pub struct Explosion(pub Timer);

pub const EXPLOSION_SHEET_MAX_INDEX: usize = 8;

#[derive(Event)]
pub struct BombExplodedEvent;

#[derive(Event)]
pub struct ShieldActivatedEvent;

#[derive(Event)]
pub struct UpdateHealthBarEvent(pub u16);

#[derive(Resource, Default)]
pub struct LastPlayState(pub PlayState);

#[derive(Resource, Default)]
pub struct CheckpointTimer(pub Timer);

#[derive(Resource, Default)]
pub struct GameSaveTimer(pub Timer);

#[derive(Component)]
pub struct SpaceWarship;

#[derive(Component)]
pub struct CheckpointLetter {
    pub letter: char,
    pub index: usize,
    pub destroyed: bool,
}

#[derive(Resource, Default)]
pub struct CheckpointLetters {
    pub letters: Vec<char>,
    pub current: usize,
}