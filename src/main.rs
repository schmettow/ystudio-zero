mod gui;
mod helpers;
mod measurements;
mod monitor;
mod threads;
mod ylab;

use std::thread;

fn main() {
    helpers::setup_subscriber();
    let app = monitor::MonitorApp::new();
    let measurements = app.measurements.clone();
    for key in ylab::yld::CHAN_IDS {
        //let key: String = ["y",&i.to_string()].join("");
        measurements
        .lock()
        .unwrap()
        .insert(key.into(),
            measurements::MeasurementWindow::new_with_look_behind(1000));
        println!("Creating window for {}", key);
    }
    
    let port = app.port.clone();
    let available_ports = app.available_ports.clone();
    let variables = app.variables.clone();
    let serial_data = app.serial_data.clone();
    let send_serial = app.send_serial.clone();
    let serial_write = app.serial_write.clone();

    thread::spawn(move || {
        threads::serial_thread(
            measurements,
            port,
            available_ports,
            variables,
            serial_data,
            send_serial,
            serial_write,
        );
    });

    gui::egui_init(app);
}
