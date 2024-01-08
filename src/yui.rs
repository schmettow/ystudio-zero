use crate::{ylab::*, yldest::{*, self}};
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

/// updates the bottom window


/// updates the plotting area
pub fn update_central_panel(ctx: &egui::Context, ystud: &mut Ystudio) 
{   egui::CentralPanel::default().show(ctx, |ui| {
        let mut plot = egui_plot::Plot::new("plotter");
        match ystud.ylab_state.lock().unwrap().clone() {
            YLabState::Reading {version: _, port_name: _} 
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
/// + Connecting; reading and disconnecting
///
pub fn update_right_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    egui::SidePanel::right("left_right_panel")
        .show(ctx,|ui| {
            let ylab_state = ystud.ylab_state.lock().unwrap();
            // setting defaults
            let selected_version 
                = match ystud.ui.selected_version.lock().unwrap().clone() {
                    Some(version) => version,
                    None => YLabVersion::Pro,};
            
            match ylab_state.clone() {
                // Connected to YLab by selecting port and version
                YLabState::Connected {version, port_name}
                    => {ui.heading("Connected");
                        ui.label(format!("{}:{}", version, port_name));
                        if ui.button("Disconnect").on_hover_text("Disconnect YLab").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Disconnect{}).unwrap();}
                        if ui.button("Read").on_hover_text("Read from YLab").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Read{}).unwrap();}
                        },
                
                // Reading from YLab, showing the port, version and sample rate
                YLabState::Reading {version, port_name}
                    => {let yld_wind = ystud.yld_wind.lock().unwrap();
                        ui.heading("Reading");
                        ui.label(format!("{}:{}", version, port_name));

                        let sample_rate: Option<f32> = yld_wind.mean_time_interval();
                        match sample_rate {
                            Some(sample_rate) => {ui.label(format!("{} Hz", (1.0/sample_rate) as usize));},
                            None => {ui.label("still buffering");},
                        }
                        // Selecting channels to plot
                        ui.heading("Channels");
                        let mut selected_channels = ystud.ui.selected_channels.lock().unwrap();
                        for (chan, b) in  selected_channels.clone().iter().enumerate(){
                            let chan_selector = ui.checkbox(&mut b.clone(), chan.to_string());
                            if chan_selector.changed() {
                                selected_channels[chan] = !b;
                            }
                        };
                        // Stop reading
                        if ui.button("Stop Read").on_hover_text("Stop reading").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Stop {}).unwrap(); 
                            println!("Cmd: Stop")};
                        
                        let yldest_state = ystud.yldest_state.lock().unwrap().clone();
                        match yldest_state {
                            YldestState::Idle{ dir: Some(dir) }
                            => {
                                ui.label("Idle");
                                if ui.button("New Rec").on_hover_text("Start a new recording").clicked() {
                                    let dir = std::env::current_dir().unwrap();
                                    ystud.yldest_cmd.send(YldestCmd::New {change_dir: Some(dir), file_name: None}).unwrap()
                                }
                            },
                            YldestState::Recording { path }
                            => {
                                ui.label(format!("Recording to {}", path.to_str().unwrap()));
                                if ui.button("Stop Rec").on_hover_text("Stop recording").clicked() {
                                        ystud.yldest_cmd.send(YldestCmd::Stop).unwrap();}
                            },
                            _ => {}
                        }
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
                                 
                            } 
                        }
                    },
                }
            });
        }

#[allow(unused_imports)]
use egui_file::FileDialog;

pub fn update_left_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    egui::SidePanel::left("left_side_panel")
        .show(ctx, |ui| {
            ui.heading("Recording");
            let ylab_state = ystud.ylab_state.lock().unwrap().clone();
            let yldest_state = ystud.yldest_state.lock().unwrap().clone();
            
            match (ylab_state, yldest_state) {
                // show New button when Reading and Idle
                (YLabState::Reading {version:_, port_name:_},
                 YldestState::Idle {dir: Some(_)})
                    => {
                        ui.label("Idle");
                        if ui.button("New Recording").on_hover_text("Start a new recording").clicked() {
                            let dir = std::env::current_dir().unwrap();
                            ystud.yldest_cmd.send(YldestCmd::New {change_dir: Some(dir), file_name: None}).unwrap()
                        }},
                // show path and stop button when recording
                (YLabState::Reading {version:_, port_name:_},
                 YldestState::Recording {path}) 
                => {
                   ui.label(format!("Recording to {}", path.to_str().unwrap()));
                   if ui.button("Stop").on_hover_text("Stop recording").clicked() {
                        ystud.yldest_cmd.send(YldestCmd::Stop).unwrap();}},
                (_,_) => {},
            }
        }
    );
}


pub fn update_bottom_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    egui::TopBottomPanel::bottom("bottom_panel")
        .show(ctx, |ui| {
            ui.heading("Analysis");
            let ylab_state = ystud.ylab_state.lock().unwrap().clone();
            //let yldest_state = ystud.yldest_state.lock().unwrap().clone();
            
            match ylab_state {
                // show New button when Reading and Idle
                YLabState::Reading {version:_, port_name:_}
                => {
                    let incoming: egui::util::History<data::Yld> = ystud.yld_wind.lock().unwrap().clone();
                    let series = incoming.split();
                    ui.heading("Spectral power analysis");
                    let mut plot = egui_plot::Plot::new("plotter");
                    let sample_rate: Option<f32> = incoming.mean_time_interval();
                    match (ystud.ylab_state.lock().unwrap().clone(), sample_rate) {
                        (YLabState::Reading {version: _, port_name: _}, Some(sample_rate)) 
                        => {// Split inconing history into sample vector
                            ui.label(format!("{} SPS", (sample_rate) as usize));
                            let incoming: egui::util::History<data::Yld> = ystud.yld_wind.lock().unwrap().clone();
                            let series = &incoming.split()[0];
                            let mut samples: [f32; 4096] = [0.0; 4096];
                            for (i,s) in series.iter().enumerate() {
                                samples[i] = s[1] as f32;
                            }
                            use spectrum_analyzer::scaling::divide_by_N;
                            use spectrum_analyzer::windows::hann_window;
                            use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
                            let hann_window = hann_window(&samples);
                            // calc spectrum
                            let spectrum_hann_window = samples_fft_to_spectrum(
                                // (windowed) samples
                                &hann_window,
                                // sampling rate
                                sample_rate as u32,
                                // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
                                FrequencyLimit::All,
                                // optional scale
                                Some(&divide_by_N),
                            )
                            .unwrap();        
                            plot = plot
                                    .auto_bounds_x()
                                    .auto_bounds_y().
                                    legend(egui_plot::Legend::default());
                            let legend = egui_plot::Legend::default();
                            plot = plot.legend(legend);
                            // Plot lines
                            plot.show(ui, |plot_ui| {
                            /*for (probe, points) in series.iter().enumerate() {
                                if ystud.ui.selected_channels.lock().unwrap()[probe] {
                                    let line = egui_plot::Line::new(PlotPoints::new(points.to_owned()));
                                    plot_ui.line(line);
                                }
                            }*/
                });
                    },
                    _ => {},
                }
        
                    
                    },
                _   => {ui.label("Idle");},
            }
        }
    );
}

