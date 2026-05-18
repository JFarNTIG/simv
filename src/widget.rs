use std::{thread, sync::{Mutex, Arc}, collections::VecDeque};
use rfd::FileDialog;
use macroquad::{prelude::*, ui::{root_ui, hash}};

use crate::{sim_state::SimState, actions::AppAction, drawing::{draw_selection_box, draw_arrow}, body::Body};

pub type Task = Box<dyn FnOnce(&Arc<Mutex<SimState>>) + Send>;

const BODY_SELECT_CLICK_DISTANCE_THRESHOLD: f32 = 50.0;

#[derive(Copy, Clone)]
pub enum WidgetInput {
    LeftClick(Vec2),
}

#[derive(PartialEq)]
pub enum WidgetType {
    MenuBar,
    StatusOverlay,
    PhysicsOverlay,
    SettingsWindow,
    AboutWindow,
}

/// Trait representing a UI widget.
/// 
/// UI widgets have access to the camera, simulation state and
/// the task queue.
pub trait Widget {
    /// Render the widget during the screen space render tick.
    /// 
    /// Parameters:
    ///     - `camera` The active camera.
    ///     - `state` The simulation state.
    ///     - `task_queue` The task queue.
    /// 
    /// Returns:
    ///     - `true` if the widget was closed and should be
    ///       removed from the widget stack, `false` if the widget
    ///       should remain.
    fn render(&self, camera: &Camera2D, state: &Arc<Mutex<SimState>>, task_queue: &Arc<Mutex<VecDeque<Task>>>) -> bool;

    /// Process an input event.
    /// If the event was not processed by the app root, then it will
    /// be passed to each widget in reverse draw order until a widget
    /// processes the event.
    /// 
    /// Parameters:
    ///     - `event` The input event.
    /// 
    /// Returns:
    ///     - `true` if the event was processed by this widget,
    ///       `false` otherwise.
    fn input_event(&mut self, state: &Arc<Mutex<SimState>>, event: WidgetInput) -> bool;

    fn get_type(&self) -> WidgetType; 
}

pub struct MenuBar;

impl Widget for MenuBar {
    fn render(&self, _camera: &Camera2D, state: &Arc<Mutex<SimState>>, task_queue: &Arc<Mutex<VecDeque<Task>>>) -> bool {
        let mut state = state.lock().unwrap();
        
        root_ui().window(hash!(), vec2(0.0, 0.0), vec2(screen_width(), 34.0), |ui| {
            ui.label(vec2(4.0, 4.0), "Sim");

            if ui.button(vec2(50.0, 4.0), "Load") {
                let queue_handle = task_queue.clone();
                // RFD is blocking, so we spawn a thread to keep the UI responsive
                thread::spawn(move || {
                    if let Some(path) = FileDialog::new().add_filter("Sim definitions (*.json)", &["json"]).pick_file() {
                        queue_handle.lock().unwrap().push_back(Box::new(move |s| {
                            let mut state_guard = s.lock().unwrap();
                            let _ = state_guard.load_state_from_file( &path);
                        }));
                    }
                });
            }

            if ui.button(vec2(100.0, 4.0), "Settings") {
                state.action_queue.push_back(AppAction::OpenSettingsWindow);
            }

            if ui.button(vec2(180.0, 4.0), "About") {
                state.action_queue.push_back(AppAction::OpenAboutWindow);
            }

            if ui.button(vec2(240.0, 4.0), "Quit") {
                state.action_queue.push_back(AppAction::Quit);
            }
        });

        false
    }

    fn input_event(&mut self, _state: &Arc<Mutex<SimState>>, _event: WidgetInput) -> bool {
        false
    }

    fn get_type(&self) -> WidgetType {
        return WidgetType::MenuBar;
    }
}

pub struct StatusOverlay;

impl Widget for StatusOverlay {
    fn render(&self, _camera: &Camera2D, state: &Arc<Mutex<SimState>>, _task_queue: &Arc<Mutex<VecDeque<Task>>>) -> bool {
        let state = state.lock().unwrap();

        let body_count = state.uni.bodies.len();
        draw_text(&format!("Bodies: {}", body_count), 20.0, 60.0, 18.0, WHITE);

        if state.uni.paused {
            draw_text("PAUSED", 20.0, 90.0, 30.0, RED);
        }

        false
    }

    fn input_event(&mut self, _state: &Arc<Mutex<SimState>>, _event: WidgetInput) -> bool {
        false
    }

    fn get_type(&self) -> WidgetType {
        return WidgetType::StatusOverlay;
    }
}

