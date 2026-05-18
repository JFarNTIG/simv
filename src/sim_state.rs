use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use macroquad::prelude::*;

use crate::actions::AppAction;
use crate::universe::Universe;
use crate::ringarray::RingArray;
use macroquad::camera::Camera2D;
use macroquad::math::DVec2;

/// Maximum number of points in a trail per body.
const TRAIL_ARRAY_CAPACITY_PER_BODY: usize = 1000;

pub struct SimConfig {
    /// Whether trails should be rendered.
    pub render_trails: bool,

    /// Whether body name labels should be rendered,
    /// e.g. `Jupiter`.
    pub render_labels: bool,

    /// Whether to display mass for
    /// selected bodies.
    pub render_mass: bool,

    /// Whether to display velocity for
    /// selected bodies.
    pub render_vel: bool,

    /// Whether to display kinetic energy for
    /// selected bodies.
    pub render_ke: bool,

    /// Whether to display potential energy for
    /// selected bodies.
    pub render_pe: bool,

    /// Whether to display the Lagrangian for
    /// selected bodies.
    pub render_lagrangian: bool,

    /// Whether to display forces acting on
    /// the selected body.
    pub render_forces: bool,
}

impl SimConfig {
    pub fn new() -> SimConfig {
        SimConfig {
            render_trails: true,
            render_labels: true,
            render_mass: false,
            render_vel: false,
            render_ke: true,
            render_pe: true,
            render_lagrangian: false,
            render_forces: false,
        }
    }
}

/// Represents the simulator program state.
pub struct SimState {
    /// Universe containing bodies and physics
    /// simulation settings
    pub uni: Universe,

    /// Active trails. Consist of a vec of one ring array
    /// per body.
    pub trails: Vec<RingArray<DVec2>>,

    /// Simulator config
    pub config: SimConfig,

    pub action_queue: VecDeque<AppAction>,

    pub zoom: f32,
    
    pub view_center: Vec2,

    pub focusing: bool,
}

impl SimState {
    pub fn load_state_from_file(&mut self, file: &Path) -> Option<()> {
        let contents = fs::read_to_string(file);
        if let Err(err) = contents {
            println!("Error reading sim file: {}", err);
            return None;
        }

        let contents = contents.unwrap();

        let universe = serde_json::from_str(&contents);
        if let Err(err) = universe {
            println!("Error parsing universe JSON: {}", err);
            return None;
        }

        let universe = universe.unwrap();
    
        self.uni = universe;
        self.trails = self.uni.bodies.iter()
            .map(|_| RingArray::new(TRAIL_ARRAY_CAPACITY_PER_BODY))
            .collect();
    
        Some(())
    }

    pub fn get_camera(&self) -> Camera2D {
        let aspect_ratio = screen_width() / screen_height();
        Camera2D {
            zoom: vec2(self.zoom, -self.zoom * aspect_ratio),
            target: self.view_center,
            ..Default::default()
        }
    }
}