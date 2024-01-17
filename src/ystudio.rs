
pub use eframe::egui;
pub use egui::util::History;
pub use egui_plot::PlotPoints;
use serialport::new;
pub use std::sync::mpsc::Sender;
use std::vec;
pub use std::{thread, sync::*};
pub use crate::ylab::{YLabState, YLabCmd, YLabVersion, data::*};
pub use crate::ylab::*;
pub use crate::yldest::*;


impl eframe::App for Ystudio {
    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // update_left_panel(ctx, self);
        update_right_panel(ctx, self);
        update_central_panel(ctx, self);
        update_bottom_panel(ctx, self);
        ctx.request_repaint();
    }
}


/// Data for the UI
/// 
/// This is sometimes necessary to hold several values in the UI 
/// (port, Version) before submitting the command to YLab
/// 
#[derive(Debug, Clone)]
pub struct Yui {
    pub selected_port: Arc<Mutex<Option<String>>>,
    pub selected_version: Arc<Mutex<Option<YLabVersion>>>,
    pub selected_channels: Arc<Mutex<[bool; 8]>>,
    pub lowpass_threshold: Arc<Mutex<f64>>,
    pub lowpass_burnin: Arc<Mutex<f64>>,
    pub frequency_range: Arc<Mutex<spectrum_analyzer::FrequencyLimit>>,
    //opened_file: Option<PathBuf>,
    //open_file_dialog: Option<FileDialog>,
}

#[derive(Debug, Clone)]
pub struct Yui2 {
    pub selected_port: Option<String>,
    pub selected_version: Option<YLabVersion>,
    pub selected_channels: [bool; 8],
    pub lowpass_threshold: f64,
    pub lowpass_burnin: f64,
    pub fft_min: f64,
    pub fft_max: f64
    //opened_file: Option<PathBuf>,
    //open_file_dialog: Option<FileDialog>,
}

/// The ystudio object contains thread-safe channels
/// for communication between components, as well as 
/// ui properties
/// 
/// + ylab_state, which is a mutexed YLabState
/// + ylab_cmd for sending commands to YLab, esp. changing states
/// + yldest_state carries the state of the storage component
/// + yldest_cmd for controlling the storage component
/// + yld_wind, which is a egui History of YLab Samples in Yld format
/// + ytf_wind, which is a egui History of samples in Ytf8 format
/// + ui, which captures UI related variables with individual locks
/// + ui2 is experimental and uses only a global lock

    
#[derive(Clone)]
pub struct Ystudio {
    pub ylab_state: Arc<Mutex<YLabState>>, // shared state 
    pub ylab_cmd: Sender<YLabCmd>, // sending commands to ylab
    pub yldest_state: Arc<Mutex<YldestState>>, // shared state
    pub yldest_cmd: mpsc::Sender<YldestCmd>, // sending commands to control storage
    pub yld_wind: Arc<Mutex<History<Yld>>>, // data stream, sort of temporal vecdeque
    pub ytf_wind: Arc<Mutex<History<Ytf8>>>, // data stream, sort of temporal vecdeque
    pub ui: Yui, // user interface value buffer
    pub ui2: Arc<Mutex<Yui2>>, // user interface value buffer as outer ARC
}

use std::collections::VecDeque;

