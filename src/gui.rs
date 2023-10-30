use crate::{ylab::*, ystudio::MultiLine};
use crate::ystudio::Ystudio;
use eframe::egui;
//use egui_plot::{PlotPoint, PlotPoints};

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
        let incoming = app.ylab_data.lock().unwrap().clone();
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
    let ylab_state = app.ylab_state.lock().unwrap();
    // RIGHT PANEL
    egui::SidePanel::right("left_right_panel")
        .show(ctx,|ui| {
            match ylab_state.clone() {
                YLabState::Connected { start_time:_, version, port }
                => {ui.label("Connected");
                    ui.label(format!("{}:{}", version, port));},
                YLabState::Reading { start_time:_, version, port }
                => {ui.label("Reading");
                    ui.label(format!("{}:{}", version, port));},
                YLabState::Disconnected {ports}
                    => {ui.label("Disconnected");
                        match ports {
                            None => {ui.label("No ports available");},
                            Some(ports) => {
                                egui::ComboBox::from_label("Available Ports")
                                    .show_ui(ui, |ui| {
                                        for i in ports.iter() {
                                            ui.selectable_value(&mut *app.ui.selected_port.lock().unwrap(), 
                                                Some(i.to_string()), 
                                                i);}}
                                            );
                                egui::ComboBox::from_label("YLab version")
                                    .show_ui(ui, |ui| {
                                        ui.radio_value(&mut *app.ui.selected_version.lock().unwrap(), Some(YLabVersion::Pro), "Pro");
                                        ui.radio_value(&mut *app.ui.selected_version.lock().unwrap(), Some(YLabVersion::Go), "Go");
                                        ui.radio_value(&mut *app.ui.selected_version.lock().unwrap(), Some(YLabVersion::Mini), "Mini");});
                                let selected_version = app.ui.selected_version.lock().unwrap().clone();
                                let selected_port = app.ui.selected_port.lock().unwrap().clone();
                                // The button is only shown when both version and port are selected
                                match (selected_version, selected_port)  {
                                    (Some(version), Some(port)) => {
                                        if ui.button("Connect").on_hover_text("Connect to YLab").clicked(){
                                        app.ylab_cmd.send(YLabCmd::Connect {
                                            version: version, 
                                            port: port}).unwrap();
                                        }
                                    },
                                    _ => {ui.label("Select port and version");}, 
                                } // Connect button
                            } // available ports
                        } // match ports
                    }, // arm
                } // match
            }); // sidepanel
        } // fn
 
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
