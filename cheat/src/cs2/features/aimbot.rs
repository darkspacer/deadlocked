use std::time::{Duration, Instant};

use glam::{Vec2, vec2};
use shared::WeaponClass;

use crate::{
    config::{
        Config,
        aim::{AimbotConfig, KeyMode},
    },
    cs2::{CS2, entity::player::Player, input::Input, key_codes::KeyCode},
    math::vec2_clamp,
    os::{
        mouse::Mouse,
        windmouse::{WindMouseSettings, sleep_until, wind_mouse_path},
    },
};

/// Worst-case time budget for tracing a single windmouse path. If the path
/// can't be finished within this window (e.g. the aimbot hotkey is released),
/// the remainder is sent in one step so the crosshair still lands on target.
const WINDMOUSE_BUDGET: Duration = Duration::from_millis(120);

#[derive(Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
}

impl Aimbot {
    fn reset(&mut self) {
        self.inertia = Vec2::ZERO;
    }
}

fn windmouse_settings(config: &AimbotConfig) -> WindMouseSettings {
    WindMouseSettings {
        wind: config.windmouse_wind,
        gustiness: config.windmouse_gustiness,
        min_radius: config.windmouse_min_radius,
        max_radius: config.windmouse_max_radius,
        event_delay: Duration::from_millis(config.windmouse_step_delay_ms),
    }
}

/// Moves the pointer along a human-like windmouse trajectory over `total`
/// input units, pacing the reports with `event_delay` and bailing out early
/// when the aimbot gets deactivated mid-path. Returns whether the aimbot
/// hotkey is still active after the trace finished.
fn mouse_move_windmouse(
    mouse: &mut Mouse,
    input: &Input,
    total: Vec2,
    settings: &WindMouseSettings,
    hotkey: KeyCode,
    mode: KeyMode,
    active: &mut bool,
) -> bool {
    let path = wind_mouse_path(total, settings);
    if path.is_empty() {
        return true;
    }

    let deadline = Instant::now() + WINDMOUSE_BUDGET;
    for step in path {
        if step != Vec2::ZERO {
            mouse.move_rel(step);
        }
        sleep_until(deadline.min(Instant::now() + settings.event_delay));

        // stop the trace as soon as the aimbot key/mode state goes inactive
        if !CS2::check_hotkey(input, mode, hotkey, active) {
            return false;
        }
    }

    true
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled {
            self.aim.reset();
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.aim.active) {
            self.aim.reset();
            return false;
        }

        let Some(target) = &self.target.player else {
            return false;
        };

        if !target.is_valid(self) {
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            return false;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            return false;
        }

        let target_angle = self.target.angle;
        let view_angles = local_player.view_angles(self);

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);
        if !sensitivity.is_finite() || sensitivity <= f32::EPSILON {
            self.aim.reset();
            return false;
        }

        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / (config.smooth + 1.0).clamp(1.0, 20.0);

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        self.aim.inertia += (mouse_angles - self.aim.inertia) * alpha;

        if config.windmouse {
            let settings = windmouse_settings(config);
            let mode = config.mode;
            let still_active = mouse_move_windmouse(
                mouse,
                &self.input,
                self.aim.inertia,
                &settings,
                hotkey,
                mode,
                &mut self.aim.active,
            );
            if !still_active {
                self.aim.reset();
                return false;
            }
        } else {
            mouse.move_rel(self.aim.inertia);
        }

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}