/// Initializing the egui window

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
/// 
pub fn update_central_panel(ctx: &egui::Context, ystud: &mut Ystudio) 
{   egui::CentralPanel::default().show(ctx, |ui| {
        let mut plot = egui_plot::Plot::new("plotter");
        match ystud.ylab_state.lock().unwrap().clone() {
            YLabState::Reading {version: _, port_name: _} 
            => {// Split inconing history into points series
                let incoming: egui::util::History<data::Yld> = ystud.yld_wind.lock().unwrap().clone();
                let series 
                    = incoming.split();
                plot = plot
                        .auto_bounds_x()
                        .include_y(0.0) // <----- still hard coded 
                        //.include_y(0.06) // <----- still hard coded
                        .auto_bounds_y()
                        .legend(egui_plot::Legend::default());
                let legend = egui_plot::Legend::default();
                plot = plot.legend(legend);
                // Plot lines
                plot.show(ui, |plot_ui| {

                
                for (probe, points) in &mut series.clone().iter().enumerate() { 
                    if ystud.ui.selected_channels.lock().unwrap()[probe] {
                        use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};
                        //let cutoff = 100.hz();
                        let cutoff = *ystud.ui.lowpass_threshold.lock().unwrap() as f32;
                        let sampling_rate = incoming.rate();
                        match sampling_rate {
                            None => {},
                            Some(rate) => {
                                // Create coefficients for the biquads
                                let coeffs 
                                    = Coefficients::<f32>::from_params(Type::LowPass, rate.hz(), cutoff.hz(), Q_BUTTERWORTH_F32);
                                match coeffs {
                                    Err(e) => println!("{:?}", e),
                                    Ok(coeffs) => {
                                        let mut filtered_points: VecDeque<[f64; 2]> = VecDeque::new();
                                        let mut biquad_lpf = DirectForm1::<f32>::new(coeffs);
                                        points.iter()
                                        .for_each(|point| filtered_points.push_front([point[0], biquad_lpf.run(point[1] as f32) as f64]));
                                        //let burnin = *ystud.ui.lowpass_burnin.lock().unwrap() as usize;
                                        let burnin: usize = ((rate * 2.0)/cutoff) as usize;
                                        
                                        for _ in 1..burnin {
                                            filtered_points.pop_back();
                                        }
                                        let filtered_line = egui_plot::Line::new(PlotPoints::new(filtered_points.to_owned().into()));
                                        plot_ui.line(filtered_line);

                                    }
                                }
                                
                            },
                        }
                        
                        
                        
                    }
                }
        });
            },
            _ => {},
        }
        
    });
}


/// updates bottom panel with FFT
pub fn update_bottom_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    egui::TopBottomPanel::bottom("bottom_panel")
        .show(ctx, |ui| {
            ui.heading("Distribution of Frequencies");
            let ylab_state = ystud.ylab_state.lock().unwrap().clone();
            // using a global lock
            let ui_state = ystud.ui2.lock().unwrap();
            
            match ylab_state {
                // show New button when Reading and Idle
                YLabState::Reading {version, port_name:_}
                => {
                    #[warn(non_upper_case_globals)]
                    let fft_size = version.min_buffer(); // <--- hard-coded
                    let mut plot = egui_plot::Plot::new("FFT");
                    let incoming = ystud.ytf_wind.lock().unwrap().clone();
                    let incoming_size = incoming.len();
                    if incoming_size < fft_size {
                        return
                    }
                    match (ystud.ylab_state.lock().unwrap().clone()) {
                        (YLabState::Reading {version: _, port_name: _}) 
                        => {// Split inconing YTF history into a sample vector
                            //let fft_size = version.fft_size(); // <----- use this to make FFT window size dynamic
                            
                            
                            
                            let series = incoming.values();
                            // making an array of readings
                            // FFT needs n to be power of 2
                            // should be done dynamic in later versions
                            // note that hann_window is a vector, not an array
                            // so it should be possible here, too
                            /*let mut samples = Vec::with_capacity(version.fft_buffer());
                            for (i, value) in series.enumerate() {
                                if i >= version.fft_buffer() {break};
                                samples.push(value.read[0]);
                            };

                            use std::convert::AsMut;
                            fn clone_into_array<A, T>(slice: &[T]) -> A
                            where
                                A: Default + AsMut<[T]>,
                                T: Clone,
                            {
                                let mut a = A::default();
                                <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
                                a
                            }
                            
                            let sample_array = clone_into_array(&samples);*/
                            const fft_size: usize = 512;
                            let mut samples: [f32; fft_size] = [0.0; fft_size];
                            for (i,s) in series.enumerate() {
                                // let average = s.read[0] + s.read[1] + s.read[2] + s.read[3] + s.read[4] + s.read[5] + s.read[6] + s.read[7]/8.0;
                                if i >= fft_size {break};
                                let average = s.read[0]; // <--------using only the first probe for now
                                samples[i] = average as f32; 
                            }
                            // configuring the FFT engine
                            use spectrum_analyzer::scaling::divide_by_N;
                            use spectrum_analyzer::windows::hann_window;
                            use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
                            let hann_window = hann_window(&samples);
                            // calculate spectrum
                            let freq_range 
                                = FrequencyLimit::Range(ui_state.fft_min as f32, ui_state.fft_max as f32);
                            let spectrum 
                                = samples_fft_to_spectrum(
                                    // (windowed) samples
                                    &hann_window,
                                    // sampling rate
                                    incoming.rate().unwrap_or(1.0) as u32,
                                    // hard-coded frequency limit
                                    // FrequencyLimit::Range(0.5, 35.0), // covering heart rate and most neuroactivity
                                    // dynamic freq limit
                                    freq_range,
                                    // optional scale
                                    Some(&divide_by_N),);
                            match spectrum {
                                Err(e) => {println!("{:?}", e);},
                                Ok(spectrum) => {
                                    let mut points = Vec::new();
                                    for (i, (fr, fr_val)) 
                                    in spectrum.data().iter().enumerate() {
                                        points.push([fr.val() as f64, fr_val.val() as f64]);
                                    }
                                    ui.label(format!("Strongest frequencies: {}", spectrum.max().0));
                                    plot = plot
                                            .auto_bounds_x()
                                            .auto_bounds_y()
                                            .include_x(ui_state.fft_min)
                                            .include_x(ui_state.fft_max)
                                            .legend(egui_plot::Legend::default());
                                    // Plot distribution
                                    plot.show(ui, |plot_ui| {
                                        let line = egui_plot::Line::new(PlotPoints::new(points));
                                        plot_ui.line(line);
                                    });

                                }
                            }
                            // change this to dynamic by creating an empty vector with ::new()
                            // which is populated by push()
                            
                            
                            
                            
                            /*for (probe, points) in series.iter().enumerate() {
                                if ystud.ui.selected_channels.lock().unwrap()[probe] {
                                    let line = egui_plot::Line::new(PlotPoints::new(points.to_owned()));
                                    plot_ui.line(line);
                                }
                            }*/
                
                    },
                    _ => {},
                }
        
                    
                    },
                _   => {ui.label("Idle");},
            }
        }
    );
}




