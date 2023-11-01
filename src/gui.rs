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

pub fn update_left_panel(ctx: &egui::Context, app: &mut Ystudio) {
    egui::SidePanel::left("left_side_panel")
        .show(ctx, |ui| {
            ui.heading("Ystudio Zero");
            ui.label("Make recording")
            /* let disp = app.serial_data.lock().unwrap().to_owned();
            let disp = disp
                .into_iter()
                .rev()
                .take(50)
                .rev()
                .collect::<Vec<String>>();
        ui.label(disp.join("\n"))*/});
    }



/// updates the plotting area
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


/// YLAB CONTROL in the right panel
/// + Connecting and Disconnecting
/// + Starting and stopping recodings
pub fn update_right_panel(ctx: &egui::Context, app: &mut Ystudio) {
    // Pulling in in the global states
    // all below need to be *dereferenced to be used
    // In the future, we'll try to only use YLabState
    //let mut this_ylab = app.ylab_version.lock().unwrap();
    
     // RIGHT PANEL
    egui::SidePanel::right("left_right_panel")
        .show(ctx,|ui| {
            let ylab_state = app.ylab_state.lock().unwrap();
            // setting defaults
            let selected_version 
                = match app.ui.selected_version.lock().unwrap().clone() {
                    Some(version) => version,
                    None => YLabVersion::Pro,};
            
            match ylab_state.clone() {
                YLabState::Connected { start_time:_, version, port }
                    => {ui.heading("Connected");
                        ui.label(format!("{}:{}", version, port));
                        if ui.button("Read").on_hover_text("Read from YLab").clicked(){
                            app.ylab_cmd.send(YLabCmd::Read{}).unwrap();}
                        if ui.button("Read").on_hover_text("Read from YLab").clicked(){
                            app.ylab_cmd.send(YLabCmd::Read{}).unwrap();}
                        },
                YLabState::Reading { start_time:_, version, port }
                    => {ui.heading("Reading");
                        ui.label(format!("{}:{}", version, port));},

                YLabState::Recording { path }
                    => {ui.heading("Recording");
                        ui.label(format!("{}", path.display()));},

                // Selecting port and YLab version
                // When both are selected, the connect button is shown
                YLabState::Disconnected {ports}
                    => {ui.heading("Disconnected");
                        // unpacking version and port
                        /* let selected_version = 
                            match app.ui.selected_version.lock().unwrap().clone() {
                                Some(version) => version,
                                None => YLabVersion::Pro,
                            };*/
                        let selected_port 
                            = match app.ui.selected_port.lock().unwrap().clone() {
                                    Some(port) => port,
                                    None => ports.as_ref().unwrap()[0].to_string(),};
               
                        // When ports are available, show the options
                        match ports {
                            None => {ui.label("No ports available");},
                            Some(ports) => {
                                // one selectable label for each port
                                ui.label("Available Ports");
                                for i in ports.iter() {
                                    // Create a selectable label for each port
                                    if ui.add(egui::SelectableLabel::new(selected_port == *i, i.to_string())).clicked() { 
                                        *app.ui.selected_port.lock().unwrap() = Some(i.clone());
                                    }
                                };
                                // one selectable per version
                                ui.label("Version");
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Pro, "Pro")).clicked() { 
                                    *app.ui.selected_version.lock().unwrap() = Some(YLabVersion::Pro);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Go, "Go")).clicked() { 
                                  *app.ui.selected_version.lock().unwrap() = Some(YLabVersion::Go);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Mini, "Mini")).clicked() { 
                                    *app.ui.selected_version.lock().unwrap() = Some(YLabVersion::Mini);
                                }
                                // The button is only shown when version and port are selected (which currently is by default).
                                // It commits the connection command to the YLab thread.

                                match ( app.ui.selected_version.lock().unwrap().clone(), app.ui.selected_port.lock().unwrap().clone())  {
                                    (Some(version), Some(port)) 
                                        =>  if ui.button("Connect")
                                                .on_hover_text("Connect to YLab")
                                                .clicked()  {app.ylab_cmd.send(  YLabCmd::Connect {version: version, port: port.to_string()}).unwrap();},
                                        _ => {ui.label("Select port and version");}
                                }
                                // The button is only shown when both version and port are selected
                                 // Connect button
                                } // available ports
                        } // match ports
                    }, // arm
                } // match
            }); // sidepanel
        } // fn
 