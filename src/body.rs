use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::universe::*;

/// Represents a force on a body.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Force {
    pub force: DVec2,
    pub from: usize,
}

/// Represents one physical body, e.g.
/// a planet, moon, or satellite.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body {
    /// Absolute position of the body in space.
    pub pos: DVec2,

    /// Absolute velocity of the body.
    pub vel: DVec2,

    /// Radius of the body.
    /// Affects the size of the body when drawn on screen.
    pub radius: f64,

    /// Mass of the body.
    pub mass: f64,

    /// Name of the body.
    pub name: String,

    /// Color of the body.
    /// Affects the color of the body when drawn on screen.
    pub color: Vec4,

    /// If true, the body is not affected by
    /// forces.
    pub immovable: bool,

    /// Forces that acted on the body during the last
    /// physics tick.
    #[serde(default)]
    pub forces: Vec<Force>,

    /// Absolute acceleration of the body during the last
    /// physics tick.
    #[serde(default)]
    pub last_accel: DVec2,
}

impl Body {
    /// Compute the gravitational force exerted by another body
    /// on this body.
    pub fn calc_grav_force_from(&self, uni: &Universe, other: &Body) -> DVec2 {
        let square_dist: f64 = self.pos.distance_squared(other.pos);
        let force_magnitude: f64 = uni.grav_const * self.mass * other.mass / square_dist;
        let angle: f64 = (other.pos - self.pos).to_angle();
        force_magnitude * DVec2::from_angle(angle)
    }

    /// Compute the kinetic energy of the body.
    pub fn get_kinetic_energy(&self) -> f64 {
        1.0 / 2.0 * self.mass * self.vel.length_squared()
    }
}