/// YLAB CONTROL in the right panel
/// + Connecting; reading and disconnecting
///
pub fn update_right_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    egui::SidePanel::right("left_right_panel")
        .show(ctx,|ui| {
            let mut ui_state = ystud.ui2.lock().unwrap();
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

                        // Selecting channels to plot
                        ui.heading("Channels");
                        let mut selected_channels = ystud.ui.selected_channels.lock().unwrap();
                        for (chan, b) in  selected_channels.clone().iter().enumerate(){
                            let chan_selector = ui.checkbox(&mut b.clone(), chan.to_string());
                            if chan_selector.changed() {
                                selected_channels[chan] = !b;
                            }
                        };

                        let buffer_size = yld_wind.len()/8;
                        // This avoids a block level
                        if buffer_size < version.min_buffer() {
                            ui.label("still buffering");
                            return;
                        }
                         // safe, because we have a minimum buffer size
                        let mean_interval = yld_wind.mean_time_interval().unwrap() as f64;
                        let duration = yld_wind.duration() as f64;
                        let sample_rate = buffer_size as f64/duration;
                        let nyquist = sample_rate/2.; 
                        let low_limit = duration/2.; 
                        
                        ui.label(format!("{} Hz", (sample_rate) as usize));

                        // Slider for low-pass filter
                        ui.label("Low-pass filter (Hz)");
                        let mut this_lowpass = ystud.ui.lowpass_threshold.lock().unwrap();
                        let lowpass_slider 
                            = egui::widgets::Slider::new(&mut *this_lowpass, low_limit..=nyquist)
                            .clamp_to_range(true);
                            ui.add(lowpass_slider);
                            
                            // Sliders for FFT range
                            ui.label("FFT min (Hz)");
                            let min_range = low_limit ..=(ui_state.fft_max - 1.0);
                            let fft_min_slider 
                                = egui::widgets::Slider::new(&mut ui_state.fft_min, min_range)
                                .clamp_to_range(true);
                            ui.add(fft_min_slider);

                            ui.label("FFT max (Hz)");
                            let max_range = (ui_state.fft_min + 1.)..=nyquist;
                            let fft_max_slider 
                                = egui::widgets::Slider::new(&mut ui_state.fft_max, max_range)
                                .clamp_to_range(true);
                            ui.add(fft_max_slider);
                            
                        
                        

                        /*let mut this_burnin = ystud.ui.lowpass_burnin.lock().unwrap();
                        let burnin_slider 
                            = egui::widgets::Slider::new(&mut *this_burnin, 0.0..=300.0)
                            .clamp_to_range(true);
                        ui.add(burnin_slider);*/

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
