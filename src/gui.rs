#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
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
    ).unwrap();
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
    });
}
