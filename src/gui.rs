use std::net::Incoming;

use crate::{ylab::*, ystudio::MultiLine};
use crate::ystudio::Ystudio;
use eframe::egui;
use egui_plot::{PlotPoint, PlotPoints};
//use std::fs;

extern crate csv;

/// Initializing the ui window
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]

pub fn egui_init(app: Ystudio) {
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

pub fn update_central_panel(ctx: &egui::Context, app: &mut Ystudio) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let mut plot = egui_plot::Plot::new("plotter");
        // This zooms out, when larger values are encountered.
        // What it doesn't do, yet, is to zoom in on a new range.
        // let y_include = app.y_include.lock().unwrap();
        // This happens instantly

        // Grab the inconing history
        let incoming = app.ylab_data.lock().unwrap().to_owned();
        // Adjust the upper y limit (just to showcase)
        plot = plot
                .include_y(*app.ui.y_include.lock().unwrap());
        // Add a legend
        let legend = egui_plot::Legend::default();
            plot = plot.legend(legend);
        // Create 8 lines
        let plot_lines = incoming.multi_lines();
        plot.show(ui, |plot_ui| {
            for series in plot_lines.iter() {
                plot_ui.line(egui_plot::Line::new(*series));
            }
        });
    });
}


// YLAB CONTROL
pub fn update_right_panel(ctx: &egui::Context, app: &mut Ystudio) {
    // Pulling in in the global states
    // all below need to be *dereferenced to be used
    // In the future, we'll try to only use YLabState
    //let mut this_ylab = app.ylab_version.lock().unwrap();
    let mut ylab_state = app.ylab_state.lock().unwrap();
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
                |YLabState::Reading { start_time:_, version, port }=> {
                        ui.label("Connected");
                        ui.label(format!("{}:{}", version, port));
                        if ui.button("Disconnect").on_hover_text("Disconnect from YLab").clicked(){
                            *ylab_state = YLabState::Disconnected{ports: None};
                        };
                },
            }
        });
    }
            //let this_ylab = app.ylab_version.lock().unwrap();



pub fn update_left_panel(ctx: &egui::Context, app: &mut Ystudio) {
    egui::SidePanel::left("left_side_panel")
        .show(ctx, |ui| {
            /* let disp = app.serial_data.lock().unwrap().to_owned();
            let disp = disp
                .into_iter()
                .rev()
                .take(50)
                .rev()
                .collect::<Vec<String>>();
        ui.label(disp.join("\n"))*/});
    }
