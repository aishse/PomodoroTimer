use std::sync::{Mutex, Arc};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use serde::{Serialize, Deserialize};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[allow(dead_code)]

// initializes the timer duration, primarily used for frontend to set the time
#[tauri::command]
fn init_time(time: u32, control: State<'_, Arc<Mutex<PomodoroControl>>>) {
    let ctrl = control.inner().lock().unwrap();
    *ctrl.duration.lock().unwrap() = time;
}

// starts pomodoro timer in an async task. 
#[tauri::command]
async fn start_pomodoro(app: AppHandle, control: State<'_, Arc<Mutex<PomodoroControl>>>) -> Result<(), String> {
    let app = app.clone();
    let control = control.inner().clone(); 
   
    // timer spawn
    tauri::async_runtime::spawn(async move {
        // check if timer is already running, if so exits early
        {
            let ctrl = control.lock().unwrap();
            if ctrl.running {
                return;
            }
        }
      
        // Set running flag and clear reset flag
        {
            let mut ctrl = control.lock().unwrap();
            ctrl.running = true;
            ctrl.reset = false; 
        }

        // Track the start time and total paused time
        let start_time = std::time::Instant::now();
        let mut total_paused_duration = Duration::new(0, 0);
        let mut pause_start: Option<std::time::Instant> = None;

        loop { 
            let (is_paused, is_reset, current_duration, is_running) = {
                let ctrl = control.lock().unwrap();
                let x = (ctrl.paused, ctrl.reset, *ctrl.duration.lock().unwrap(), ctrl.running); x
            };

            // Check if we should stop the timer (reset OR not running anymore)
            if is_reset || !is_running {
                {
                    let mut ctrl = control.lock().unwrap();
                    ctrl.running = false;
                }
                break; 
            }

            if is_paused {
                // Mark pause start time if not already paused
                if pause_start.is_none() {
                    pause_start = Some(std::time::Instant::now());
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            } else {
                // If we were paused and now we're not, add the pause duration
                if let Some(pause_time) = pause_start {
                    total_paused_duration += pause_time.elapsed();
                    pause_start = None;
                }
            }

            // calculate remaining time based on paused time and total elapsed time of the timer
            let elapsed = start_time.elapsed() - total_paused_duration;
            let elapsed_secs = elapsed.as_secs() as u32;

            // calculate the number of seconds left, if negative then we're done
            let seconds_left = if current_duration > elapsed_secs {
                current_duration - elapsed_secs
            } else {
                0
            };

            // Emit the tick event with remaining time
            if let Err(e) = app.emit("tick", seconds_left) {
                eprintln!("Failed to emit tick event: {}", e);
                break;
            }

            // Check if timer completed
            if seconds_left == 0 {
                // delay a bit to ensure frontend can process last tick 
                tokio::time::sleep(Duration::from_secs(1)).await;

                // When timer completes naturally, auto-switch to next phase
                switch_phase_internal(app.clone(), control.clone());
                break; 
            }
            // sleep for a second before next tick 
            tokio::time::sleep(Duration::from_secs(1)).await;

        } // end loop 

        // Timer completion logic 
        // -----------------------------------------------------
     
        // Set running flag to false when timer completes
        {
            let mut ctrl = control.lock().unwrap();
            ctrl.running = false;
        }
        
        // Only emit done event if timer completed naturally (not reset)
        let should_emit_done = {
            let ctrl = control.lock().unwrap();
            !ctrl.reset
        };
        
        if should_emit_done {
            if let Err(e) = app.emit("done", ()) {
                eprintln!("Failed to emit done event: {}", e);
            }
        }
    });
    
    Ok(())
}

// pauses and resumes the timer 
#[tauri::command]
fn toggle_pomodoro(app: AppHandle, control: State<'_, Arc<Mutex<PomodoroControl>>>) {
    let mut ctrl = control.inner().lock().unwrap();
    ctrl.paused = !ctrl.paused;

    let _ = app.emit("toggle", ctrl.paused);
}
// resets the timer, stops any running timer and sets the reset flag
#[tauri::command]
fn reset_pomodoro(control: State<'_, Arc<Mutex<PomodoroControl>>>) {
    let mut ctrl = control.inner().lock().unwrap();
    ctrl.reset = true;
    ctrl.running = false; // Stop any running timer
}

// switches the phase of the pomodoro timer atomically, updates states and emits events to frontend
fn switch_phase_internal(app: AppHandle, control: Arc<Mutex<PomodoroControl>>) {
   // stop any running timer
    {
        let mut ctrl = control.lock().unwrap();
        ctrl.reset = true;
        ctrl.running = false;  // This will cause the timer loop to exit immediately
    }

    // delay to ensure async process exits

    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let mut ctrl = control.lock().unwrap();
    
    // Get current state and update it
    let new_state = {
        let current_state = ctrl.state.lock().unwrap();
        let mut sessions = ctrl.sessions.lock().unwrap();
        // determine the next state based on current state and session count. using asusmption that 
        // long breaks happen every 4 work sessions, may make this configurable later. 

        match *current_state {
            PomodoroState::Work => {
                *sessions += 1; 
                if *sessions % 4 == 0 {
                    PomodoroState::LongBreak
                } else {
                    PomodoroState::ShortBreak
                }
            },
            PomodoroState::ShortBreak | PomodoroState::LongBreak => {
                PomodoroState::Work
            }
        }
    };
    
    // Update the global state
    {
        let mut state = ctrl.state.lock().unwrap();
        *state = new_state.clone();
    }
    
    // Reset control flags
    ctrl.paused = false; // Don't set to paused, let the user control this
    ctrl.reset = false;  // Clear reset flag after stopping timer
    
    // notify frontend 
    let _ = app.emit("phase_changed", new_state);
    
    // delay to ensure frontend processes phase change
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Emit toggle event for frontend pause state  
    let _ = app.emit("toggle", ctrl.paused);
    
    // Emit event for timer stopped 
    let _ = app.emit("timer_stopped", ());
}

// wrapper to call switch phase from frontend
#[tauri::command]
fn switch_phase(app: AppHandle, control: State<'_, Arc<Mutex<PomodoroControl>>>) {
    switch_phase_internal(app, control.inner().clone());
}

// pomodoro states
#[derive(Default, Clone, Serialize, Deserialize)]
enum PomodoroState {
    #[default]
    Work, 
    ShortBreak, 
    LongBreak
}
#[allow(dead_code)]
struct Phase(Mutex<PomodoroState>);

// Control structure to manage the state of the pomodoro timer. Uses mutex for thread safety and atomicity.
#[derive(Default)]
struct PomodoroControl {
    paused: bool,
    reset: bool,
    running: bool, 
    state: Mutex<PomodoroState>,
    sessions: Mutex<u32>,
    duration: Mutex<u32>, 
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Phase(Mutex::new(PomodoroState::default())))
        .manage(Arc::new(Mutex::new(PomodoroControl { 
            paused: false, 
            reset: false, 
            running: false, 
            duration: Mutex::new(25 * 60), // Default to 25 minutes instead of 0
            state: Mutex::new(PomodoroState::default()), 
            sessions: Mutex::new(0) 
        })))
        .invoke_handler(tauri::generate_handler![start_pomodoro, toggle_pomodoro, reset_pomodoro, switch_phase, init_time])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