pub struct PhysicsOverlays {
    pub selected_body: Option<usize>,
}

impl Widget for PhysicsOverlays {
    fn render(&self, camera: &Camera2D, state: &Arc<Mutex<SimState>>, _task_queue: &Arc<Mutex<VecDeque<Task>>>) -> bool {
        let state = &mut *state.lock().unwrap();

        // Draw trails
        if state.config.render_trails {
            for (i, trail_buffer) in state.trails.iter().enumerate() {
                let mut is_selected = false;
                if let Some(selected_idx) = self.selected_body {
                    if selected_idx == i {
                        is_selected = true;
                    }
                }

                // If the body is selected, draw a red trail
                // so that the trail stands out
                let trail_color = match is_selected {
                    true => RED,
                    false => WHITE,
                };

                let count = trail_buffer.len();
                if count < 2 { continue; }
    
                let points: Vec<Vec2> = trail_buffer.iter()
                    .map(|p| camera.world_to_screen(p.as_vec2()))
                    .collect();
    
                for i in 0..points.len() - 1 {
                    // Draw the trail as fading at the end only if
                    // the body is not selected; if the body is selected,
                    // don't draw with alpha, again so that the trail
                    // stands out
                    let alpha = 0.5 * (i as f32 / count as f32);
                    let segment_color = match is_selected {
                        true => trail_color,
                        false => trail_color.with_alpha(alpha),
                    };
                    
                    draw_line(
                        points[i].x, points[i].y, 
                        points[i+1].x, points[i+1].y, 
                        1.0, 
                        segment_color
                    );
                }
            }
        }

        // Draw labels
        if state.config.render_labels {
            for body in &state.uni.bodies {
                let screen_pos = camera.world_to_screen(body.pos.as_vec2());
                draw_text(&body.name, screen_pos.x + 10., screen_pos.y - 4., 16., WHITE);
            }
        }

        // If a body is selected
        // Render selection box, kinetic energy and potential energy
        if let Some(body_idx) = self.selected_body {
            if let Some(body) = state.uni.bodies.get(body_idx) {
                // If focus mode is enabled, set the camera view center to the selected body.
                if state.focusing {
                    state.view_center = body.pos.as_vec2();
                }

                let screen_pos = camera.world_to_screen(body.pos.as_vec2());

                draw_selection_box(screen_pos.x, screen_pos.y, 16.0, 1.0, WHITE);

                // We will add each info line to a vec, depending on which info lines
                // are enabled in settings. Then we will render the entire vec at once.
                let mut body_info: Vec<String> = Vec::new();

                // Sim definitions are using km, so to convert whatever units these are to J
                // we need to multiply by a factor of (m per km)^2
                let kinetic_energy = body.get_kinetic_energy() * (1000.0f64).powi(2);
                let potential_energy = state.uni.get_potential_energy(body) * (1000.0f64).powi(2);

                if state.config.render_mass {
                    body_info.push(format!("M = {:.2e} kg", body.mass));
                }

                if state.config.render_vel {
                    body_info.push(format!("|v| = {:.2} km/s", body.vel.length()));
                }

                if state.config.render_ke {
                    body_info.push(format!("KE = {:.2e} J", kinetic_energy));
                }

                if state.config.render_pe {
                    body_info.push(format!("PE = {:.2e} J", potential_energy));
                }

                if state.config.render_lagrangian {
                    body_info.push(format!("L = {:.2e} J", kinetic_energy - potential_energy));
                }

                for (i, info_line) in body_info.iter().enumerate() {
                    draw_text(info_line.as_str(), screen_pos.x + 10.0, screen_pos.y + 18.0 + 18.0 * i as f32, 16.0, WHITE);
                }

                // Render forces acting on the body
                if state.config.render_forces {
                    for force in &body.forces {
                        // Forces are in kN, need to convert to N
                        // with factor of 1000
                        let force_mag = force.force.length() * 1000.0f64;
                        let invert_y = Vec2::new(1.0, -1.0);
                        let force_screen = 60.0 * force.force.normalize().as_vec2() * invert_y;
                        let force_end_screen = screen_pos + force_screen;
                        let force_half_screen = screen_pos + 0.5 * force_screen;

                        let font_character_size = 16.0;
                        let font_character_width = font_character_size * 0.6;
                        let force_label1 = format!("{:.2e} N", force_mag);
                        let label1_width = force_label1.len() as f32 * font_character_width;

                        let other_body: Option<&Body> = state.uni.bodies.get(force.from);
                        let other_name: &str = match other_body {
                            Some(other_body) => &other_body.name,
                            None => "?"
                        };
                        let force_label2 = format!("from {}", other_name);
                        let label2_width = force_label2.len() as f32 * font_character_width;

                        draw_arrow(screen_pos.x, screen_pos.y, force_end_screen.x, force_end_screen.y, 1.0, YELLOW);
                        draw_text(&force_label1, force_half_screen.x - 4.0 - label1_width, force_half_screen.y - 20.0, font_character_size, YELLOW);
                        draw_text(&force_label2, force_half_screen.x - 4.0 - label2_width, force_half_screen.y + 0.0, font_character_size, YELLOW);
                    }
                }
            }
        }

        false
    }

