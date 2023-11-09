use crate::ylab::*;
use crate::ylab::data::*;
use crate::ystudio::Ystudio;
use eframe::egui;
use egui_plot::PlotPoints;

/// Initializing the ui window
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]

//pub fn egui_init(ystud: Ystudio) {
pub fn egui_init(ystud: Ystudio) {
    let options = eframe::NativeOptions {
        transparent: true,
        initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        resizable: true,
        ..Default::default()
    };
    eframe::run_native(
        "Ystudio Zero", // unused title
        options,
        Box::new(|_cc| Box::new(ystud)),
    ).unwrap();
}


/// updates the plotting area
pub fn update_central_panel(ctx: &egui::Context, ystud: &mut Ystudio) 
{   egui::CentralPanel::default().show(ctx, |ui| {
        let mut plot = egui_plot::Plot::new("plotter");
        match ystud.ylab_state.lock().unwrap().clone() {
            YLabState::Reading { start_time:_, version:_, port_name:_ , recording:_} 
            => {// Split inconing history into points series
                let incoming: egui::util::History<data::Yld> = ystud.yld_wind.lock().unwrap().clone();
                let series = incoming.split();
                plot = plot
                        .auto_bounds_x()
                        .auto_bounds_y().
                        legend(egui_plot::Legend::default());
                let legend = egui_plot::Legend::default();
                plot = plot.legend(legend);
                // Plot lines
                plot.show(ui, |plot_ui| {
                for (probe, points) in series.iter().enumerate() {
                    if ystud.ui.selected_channels.lock().unwrap()[probe] {
                        let line = egui_plot::Line::new(PlotPoints::new(points.to_owned()));
                        plot_ui.line(line);
                    }
                }
        });
            },
            _ => {},
        }
        
    });
}


/// YLAB CONTROL in the right panel
/// + Connecting and Disconnecting
/// + Starting and stopping recodings
/// 
pub fn update_right_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    // Pulling in in the global states
    // all below need to be *dereferenced to be used
    // In the future, we'll try to only use YLabState
    //let mut this_ylab = ystud.ylab_version.lock().unwrap();
    
     // RIGHT PANEL
    egui::SidePanel::right("left_right_panel")
        .show(ctx,|ui| {
            let ylab_state = ystud.ylab_state.lock().unwrap();
            // setting defaults
            let selected_version 
                = match ystud.ui.selected_version.lock().unwrap().clone() {
                    Some(version) => version,
                    None => YLabVersion::Pro,};
            
            match ylab_state.clone() {
                YLabState::Connected { start_time:_, version, port_name }
                    => {ui.heading("Connected");
                        ui.label(format!("{}:{}", version, port_name));
                        if ui.button("Read").on_hover_text("Read from YLab").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Read{}).unwrap();}
                        if ui.button("Disconnect").on_hover_text("Disconnect YLab").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Disconnect{}).unwrap();}
                        },
                YLabState::Reading { start_time:_, version, port_name , recording:_}
                    => {let yld_wind = ystud.yld_wind.lock().unwrap();
                        ui.heading("Reading");
                        ui.label(format!("{}:{}", version, port_name));

                        let sample_rate: Option<f32> = yld_wind.mean_time_interval();
                        match sample_rate {
                            Some(sample_rate) => {ui.label(format!("{} Hz", (1.0/sample_rate) as usize));},
                            None => {ui.heading("Reading");},
                        }

                        ui.heading("Channels");
                        let mut selected_channels = ystud.ui.selected_channels.lock().unwrap();
                        for (chan, b) in  selected_channels.clone().iter().enumerate(){
                            let chan_selector = ui.checkbox(&mut b.clone(), chan.to_string());
                            if chan_selector.changed() {
                                selected_channels[chan] = !b;
                            }
                        };
                        if ui.button("Stop").on_hover_text("Stop reading").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Stop {}).unwrap(); 
                            println!("Cmd: Stop")};
                        },

                // Selecting port and YLab version
                // When both are selected, the connect button is shown
                YLabState::Disconnected {ports}
                    => {ui.heading("Disconnected");              
                        // When ports are available, show the options
                        match ports {
                            None => {ui.label("No ports available");
                                    eprintln!("No ports available");},
                            Some(ports) 
                                => {
                                // unpacking version and port
                                let selected_port: Option<String> 
                                    = match ystud.ui.selected_port.lock().unwrap().clone() {
                                        // in case there is a user-selected port, use it
                                        Some(port) => Some(port),
                                        // otherwise use the first available port
                                        None => if ports.len() > 0 {Some(ports[0].to_string())}
                                                else {None},
                                    };
                                // one selectable label for each port
                                ui.label("Available Ports");
                                for i in ports.iter() {
                                    // Create a selectable label for each port
                                    if ui.add(egui::SelectableLabel::new(selected_port == Some((*i).to_string()), i.to_string())).clicked() { 
                                        *ystud.ui.selected_port.lock().unwrap() = Some(i.clone());
                                    }
                                };
                                // one selectable per version
                                ui.label("Version");
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Pro, "Pro")).clicked() { 
                                    *ystud.ui.selected_version.lock().unwrap() = Some(YLabVersion::Pro);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Go, "Go")).clicked() { 
                                  *ystud.ui.selected_version.lock().unwrap() = Some(YLabVersion::Go);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Mini, "Mini")).clicked() { 
                                    *ystud.ui.selected_version.lock().unwrap() = Some(YLabVersion::Mini);
                                }
                                // The button is only shown when version and port are selected (which currently is by default).
                                // It commits the connection command to the YLab thread.

                                match ( ystud.ui.selected_version.lock().unwrap().clone(), ystud.ui.selected_port.lock().unwrap().clone())  {
                                    (Some(version), Some(port)) 
                                        =>  if ui.button("Connect")
                                                .on_hover_text("Connect to YLab")
                                                .clicked()  {ystud.ylab_cmd.send(  YLabCmd::Connect {version: version, port_name: port.to_string()}).unwrap();},
                                        _ => {ui.label("Select port and version");}
                                }
                                // The button is only shown when both version and port are selected
                                 
                            } 
                        }
                    },
                }
            });
        }
 
use egui_file::FileDialog;

pub fn update_left_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    egui::SidePanel::left("left_side_panel")
        .show(ctx, |ui| {
            ui.heading("Recording");
            let ylab_state = ystud.ylab_state.lock().unwrap().clone();
            match ylab_state {
                YLabState::Reading { start_time:_, version:_, port_name:_ , recording}
                => {match recording {
                    Some(Recording::Raw {start_time, file}) 
                    => {
                        ui.heading("Recording");
                        ui.label(format!("Raw: {}", file.display()));
                        ui.label(format!("Started: {}", start_time.elapsed().as_secs()));
                        if ui.button("Stop").on_hover_text("Stop recording").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Read{}).unwrap();}},
                    Some(Recording::Yld {start_time:_, file}) 
                    => {},
                    Some(Recording::Paused {start_time:_, file}) 
                    => {},
                    None 
                    => {
                        let start_rec = ui.button("New Recording")
                        .on_hover_text("Start a new recording");
                        if start_rec.clicked() {
                            let file = "test.csv";
                        ystud.ylab_cmd.send(YLabCmd::Record{file: file.into()}).unwrap();}},
                    }
                },
              _ => {},
            }
        });
    }

