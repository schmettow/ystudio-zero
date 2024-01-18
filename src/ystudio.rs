
pub use eframe::egui;
pub use egui::util::History;
pub use egui_plot::PlotPoints;
pub use std::sync::mpsc::Sender;
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
/// + ui, which captures UI related variables with one global lock

#[derive(Clone)]
pub struct Ystudio {
    pub ylab_state: Arc<Mutex<YLabState>>, // shared state 
    pub ylab_cmd: Sender<YLabCmd>, // sending commands to ylab
    pub yldest_state: Arc<Mutex<YldestState>>, // shared state
    pub yldest_cmd: mpsc::Sender<YldestCmd>, // sending commands to control storage
    pub yld_wind: Arc<Mutex<History<Yld>>>, // data stream, sort of temporal vecdeque
    pub ytf_wind: Arc<Mutex<History<Ytf8>>>, // data stream, sort of temporal vecdeque
    pub ui: Arc<Mutex<Yui>>, // ui parameters with outer lock, more convenient
}

/// Data for the UI
/// 
/// This is sometimes necessary to hold several values in the UI 
/// (port, Version) before submitting the command to YLab
/// 
/// 
/// Structure holding the parameters of the ui
/// 
/// such as:
/// 1.  serial port connected to YLab
/// 1.  selecting channels for display
/// 1.  keeping filter controls in the safe range of the underlying algorithm. 
///     The upper frequency limit is always given by the *Nyquist* frequency. But for the lower limit, it differs.
///
/// For *low pass filters* zero is a possible lower limit, effectively switching the filter off. However, when 
/// passing through the range (1, 0), wavelengths become very long. Apparently, the used algorithm 
/// 
/// , and the value buffer becomes the limiting factor. 


#[derive(Debug, Clone)]
pub struct Yui {
    pub selected_port: Option<String>,
    pub selected_version: Option<YLabVersion>,
    pub selected_channels: [bool; 8],
    pub lowpass_threshold: f64,
    pub fft_min: f64,
    pub fft_max: f64
    //opened_file: Option<PathBuf>,
    //open_file_dialog: Option<FileDialog>,
}
    