    fn input_event(&mut self, state: &Arc<Mutex<SimState>>, _event: WidgetInput) -> bool {
        let state = state.lock().unwrap();

        match _event {
            WidgetInput::LeftClick(pos) => {
                let camera = &state.get_camera();
                let world_pos = camera.screen_to_world(pos).as_dvec2();
                let maybe_closest_body_idx = state.uni.get_closest_body_to_point(&world_pos);
                
                if let Some(closest_body_idx) = maybe_closest_body_idx {
                    let body = state.uni.bodies.get(closest_body_idx).unwrap();
                    let screen_pos = camera.world_to_screen(body.pos.as_vec2());

                    if screen_pos.distance(pos) < BODY_SELECT_CLICK_DISTANCE_THRESHOLD {
                        self.selected_body = Some(closest_body_idx);

                        return true;
                    }
                }

                self.selected_body = None;
            }
        }

        true
    }

    fn get_type(&self) -> WidgetType {
        return WidgetType::PhysicsOverlay;
    }
}

pub struct SettingsWindow;

impl Widget for SettingsWindow {
    fn render(&self, _camera: &Camera2D, state: &Arc<Mutex<SimState>>, _task_queue: &Arc<Mutex<VecDeque<Task>>>) -> bool {
        let state = &mut *state.lock().unwrap();
        let mut done = false;

        root_ui().window(hash!(), vec2(screen_width() / 2.0 - 200.0, screen_height() / 2.0 - 200.0), vec2(400.0, 400.0), |ui| {
            ui.label(None, "Settings");
            
            ui.checkbox(hash!(), "Render Trails", &mut state.config.render_trails);
            ui.checkbox(hash!(), "Render Labels", &mut state.config.render_labels);

            ui.label(None, "Selected Body Info");
            ui.checkbox(hash!(), "Show Mass", &mut state.config.render_mass);
            ui.checkbox(hash!(), "Show Velocity", &mut state.config.render_vel);
            ui.checkbox(hash!(), "Show Kinetic Energy", &mut state.config.render_ke);
            ui.checkbox(hash!(), "Show Potential Energy", &mut state.config.render_pe);
            ui.checkbox(hash!(), "Show Lagrangian", &mut state.config.render_lagrangian);
            ui.checkbox(hash!(), "Show Forces", &mut state.config.render_forces);

            if ui.button(vec2(180.0, 370.0), "Done") {
                done = true;
            }
        });

        done
    }

    fn input_event(&mut self, _state: &Arc<Mutex<SimState>>, _event: WidgetInput) -> bool {
        false
    }

    fn get_type(&self) -> WidgetType {
        return WidgetType::SettingsWindow;
    }
}

pub struct AboutWindow;

impl Widget for AboutWindow {
    fn render(&self, _camera: &Camera2D, state: &Arc<Mutex<SimState>>, _task_queue: &Arc<Mutex<VecDeque<Task>>>) -> bool {
        let _state = &mut *state.lock().unwrap();
        let mut done = false;

        root_ui().window(hash!(), vec2(screen_width() / 2.0 - 200.0, screen_height() / 2.0 - 200.0), vec2(400.0, 400.0), |ui| {
            ui.label(None, "Visual Physics");
            ui.label(None, format!("Version {}", env!("CARGO_PKG_VERSION")).as_str());
            ui.label(None, "Copyright (C) 2026 Jacob Farnsworth");

            ui.label(None, "");
            
            ui.label(None, "Visual Physics is licensed under the");
            ui.label(None, "terms of the GNU GPLv2.");

            ui.label(None, "");

            ui.label(None, "https://github.com/JFarNTIG/simv");

            if ui.button(vec2(180.0, 370.0), "Done") {
                done = true;
            }
        });

        done
    }

    fn input_event(&mut self, _state: &Arc<Mutex<SimState>>, _event: WidgetInput) -> bool {
        false
    }

    fn get_type(&self) -> WidgetType {
        return WidgetType::AboutWindow;
    }
}