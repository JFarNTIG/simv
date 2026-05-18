#![windows_subsystem = "windows"]

mod drawing;
mod body;
mod universe;
mod ringarray;
mod sim_thread;
mod sim_state;
mod widget;
mod actions;

use std::{thread, sync::{Mutex, Arc}, time, collections::VecDeque};
use macroquad::{prelude::*, ui::root_ui};

use sim_thread::spawn_sim_thread;
use universe::Universe;
use widget::{Task, Widget, StatusOverlay, MenuBar, PhysicsOverlays, SettingsWindow, AboutWindow};
use sim_state::{SimConfig, SimState};
use actions::AppAction;

/// Time in seconds between points in a trail.
const TRAIL_UPDATE_RATE: f64 = 0.1;
const ZOOM_FACTOR: f32 = 1.5;

/// Default zoom level applied to the camera.
const DEFAULT_ZOOM: f32 = 1.0 / 600000.0;

pub struct AppContext {
    state: Arc<Mutex<SimState>>,
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    widgets: Vec<Box<dyn Widget>>,
    trail_timer: f64,
    last_time: f64,
}

#[macroquad::main("Visual Physics")]
async fn main() {
    let mut app = AppContext::new();

    // Initial setup
    app.init_physics_thread();

    loop {
        let dt = app.update_timer();
        let fps = (1.0 / dt).round() as i32;
        
        // Process input
        app.handle_input();
        app.dispatch_pending_actions();
        app.update_trails(dt);
        app.process_tasks();

        // Prepare to render the frame
        clear_background(BLACK);
        let camera = app.state.lock().unwrap().get_camera();
        
        // World space
        set_camera(&camera);
        app.draw_world();

        // Screen space
        set_default_camera();
        app.draw_ui(&camera);

        // Render FPS text in bottom right
        let fps_text = format!("FPS: {}", fps);
        let fps_text_dims = measure_text(&fps_text, None, 24, 1.0);

        draw_text(&fps_text, screen_width() - fps_text_dims.width - 4.0, screen_height() - fps_text_dims.height - 4.0, 24.0, WHITE);

        next_frame().await
    }
}

impl AppContext {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SimState {
                uni: Universe::new(),
                trails: Vec::new(),
                config: SimConfig::new(),
                action_queue: VecDeque::new(),
                zoom: DEFAULT_ZOOM,
                view_center: vec2(0.0, 0.0),
                focusing: false,
            })),
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            widgets: vec![
                Box::new(PhysicsOverlays {selected_body: None}),
                Box::new(MenuBar {}),
                Box::new(StatusOverlay {}),
            ],
            trail_timer: TRAIL_UPDATE_RATE,
            last_time: get_time(),
        }
    }

    fn init_physics_thread(&self) {
        // Verlet requires one initial step to populate previous positions
        self.state.lock().unwrap().uni.physics_tick(0.0);

        spawn_sim_thread(&self.state, physics_thread_body, || {
            thread::sleep(time::Duration::from_millis(1));
        });
    }

    fn update_timer(&mut self) -> f64 {
        let current_time = get_time();
        let dt = current_time - self.last_time;
        self.last_time = current_time;
        dt
    }

    fn handle_input(&mut self) {
        {
            let mut state = self.state.lock().unwrap();

            // Mouse wheel changes zoom
            let (_wheel_x, wheel_y) = mouse_wheel();
            if wheel_y > 0.0 { state.zoom *= ZOOM_FACTOR; }
            if wheel_y < 0.0 { state.zoom /= ZOOM_FACTOR; }

            // Spacebar pauses / unpauses
            if is_key_pressed(KeyCode::Space) {
                state.uni.paused = !state.uni.paused;
            }

            // F toggles focus
            if is_key_pressed(KeyCode::F) {
                state.focusing = !state.focusing;
            }

            // Dragging while holding RMB pans, but only
            // if we are not focusing on an object.
            if !state.focusing {
                if is_mouse_button_down(MouseButton::Right) {
                    let mouse_delta = mouse_delta_position() / state.zoom;
                    state.view_center.x += mouse_delta.x;
                    state.view_center.y -= mouse_delta.y;
                }
            }

            // O opens settings window
            if is_key_pressed(KeyCode::O) {
                state.action_queue.push_front(AppAction::OpenSettingsWindow);
            }
        }

        // LMB pressed; forward to widgets
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_pos = vec2(mouse_x, mouse_y);

            // Only forward to widgets if the mouse is not over a UI element
            if !root_ui().is_mouse_over(mouse_pos) {
                self.widgets.iter_mut().for_each(|w| {
                    w.input_event(&self.state, widget::WidgetInput::LeftClick(mouse_pos));
                });
            }
        }
    }

    fn dispatch_pending_actions(&mut self) {
        let mut actions: Vec<AppAction> = Vec::new();

        // collect the current actions in a temporary vector,
        // and clear the action queue before processing actions
        {
            let mut state = self.state.lock().unwrap();
            actions.append(&mut state.action_queue.iter().map(|a| *a).collect());
            state.action_queue.clear();
        }
        
        // Now we process the pending actions
        for pending_action in &actions {
            match pending_action {
                AppAction::OpenSettingsWindow => {
                    self.add_widget_if_not_exists(Box::new(SettingsWindow {}));
                },
                AppAction::OpenAboutWindow => {
                    self.add_widget_if_not_exists(Box::new(AboutWindow {}));
                },
                AppAction::Quit => {
                    std::process::exit(0);
                },
            }
        }
    }

    fn add_widget_if_not_exists(&mut self, new_widget: Box<dyn Widget>) {
        if let None = self.widgets.iter().find(|w| { w.get_type() == new_widget.get_type() }) {
            self.widgets.push(new_widget);
        }
    }

    fn update_trails(&mut self, dt: f64) {
        self.trail_timer -= dt;
        if self.trail_timer <= 0.0 {
            self.trail_timer = TRAIL_UPDATE_RATE;
            let state = &mut *self.state.lock().unwrap();

            for (i, body) in state.uni.bodies.iter().enumerate() {
                if i < state.trails.len() {
                    state.trails[i].push(body.pos);
                }
            }
        }
    }

    fn draw_world(&self) {
        let state = self.state.lock().unwrap();

        for body in &state.uni.bodies {
            draw_circle(
                body.pos.x as f32, 
                body.pos.y as f32, 
                body.radius as f32, 
                Color::from_vec(body.color)
            );
        }
    }

    fn draw_ui(&mut self, camera: &Camera2D) {
        // While drawing widgets, a widget may return `true` from its render function,
        // signaling that the widget has closed.
        // To remove widgets that have closed from the widget vec, we use `.retain()` and
        // call `render()` in the provided closure.
        self.widgets.retain(|w| !w.render(camera, &self.state, &self.task_queue));
    }

    fn process_tasks(&self) {
        let mut queue = self.task_queue.lock().unwrap();

        while let Some(task) = queue.pop_front() {
            task(&self.state);
        }
    }
}

pub fn physics_thread_body(uni: &mut Universe, dt: f64) -> bool {
    if !uni.paused {
        uni.physics_tick(dt * uni.time_scale);
    }
    false
}