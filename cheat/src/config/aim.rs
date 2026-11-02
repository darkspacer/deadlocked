use std::{collections::HashMap, ops::RangeInclusive};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use shared::{Bones, Weapon};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::cs2::key_codes::KeyCode;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponConfig {
    pub aimbot: AimbotConfig,
    pub rcs: RcsConfig,
    pub triggerbot: TriggerbotConfig,
}

impl WeaponConfig {
    pub fn enabled(enabled: bool) -> Self {
        let aimbot = AimbotConfig {
            enable_override: enabled,
            ..Default::default()
        };
        Self {
            aimbot,
            rcs: RcsConfig::default(),
            triggerbot: TriggerbotConfig::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub mode: KeyMode,
    pub target_friendlies: bool,
    pub distance_adjusted_fov: bool,
    pub start_bullet: i32,
    pub visibility_check: bool,
    pub flash_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub inertia: f32,
    /// Move the crosshair along a human-like windmouse path instead of straight jumps.
    pub windmouse: bool,
    /// Distance from the target where the wind force starts flattening the curve.
    pub windmouse_wind: f32,
    /// Maximum strength of the random gust deviations along the path.
    pub windmouse_gustiness: f32,
    /// Below this distance (in input units) the pointer walks straight onto the target.
    pub windmouse_min_radius: f32,
    /// Above this distance (in input units) big linear steps are taken first.
    pub windmouse_max_radius: f32,
    /// Milliseconds between individual mouse reports while tracing the path.
    pub windmouse_step_delay_ms: u64,
    /// Seconds of target motion to lead when aiming.
    pub prediction_time: f32,
    pub bones: Vec<Bones>,
    pub targeting_mode: TargetingMode,
    pub penetration_check: bool,
    pub minimum_penetration_damage: u32,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: true,
            mode: KeyMode::Hold,
            target_friendlies: false,
            distance_adjusted_fov: true,
            start_bullet: 0,
            visibility_check: true,
            flash_check: true,
            fov: 2.5,
            smooth: 5.0,
            inertia: 1.0,
            windmouse: false,
            windmouse_wind: 4.0,
            windmouse_gustiness: 6.0,
            windmouse_min_radius: 15.0,
            windmouse_max_radius: 80.0,
            windmouse_step_delay_ms: 3,
            prediction_time: 0.05,
            bones: vec![
                Bones::Head,
                Bones::Neck,
                Bones::Spine4,
                Bones::Spine3,
                Bones::Spine2,
                Bones::Spine1,
                Bones::Hip,
            ],
            targeting_mode: TargetingMode::Fov,
            penetration_check: false,
            minimum_penetration_damage: 50,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RcsConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub strength: Vec2,
}

impl Default for RcsConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            strength: Vec2::splat(0.5),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, EnumIter, Display)]
pub enum KeyMode {
    Hold,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, Display)]
pub enum TargetingMode {
    #[strum(to_string = "FOV")]
    Fov,
    Distance,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub delay: RangeInclusive<u64>,
    pub shot_duration: u64,
    pub mode: KeyMode,
    pub flash_check: bool,
    pub scope_check: bool,
    pub velocity_check: bool,
    pub velocity_threshold: f32,
    pub head_only: bool,
    pub penetration_check: bool,
    pub minimum_penetration_damage: u32,
}

impl Default for TriggerbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            delay: 100..=200,
            shot_duration: 200,
            mode: KeyMode::Hold,
            flash_check: true,
            scope_check: true,
            velocity_check: true,
            velocity_threshold: 100.0,
            head_only: false,
            penetration_check: false,
            minimum_penetration_damage: 50,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrenadeAlignConfig {
    pub enabled: bool,
    pub hotkey: KeyCode,
    pub fov: f32,
    pub smooth: f32,
}

impl Default for GrenadeAlignConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            hotkey: KeyCode::None,
            fov: 10.0,
            smooth: 25.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimConfig {
    pub aimbot_hotkey: KeyCode,
    pub triggerbot_hotkey: KeyCode,
    pub grenade_align: GrenadeAlignConfig,
    pub global: WeaponConfig,
    pub weapons: HashMap<Weapon, WeaponConfig>,
}

impl Default for AimConfig {
    fn default() -> Self {
        let mut weapons = HashMap::new();
        for weapon in Weapon::iter() {
            weapons.insert(weapon, WeaponConfig::default());
        }

        Self {
            aimbot_hotkey: KeyCode::Mouse5,
            triggerbot_hotkey: KeyCode::Mouse4,
            grenade_align: GrenadeAlignConfig::default(),
            global: WeaponConfig::enabled(true),
            weapons,
        }
    }
}
