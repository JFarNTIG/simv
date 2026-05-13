use std::cmp::Ordering;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::body::{Body, Force};

/// Represents the physical universe.
/// Contains bodies and physics settings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Universe {
    /// Gravitational constant.
    /// The value of G in the formula F = G m_1 m_2 / r^2.
    pub grav_const: f64,

    /// Time scale.
    pub time_scale: f64,

    /// Physics bodies.
    pub bodies: Vec<Body>,

    /// If true, no physics ticks occur.
    #[serde(default)]
    pub paused: bool,
}

impl Universe {
    pub fn get_closest_body_to_point(&self, point: &DVec2) -> Option<usize> {
        self.bodies.iter().enumerate().min_by(|&a, &b| {
            let a_dist = a.1.pos.distance(*point);
            let b_dist = b.1.pos.distance(*point);

            if a_dist < b_dist {
                return Ordering::Less;
            } else if a_dist > b_dist {
                return Ordering::Greater;
            } else {
                return Ordering::Equal;
            }
        }).and_then(|a| {Some(a.0)})
    }

    pub fn calc_body_forces(&self) -> Vec<Vec<Force>> {
        let mut body_forces: Vec<Vec<Force>> = Vec::new();
        body_forces.reserve(self.bodies.len());

        for i in 0..self.bodies.len() {
            let body1 = &self.bodies[i];
            let mut forces: Vec<Force> = Vec::new();
    
            if !body1.immovable {
                for j in 0..self.bodies.len() {
                    if i == j { continue };
        
                    let body2 = &self.bodies[j];
        
                    forces.push(Force{
                        force: body1.calc_grav_force_from(self, body2),
                        from: j,
                    });
                }
            }

            body_forces.push(forces);
        }

        body_forces
    }

    pub fn physics_tick(&mut self, dt: f64) {
        for body in self.bodies.iter_mut() {
            body.pos += body.vel * dt + 0.5 * body.last_accel * dt * dt;
        }

        let body_forces: Vec<Vec<Force>> = self.calc_body_forces();

        for (i, body) in self.bodies.iter_mut().enumerate() {
            let forces: &[Force] = &body_forces[i];
    
            let mut total_force: DVec2 = dvec2(0.0, 0.0);
    
            forces.iter().for_each(|force| {
                total_force += force.force;
            });
    
            let new_accel = total_force / body.mass;
    
            // Update velocity
            body.vel += 0.5 * (body.last_accel + new_accel) * dt;
    
            body.last_accel = new_accel;

            body.forces = forces.to_vec();
        }
    }

    pub fn get_potential_energy(&self, body: &Body) -> f64 {
        let mut total_pe: f64 = 0.0;

        for force in &body.forces {
            let other = self.bodies.get(force.from).unwrap();

            let pe = -force.force.length() * body.pos.distance(other.pos);

            total_pe += pe;
        }

        total_pe
    }

    pub fn new() -> Universe {
        Universe { grav_const: 1.0, time_scale: 1.0, bodies: Vec::new(), paused: false }
    }
}