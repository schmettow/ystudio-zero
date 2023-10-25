mod gui;
mod measurements;
mod monitor;
mod threads;
mod ylab;

use std::thread;

fn main() {
    // initialize the app
    let app = monitor::MonitorApp::new();
    // make a copy of the measurements hash map
    let measurements = app.measurements.clone();

    // Creating sliding windows for 8 channels
    for chan_id in ylab::yld::CHAN_IDS {
        // open the thread-safe hashmap and add a channel
        measurements
        .lock()
        .unwrap()
        .insert(chan_id.into(),
            measurements::MeasurementWindow::new_with_look_behind(1000));
        println!("Creating window for {}", chan_id);
    }

    // Alternative implementation for sliding windows using egui History
    let history = app.history.clone();

    // this is all needed by the serial thread
    let port = app.port.clone();
    let available_ports = app.available_ports.clone();
    let serial_data = app.serial_data.clone();
    let this_ylab = app.ylab_version.clone();
    let connected = app.connected.clone();

    // starting the serial listener thread, 
    // consuming all mutexes
    thread::spawn(move || {
        threads::serial_thread(
            measurements,
            this_ylab,
            connected,  
            history,
            port,
            available_ports,
            serial_data,
        );
    });

    // starting the egui
    gui::egui_init(app);
}
