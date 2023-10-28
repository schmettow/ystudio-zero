#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use crate::{measurements, ylab};

use crate::ylab::{YLab, YLabState};
use crate::app::YUI;
use eframe::egui;
use std::collections::HashMap;
use std::fs;

extern crate csv;

/// Initializing the ui window
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]

pub fn egui_init(app: YUI) {
    let options = eframe::NativeOptions {
        transparent: true,
        initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        resizable: true,
        ..Default::default()
    };
    eframe::run_native(
        "Ystudio Zero", // unused title
        options,
        Box::new(|_cc| Box::new(app)),
    ).unwrap();
}

/// updates the plotter
/// 

pub fn update_central_panel(ctx: &egui::Context, app: &mut YUI) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let mut plot = egui_plot::Plot::new("plotter");
        // This zooms out, when larger values are encountered.
        // What it doesn't do, yet, is to zoom in on a new range.
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


// YLAB CONTROL
pub fn update_right_panel(ctx: &egui::Context, app: &mut YUI) {
    // Pulling in in the global states
    // all below need to be *dereferenced to be used
    // In the future, we'll try to only use YLabState
    //let mut this_ylab = app.ylab_version.lock().unwrap();
    let mut connected = app.connected.lock().unwrap();
    let mut ylab_state = app.ylab_state.lock().unwrap();
    let mut serial_port = app.port.lock().unwrap();

    // RIGHT PANEL
    egui::SidePanel::right("left_right_panel")
        .show(ctx,|ui| {
            /*match *connected {
                true => {
                    ui.label("Connected");
                    if ui.button("Disconnect").on_hover_text("Disconnect from YLab").clicked(){
                        *connected = false;
                        *ylab_state = YLabState::Disconnected;
                    };
                },
                false => {
                    ui.label("Disconnected");
                    if ui.button("Connect").on_hover_text("Connect to YLab").clicked() {
                        *ylab_state = YLabState::ReqConnect {
                            version: *this_ylab, 
                            port: *serial_port};
                    };
                }
            }*/

            // YLAB VERSION
            /* egui::ComboBox::from_label("YLab version")
                .selected_text(format!("{}", this_ylab))
                .show_ui(ui, |ui| {
                    ui.radio_value(&mut *this_ylab, YLab::Pro, "Pro");
                    ui.radio_value(&mut *this_ylab, YLab::Go, "Go");
                    ui.radio_value(&mut *this_ylab, YLab::Mini, "Mini");});
                    ui.label(this_ylab.baud().to_string());
            ui.label("Serial Ports");*/
            
            // SERIAL PORT
            /* egui::ComboBox::from_label("Serial Port")
                .selected_text(format!("{}", serial_port.to_owned()))
                .show_ui(ui, |ui| {
                    for i in app.available_ports.lock().unwrap().iter() {
                        ui.selectable_value(&mut *serial_port, 
                                            i.to_string(), 
                                            i.to_string());
                    }
                });*/



            // The new part, using YLabState matching
            let this_state = ylab_state.clone();
            match this_state {
                // Disconnected with possibly available ports
                YLabState::Disconnected {ports} => {
                    egui::ComboBox::from_label("Available Ports")
                        .show_ui(ui, |ui| {
                            if let Some(ports) = ports {
                                for i in ports.iter() {
                                    ui.selectable_value(&mut *serial_port, 
                                        i.to_string(), 
                                        i.to_string());
                            }}});},
                // These three states show the disconnect button
                YLabState::Connected {start_time:_, version, port} 
                |YLabState::Read { start_time:_, version, port } 
                |YLabState::Reading { start_time:_, version, port }=> {
                        ui.label("Connected");
                        ui.label(format!("{}:{}", version, port));
                        if ui.button("Disconnect").on_hover_text("Disconnect from YLab").clicked(){
                            *connected = false;
                            *ylab_state = YLabState::Disconnect{};
                        };
                },
                YLabState::Connect {version, port} => {
                    egui::ComboBox::from_label("Connecting")
                        .show_ui(ui, |ui| {
                            ui.label(format!("{}:{}", version, port));
                        });
                    //ui.label("Connecting");
                    //ui.label(format!("{}:{}", version, port));
                    ();
                },
                YLabState::Disconnect {  } => {
                    ui.label("Disconnecting");
                },
            }
        });
    }
            //let this_ylab = app.ylab_version.lock().unwrap();



pub fn update_left_panel(ctx: &egui::Context, app: &mut YUI) {
    egui::SidePanel::left("left_side_panel")
        .show(ctx, |ui| {
            let disp = app.serial_data.lock().unwrap().to_owned();
            let disp = disp
                .into_iter()
                .rev()
                .take(50)
                .rev()
                .collect::<Vec<String>>();
        ui.label(disp.join("\n"))});
    }
