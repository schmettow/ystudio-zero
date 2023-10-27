mod gui;
mod measurements;
mod app;
mod threads;
mod ylab;

use egui::util::History;
use ::mpsc::Sender;
use std::{thread, sync::*};
const N_HISTORY: usize = 1000;

fn main() {
    // initialize the app
    let ylab = ylab::YLabState::Disconnected {ports: None};
    let ylab_hist = Arc::new(Mutex::new(History::new(0..200,100.0)));
    let (ylab_cmd, ylab_listen) = mpsc::channel();

    // Creating sliding windows for 8 channels
    for chan_id in ylab::yld::CHAN_IDS {
        // open the thread-safe hashmap and add a channel
        measurements
        .lock()
        .unwrap()
        .insert(chan_id.into(),
            measurements::MeasurementWindow::new_with_look_behind(N_HISTORY));
        println!("Creating window for {}", chan_id);
    }

    // Alternative implementation for sliding windows using egui History
    
    // this is all needed by the serial thread
    let ylab_state = app.ylab_state.clone();
    let ylab_data = app.history.clone();

    // starting the serial listener thread, 
    // consuming all mutexes
    thread::spawn(move || {
        threads::serial_thread(
            ylab_state,
            ylab_data,
            ylab_listen
        );
    });

    // starting the egui
    gui::egui_init(app);
}