/// Because we have a window, we use a double ended queue
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
        let ui_state = ystud.ui.lock().unwrap();
        match ystud.ylab_state.lock().unwrap().clone() {
            YLabState::Reading {version: _, port_name: _} 
            => {// Split inconing history into points series
                let incoming: egui::util::History<data::Yld> = ystud.yld_wind.lock().unwrap().clone();
                let series 
                    = incoming.split();
                plot = plot
                        .auto_bounds_x()
                        .include_y(0.0) 
                        .auto_bounds_y()
                        .legend(egui_plot::Legend::default());
                let legend = egui_plot::Legend::default();
                plot = plot.legend(legend);
                // Plot lines
                plot.show(ui, |plot_ui| {

                let selected_channels = ui_state.selected_channels;
                for (probe, points) in &mut series.clone().iter().enumerate() {
                    if selected_channels[probe] {
                        // applying the low-pass filter
                        use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};
                        let sampling_rate = incoming.rate();
                        match sampling_rate {
                            None => {},
                            Some(rate) => {
                                // suppress the burn-in phase, dependent on frequency
                                // Create coefficients for the biquads
                                let lowpass = ui_state.lowpass_threshold as f32;
                                let coeffs 
                                    = Coefficients::<f32>::from_params( Type::LowPass, 
                                                                        rate.hz(), 
                                                                        lowpass.hz(), 
                                                                        Q_BUTTERWORTH_F32);
                                match coeffs {
                                    Err(e) => println!("{:?}", e),
                                    Ok(coeffs) => {
                                        let mut filtered_points: VecDeque<[f64; 2]> = VecDeque::new();
                                        let mut biquad_lpf 
                                                = DirectForm1::<f32>::new(coeffs);
                                        points  .iter()
                                                .for_each(|point| filtered_points
                                                                              .push_front([point[0], biquad_lpf.run(point[1] as f32) as f64]));
                                        // 
                                        let burnin = 2 * (rate/lowpass) as usize + 1; // <--- formula for low pass burnin
                                        for _ in 0..(burnin as usize) {
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
            let ui_state = ystud.ui.lock().unwrap();
            
            match ylab_state {
                // show New button when Reading and Idle
                YLabState::Reading {version, port_name:_}
                => {
                    let fft_size = version.min_buffer();
                    let mut plot = egui_plot::Plot::new("FFT");
                    let incoming = ystud.ytf_wind.lock().unwrap().clone();
                    let incoming_size = incoming.len();
                    // 
                    if incoming_size < fft_size {
                        ui.label(format!("still buffering ... {}%", incoming_size/fft_size * 100));
                        return
                    }
                    // Split inconing YTF history into a sample vector
                    let mut ytf8 = incoming.values();
                    // FFT works with fixed sample sizes of power 2. Comnfigured by YLab version.
                    let fft_size = version.fft_size();
                    let mut samples: Vec<f32> = Vec::with_capacity(fft_size);
                    // collecting the fft data
                    for _ in 0..fft_size {
                        // using the low-level iterator
                        match ytf8.next(){
                            // For FFT we need the data in the exact size
                            // so we check the result of the iterator
                            None => {println!("FFT buffer underrun")},
                            Some(sample) => {samples.push(sample.read[0] as f32)}
                        }
                    }
                    // configuring the FFT engine
                    use spectrum_analyzer::scaling::divide_by_N;
                    use spectrum_analyzer::windows::hann_window;
                    use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
                    let hann_window = hann_window(samples.as_slice());
                    // calculate spectrum
                    let freq_range 
                        = FrequencyLimit::Range(ui_state.fft_min as f32, ui_state.fft_max as f32);
                    let spectrum 
                        = samples_fft_to_spectrum(
                            &hann_window,
                            incoming.rate().unwrap_or(1.0) as u32,
                            freq_range,
                            Some(&divide_by_N),);
                    match spectrum {
                        Err(e) => {println!("{:?}", e);},
                        Ok(spectrum) => {
                            let mut points = Vec::new();
                            for (fr, fr_val) 
                            in spectrum.data().iter() {
                                points.push([fr.val() as f64, fr_val.val() as f64]);
                            }
                            ui.label(format!("Strongest frequencies: {}", spectrum.max().0));
                            ui.label(format!("FFT size: {}", version.fft_size()));
                            plot = plot
                                    .auto_bounds_x()
                                    .auto_bounds_y()
                                    .include_x(freq_range.min())
                                    .include_x(freq_range.max())
                                    .legend(egui_plot::Legend::default());
                            // Plot distribution
                            plot.show(ui, |plot_ui| {
                                let line = egui_plot::Line::new(PlotPoints::new(points));
                                plot_ui.line(line);
                            });

                        }
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
            let mut ui_state = ystud.ui.lock().unwrap();
            let ylab_state = ystud.ylab_state.lock().unwrap();
            // setting defaults
            let selected_version 
                = match ui_state.selected_version.clone() {
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
                        //let mut selected_channels = ystud.ui.selected_channels.lock().unwrap();
                        let selected_channels = ui_state.selected_channels;
                        for (chan, _) in  selected_channels.iter().enumerate() {
                            ui.checkbox( &mut ui_state.selected_channels[chan], 
                                                    chan.to_string());
                        };

                        let buffer_size = yld_wind.len()/8;
                        // Check for buffer under-run
                        if buffer_size < version.fft_size() {
                            ui.label("still buffering");
                            return;
                        }
                        let duration = yld_wind.duration() as f64;
                        let sample_rate = buffer_size as f64/duration;
                        let nyquist = sample_rate/2.; 
                        let low_limit = duration/2.; 
                        
                        ui.label(format!("{} Hz per channel", sample_rate as usize));
                        ui.heading("Signal Processing");
                        // Slider for low-pass filter
                        ui.label("Low-pass filter (Hz)");
                        //let mut this_lowpass = ystud.ui.lowpass_threshold.lock().unwrap(); 
                        let lowpass_slider 
                            = egui::widgets::Slider::new(&mut ui_state.lowpass_threshold, low_limit..=nyquist)
                            .clamp_to_range(true)
                            .logarithmic(true)
                            .fixed_decimals(3);
                        ui.add(lowpass_slider);
                            
                        // Sliders for FFT range
                        ui.label("FFT min (Hz)");
                        let min_range = 0. ..=(nyquist - 2.);
                        let fft_min_slider 
                            = egui::widgets::Slider::new(&mut ui_state.fft_min, min_range)
                            .clamp_to_range(true)
                            .logarithmic(true)
                            .fixed_decimals(3);
                        ui.add(fft_min_slider);

                        ui.label("FFT max (Hz)");
                        let max_range = (ui_state.fft_min + 2.)..=nyquist - 2.;
                        let fft_max_slider 
                            = egui::widgets::Slider::new(&mut ui_state.fft_max, max_range)
                            .clamp_to_range(true)
                            .fixed_decimals(1);
                        ui.add(fft_max_slider);
                            
                        // Stop reading
                        //
                        // send command to YLab
                        if ui.button("Stop Reading").on_hover_text("Stop reading").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Stop {}).unwrap(); 
                            println!("Cmd: Stop")};
                        
                        ui.heading("Recording");
                        // Start or stop recording
                        // asking the state of recording thread
                        let yldest_state = ystud.yldest_state.lock().unwrap().clone();
                        match yldest_state {
                            YldestState::Idle{ dir: Some(_dir) }
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
                                    //= match ystud.ui.selected_port.lock().unwrap().clone() {
                                    = match ui_state.selected_port.clone() {
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
                                    if ui.add(egui::SelectableLabel::new(selected_port == Some((*i).to_string()), 
                                                                                i.to_string())).clicked() { 
                                        ui_state.selected_port = Some(i.clone())
                                        //*ystud.ui.selected_port.lock().unwrap() = Some(i.clone());
                                    }
                                };
                                // one selectable per version
                                ui.label("Version");
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Pro, "Pro")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::Pro);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Go, "Go")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::Go);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Mini, "Mini")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::Mini);
                                }
                                // The button is only shown when version and port are selected (which currently is by default).
                                // It commits the connection command to the YLab thread.
                                //match ( ystud.ui.selected_version.lock().unwrap().clone(), ystud.ui.selected_port.lock().unwrap().clone())  {
                                match (ui_state.selected_version, ui_state.selected_port.clone())  {
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

#[allow(dead_code)]
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
