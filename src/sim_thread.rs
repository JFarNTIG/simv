use std::{sync::{Arc, Mutex}, time, thread};
use crate::{universe::Universe, sim_state::SimState};

pub fn enter_thread_loop(state: &Arc<Mutex<SimState>>, body: fn(&mut Universe, f64) -> bool, end: fn()) {
    let mut now = time::Instant::now();
    let mut last_time = time::Instant::now();

    loop {
        let dt = (now - last_time).as_secs_f64();
        last_time = now;

        {
            if body(&mut state.lock().unwrap().uni, dt) {
                break;
            }
        }

        end();

        now = time::Instant::now();
    }
}

pub fn spawn_sim_thread(state: &Arc<Mutex<SimState>>, body: fn(&mut Universe, f64) -> bool, end: fn()) {
    let state = Arc::clone(state);

    thread::spawn(move || {
        enter_thread_loop(&state, body, end);
    });
}