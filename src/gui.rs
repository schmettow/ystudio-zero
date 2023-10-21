#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::helpers;
use crate::measurements;
use crate::monitor::MonitorApp;
use eframe::egui;
use std::collections::HashMap;
use std::fs;

extern crate csv;

/// Initializing the ui window
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub fn egui_init(app: MonitorApp) {
    let options = eframe::NativeOptions {
        transparent: true,
        initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        resizable: true,
        ..Default::default()
    };
    eframe::run_native(
        "Custom window frame", // unused title
        options,
        Box::new(|_cc| Box::new(app)),
    );
}

/// updates the plotter
/// 
pub fn update_central_panel(ctx: &egui::Context, app: &mut MonitorApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let mut plot = egui_plot::Plot::new("plotter");
        // reading the present y_include from the UI
        // NOTE: Can be removed
        let y_include = app.y_include.lock().unwrap();
        plot = plot
                .include_y(*y_include);

        let legend = egui_plot::Legend::default();
        plot = plot.legend(legend);

        plot.show(ui, |plot_ui| {
            for (_key, window) in &*app.measurements.lock().unwrap() {
                //println!("{}:{}", key);
                plot_ui.line(egui_plot::Line::new(window.plot_values()));
            }
        });
    });
}

pub fn update_right_panel(ctx: &egui::Context, app: &mut MonitorApp) {
    egui::SidePanel::right("left_right_panel").show(ctx, |ui| {
        ui.label("Serial Ports");
        let mut serial_port = app.port.lock().unwrap();
        // drop down
        egui::ComboBox::from_label("")
            .selected_text(format!("{}", serial_port.to_owned()))
            .show_ui(ui, |ui| {
                for i in app.available_ports.lock().unwrap().iter() {
                    ui.selectable_value(&mut *serial_port, i.to_string(), i.to_string());
                }
            });

        ui.label("Variables");
        ui.text_edit_multiline(&mut app.ui.vars);

        ui.label("Include Y-axis Range");
        ui.text_edit_singleline(&mut app.ui.y_include);
        // check if the y_include field has been changed
        if app.ui.y_include != app.ui.y_include_prev {
            app.ui.y_include_prev = app.ui.y_include.to_owned();
            // Extracting y-include as a number
            let a = helpers::parse_str_to_num(&app.ui.y_include);
            match a.parse::<f32>() {
                Ok(n) => {
                    // Locking the Mutex for use
                    let mut y_range = app.y_include.lock().unwrap();
                    // changing the dereferenced value
                    *y_range = n;
                }
                Err(_) => (),
            }
        }

        if ui.button("Export CSV").clicked() {
            /*
            let mut dict = HashMap::new();
            let meas = app.measurements.lock().unwrap();
            let m = meas.keys().to_owned();
            for i in m {
                let n = meas.get(i).to_owned();
                for j in n {
                    let o = j.values.to_owned();
                    for k in o {
                        let p = k.x;
                        let q = k.y;
                        dict.entry("Time").or_insert(Vec::new()).push(p);
                        dict.entry(i).or_insert(Vec::new()).push(q);
                    }
                }
            }
            let mut wtr = csv::Writer::from_path("test.csv").unwrap();
            for i in dict.keys() {
                // convert floats to strings because csv::Writer only writes u8's
                let str_dict = dict[i].clone().into_iter().map(|e| e.to_string());
                wtr.write_record(str_dict).unwrap();
            }
            wtr.flush().unwrap();*/
        }
    });
}

pub fn update_left_panel(ctx: &egui::Context, app: &mut MonitorApp) {
    egui::SidePanel::left("left_side_panel").show(ctx, |ui| {
        let disp = app.serial_data.lock().unwrap().to_owned();
        let disp = disp
            .into_iter()
            .rev()
            .take(50)
            .rev()
            .collect::<Vec<String>>();
        ui.label(disp.join("\n"));

        let mut user_command = app.serial_write.lock().unwrap();
        ui.text_edit_singleline(&mut *user_command);

        if ui.button("Send").clicked() {
            user_command.push_str("\n\r");
            let mut b = app.send_serial.lock().unwrap();
            *b = true;
        }

        ui.text_edit_singleline(&mut app.ui.log_name);

        if ui.button("Save logs").clicked() {
            match fs::File::create(&app.ui.log_name) {
                Ok(_) => println!("File created saved succesfully"),
                Err(e) => eprintln!("{:?}", e),
            }
            let log_data = app.serial_data.lock().unwrap().to_owned();
            match fs::write(&app.ui.log_name, log_data.join("\n")) {
                Ok(_) => println!("Logs saved succesfully"),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    });
}
