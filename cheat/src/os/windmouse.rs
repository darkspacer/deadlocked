//! Human-like mouse path generation, based on the WindMouse algorithm described in
//! <https://ben.land/post/2021/04/25/windmouse-human-mouse-movement/>.
//!
//! The idea: a point is "blown" toward the target by a constant wind force plus random
//! gusts while it stays far away. As it approaches the target the wind and gusts shrink,
//! so the resulting path is curved and noisy at long distances but converges smoothly.

use std::time::{Duration, Instant};

use glam::{Vec2, vec2};
use rand::{Rng, RngExt};
use rand_distr::{Distribution, Normal};

/// Distance from the target where the curve starts to flatten out (in input units).
const DEFAULT_WIND: f32 = 4.0;
/// Maximum strength of the random deviations (in input units).
const DEFAULT_GUSTYNESS: f32 = 6.0;
/// Below this distance the pointer moves in straight steps without wind or gusts.
const DEFAULT_MIN_RADIUS: f32 = 15.0;
/// Above this distance the pointer takes big linear steps toward the target.
const DEFAULT_MAX_RADIUS: f32 = 80.0;
/// Default number of milliseconds between individual mouse events.
const DEFAULT_EVENT_DELAY_MS: u64 = 3;

#[derive(Clone, Copy, Debug)]
pub struct WindMouseSettings {
    pub wind: f32,
    pub gustiness: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub event_delay: Duration,
}

impl Default for WindMouseSettings {
    fn default() -> Self {
        Self {
            wind: DEFAULT_WIND,
            gustiness: DEFAULT_GUSTYNESS,
            min_radius: DEFAULT_MIN_RADIUS,
            max_radius: DEFAULT_MAX_RADIUS,
            event_delay: Duration::from_millis(DEFAULT_EVENT_DELAY_MS),
        }
    }
}

/// Samples a gaussian deviation, falling back to a uniform sample if the
/// distribution can't be constructed (e.g. non-finite standard deviation).
fn sample_normal(rng: &mut impl Rng, mean: f32, deviation: f32) -> f32 {
    if !deviation.is_finite() || deviation <= f32::EPSILON {
        return mean;
    }
    match Normal::new(mean, deviation) {
        Ok(normal) => normal.sample(rng),
        Err(_) => rng.random_range(-deviation..deviation),
    }
}

/// Generates the sequence of relative movements that carry the pointer along a
/// human-like windmouse trajectory over `total` input units.
///
/// The returned steps sum up (modulo rounding) to `total`, so applying all of
/// them moves the crosshair onto the requested destination.
pub fn wind_mouse_path(total: Vec2, settings: &WindMouseSettings) -> Vec<Vec2> {
    let mut path = Vec::new();

    if !total.is_finite() {
        return path;
    }

    let mut rng = rand::rng();
    let mut x = total.x;
    let mut y = total.y;

    // large overshoots are walked toward the target in straight chunks first
    let distance = Vec2::new(x, y).length();
    if distance > settings.max_radius && distance > f32::EPSILON {
        let step_x = x / distance * settings.max_radius;
        let step_y = y / distance * settings.max_radius;

        path.push(vec2(step_x.round(), step_y.round()));
        x -= step_x;
        y -= step_y;
    }

    loop {
        let distance = Vec2::new(x, y).length();

        if distance < 1.0 {
            break;
        }

        // no wind or gusts when very close to the target, just walk onto it
        if distance < settings.min_radius {
            path.push(vec2(x.round(), y.round()));
            break;
        }

        // both wind and gust magnitude decay as the target gets closer
        let wind_mag = (settings.wind * (settings.min_radius / distance)).max(1.0);
        let gust_mag = settings.gustiness * (settings.min_radius / distance);

        let gust = vec2(
            sample_normal(&mut rng, 0.0, gust_mag),
            sample_normal(&mut rng, 0.0, gust_mag),
        );

        let ux = x / distance;
        let uy = y / distance;

        let dx = ux * wind_mag + gust.x;
        let dy = uy * wind_mag + gust.y;

        // never move against the general direction of the target
        let next_x = if dx.signum() != ux.signum() { 0.0 } else { dx };
        let next_y = if dy.signum() != uy.signum() { 0.0 } else { dy };

        let next_x = next_x.clamp(-distance, distance);
        let next_y = next_y.clamp(-distance, distance);

        x -= next_x;
        y -= next_y;

        path.push(vec2(next_x.round(), next_y.round()));

        // safety net against pathological oscillation
        if path.len() >= 1024 {
            let remainder = vec2(x.round(), y.round());
            if remainder != Vec2::ZERO {
                path.push(remainder);
            }
            break;
        }
    }

    path
}

/// Sleeps until the given instant, used to pace out the generated path so the
/// individual reports arrive like real hardware input would.
pub fn sleep_until(deadline: Instant) {
    let now = Instant::now();
    if deadline > now {
        std::thread::sleep(deadline - now);
    }
}
