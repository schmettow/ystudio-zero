
pub use eframe::egui;
// use egui::load::LoadError;
pub use egui::util::History;
use egui::Ui;
// use egui_plot::Plot;
pub use egui_plot::PlotPoints;
pub use std::sync::mpsc::Sender;
pub use std::{thread, sync::*};
pub use crate::ylab::{YLabState, YLabCmd, YLabVersion, data::*};
pub use crate::ylab::*;
pub use crate::yldest::*;


impl eframe::App for Ystudio {
    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    
    /*//#[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }*/

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
    pub ytf_wind: Arc<Mutex<Banks>>, // data stream, sort of temporal vecdeque, one per sensory
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataView{
    None, 
    Log, 
    Plot,
    PlotFft,
}

#[derive(Debug, Clone)]
pub struct Yui {
    pub selected_port: Option<String>,
    pub selected_version: Option<YLabVersion>,
    pub selected_bank: u8,
    pub selected_channels: [bool; 8],
    pub view: DataView,
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
        // transparent: true,
        // initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        // resizable: true,
        ..Default::default()
    };
    //egui_logger::builder().init().unwrap();
    eframe::run_native(
        "Ystudio Zero", // unused title
        options,
        Box::new(|_cc| Ok(Box::new(ystud))),
    ).unwrap();
}

use egui::ecolor::Color32;
const LINE_COLORS: [Color32; 8] 
    = [Color32::BLACK,
       Color32::DARK_BLUE,
       Color32::DARK_GREEN,
       Color32::DARK_RED,
       Color32::DARK_GRAY,
       Color32::BLUE,
       Color32::GREEN,
       Color32::RED
        ];




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
                    None => YLabVersion::Go,};
            
            match ylab_state.clone() {
                // Selecting port and YLab version
                // When both are selected, the connect button is shown
                YLabState::Disconnected {ports}
                    => {ui.heading("Disconnected");              
                        // When ports are available, show the options
                        match ports {
                            None => {ui.label("Searching ports");
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
                                ui.separator();
                                // one selectable per version
                                ui.label("Version");
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Pro, "Pro")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::Pro);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::ProMotion(1), "Pro Motion 1")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::ProMotion(1));
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Go, "Go")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::Go);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::GoMotion(4), "Go Motion 4")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::GoMotion(4));
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::GoStress, "Go Stress")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::GoStress);
                                }
                                if ui.add(egui::SelectableLabel::new(selected_version == YLabVersion::Mini, "Mini")).clicked() { 
                                    ui_state.selected_version = Some(YLabVersion::Mini);
                                }
                                ui.separator();
                                // The button is only shown when version and port are selected (which currently is by default).
                                // It commits the connection command to the YLab thread.
                                //match ( ystud.ui.selected_version.lock().unwrap().clone(), ystud.ui.selected_port.lock().unwrap().clone())  {
                                match (ui_state.selected_version, ui_state.selected_port.clone())  {
                                    (Some(version), Some(port)) 
                                        =>  if ui.button("Connect")
                                                .on_hover_text("Connect to YLab")
                                                .clicked()  {
                                                    ystud.ylab_cmd.send(  YLabCmd::Connect {version: version, port_name: port.to_string()}).unwrap();
                                                },
                                        _ => {ui.label("Select port and version");}
                                }
                                 
                            } 
                        }
                    },
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
                    => {
                        // Get incoming data
                        let yld_wind = ystud.yld_wind.lock().unwrap();
                        // Info header
                        ui.heading("Reading");
                        ui.label(format!("{}:{}", version, port_name));
                        // Disconnect
                        if ui.button("Disconnect").on_hover_text("Disconnects YLab").clicked(){
                            ystud.ylab_cmd.send(YLabCmd::Disconnect {}).unwrap();
                            ui_state.view = DataView::None;
                            //println!("Cmd: Stop")
                        };
                        ui.separator();

                        // View
                        ui.label("View");
                        egui::ComboBox::from_label("select View")
                            .selected_text(format!("{:?}", ui_state.view))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut ui_state.view, DataView::PlotFft, "Plot + FFT");
                                ui.selectable_value(&mut ui_state.view, DataView::Plot, "Plot");
                                ui.selectable_value(&mut ui_state.view, DataView::Log, "Log");
                                ui.selectable_value(&mut ui_state.view, DataView::None, "None");
                            });
                        // Bank selector (if more than one)
                        let n_banks = version.n_banks();
                        
                        if n_banks > 1 {
                            ui.heading("Sensory");
                            ui.add(egui::Slider::new(&mut ui_state.selected_bank, 1..=(version.n_banks())).text(""));
                            ui.label(format!("Sensory {} of {}", &ui_state.selected_bank, n_banks));
                            ui.separator();
                        }

                        /*let bank_slider: egui::Slider<'_> 
                            = egui::widgets::Slider::new(&mut ui_state.selected_bank, 0..=7)
                            .clamp_to_range(true)
                            .fixed_decimals(0);*/
                        
                        //println!("Banks");
                        /*let banks = version.bank_labels();
                        println!("Selected Banks");
                        for (bank, label) in  banks.iter().enumerate() {
                            ui.checkbox( &mut ui_state.selected_bank, label.to_string());
                        };*/
                        
                        
                        // Selecting channels to plot
                        ui.heading("Channels");
                        let selected_channels = ui_state.selected_channels.clone();
                        for (chan, _label) in  selected_channels.iter().enumerate() {
                            //ui.menu_image_text_button(image, title, add_contents)
                            ui.horizontal(|ui| {
                                ui.checkbox( &mut ui_state.selected_channels[chan], "");
                                // ui.checkbox(&mut checked, &label);
                                // Set the color for the label
                                ui.label(egui::RichText::new(chan.to_string()).color(LINE_COLORS[chan]));
                            });
                        };
                        
                        if ui_state.selected_bank > 0 {
                            let buffer_size = yld_wind.len()/8;
                            
                            
                            /*
                            // Check for buffer under-run
                            if buffer_size < version.fft_size() {
                                ui.label("still buffering");
                                if ui.button("Stop Reading").on_hover_text("Stop reading").clicked(){
                                    ystud.ylab_cmd.send(YLabCmd::Stop {}).unwrap(); 
                                    println!("Cmd: Stop")};
                                return;
                            }
                             */
                            
                            
                            let duration = yld_wind.duration() as f64; // <------ Should be based on YTF stream
                            let sample_rate = buffer_size as f64/duration;
                            let nyquist = sample_rate/2.; 
                            let low_limit = duration/2.; 
                            ui.separator();
                            ui.label(format!("{} Hz per channel", sample_rate as usize));
                            ui.separator();

                            ui.heading("Raw signal");
                            // Slider for low-pass filter
                            ui.label("Low-pass filter (Hz)");
                            //let mut this_lowpass = ystud.ui.lowpass_threshold.lock().unwrap(); 
                            let lowpass_slider 
                                = egui::widgets::Slider::new(&mut ui_state.lowpass_threshold, low_limit..=nyquist)
                                //.clamp_to_range(true)
                                .logarithmic(true)
                                .fixed_decimals(3);
                            ui.add(lowpass_slider);
                            
                            // ui.separator();
                            // ui.heading("FFT");
                            // // Sliders for FFT range
                            // ui.label("min (Hz)");
                            // let min_range = 0. ..=(nyquist - 5.);
                            // let fft_min_slider 
                            //     = egui::widgets::Slider::new(&mut ui_state.fft_min, min_range)
                            //     //.clamping(true)
                            //     .logarithmic(true)
                            //     .fixed_decimals(3);
                            // ui.add(fft_min_slider);

                            // ui.label("max (Hz)");
                            // let max_range = (ui_state.fft_min + 2.)..=(nyquist - 5.);
                            // let fft_max_slider 
                            //     = egui::widgets::Slider::new(&mut ui_state.fft_max, max_range)
                            //     //.clamp_to_range(true)
                            //     .fixed_decimals(1);
                            // ui.add(fft_max_slider);
                            // ui.separator();
                        }
                        
                        
                        ui.heading("Recording");
                        // Start or stop recording
                        // asking the state of recording thread
                        let yldest_state = ystud.yldest_state.lock().unwrap().clone();
                        match yldest_state {
                            YldestState::Idle{ dir: Some(_dir) }
                            => {
                                ui.label("Idle");
                                if ui.button("New Rec")
                                    .on_hover_text("Start a new recording")
                                    .clicked() {
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

                
                }
            });
        }


/// updates the plotting area
/// 
/// + one line per active channel
/// + reading the Yld stream
pub fn update_central_panel(ctx: &egui::Context, ystud: &mut Ystudio) 
{   egui::CentralPanel::default().show(ctx, |ui| {
        let mut plot = egui_plot::Plot::new("plotter");
        let ui_state = ystud.ui.lock().unwrap();
    
        match ystud.ylab_state.lock().unwrap().clone() {
            YLabState::Connected {version: _, port_name: _} => {
            },
            YLabState::Reading {version: _, port_name: _}
            => {// Handle an empty buffer
                let incoming= ystud.yld_wind.lock().unwrap().clone();
                if incoming.is_empty() {
                    ui.label(format!("Sensory buffer empty"));
                    return
                }
                let incoming 
                        = &ystud.ytf_wind.lock().unwrap().clone()
                            [ui_state.selected_bank as usize];
                
                if incoming.is_empty() {
                    ui.label(format!("Sensory buffer empty"));
                    return       // very important! Otherwise the below can crash because of emtoy buffer
                }

                //let incoming= &ystud.ytf_wind.lock().unwrap().clone()[ui_state.selected_bank as usize];
                ui.label(format!("Sensory {}", ui_state.selected_bank));
                // Split inconing history into points series
                
                match ui_state.view {
                    DataView::None => {}, 
                    DataView::Log => {
                        egui::ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .max_height(ui.available_height() - 30.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            let incoming 
                                = &ystud.ytf_wind.lock().unwrap().clone()
                                    [ui_state.selected_bank as usize];
                        
                            if incoming.is_empty() {
                                ui.label(format!("Sensory buffer empty"));
                                return
                                }

                            for r in incoming.iter() {
                                let (_, ytf)  = r;
                                ui.label(format!("{:?}", ytf));
                                };
                            });
                            },
                            DataView::Plot | DataView::PlotFft => {
                                plot = plot
                                .auto_bounds([true, true].into())
                                //.auto_bounds_x()
                                .include_y(0.0) 
                                //.auto_bounds_y()
                                .legend(egui_plot::Legend::default());
                                plot.show(ui, |plot_ui| {                
                                    let rate = incoming.rate().unwrap(); // safe because above we check for empty buffer
                                    let series  = incoming.split();
                                    for (chan, active) in ui_state.selected_channels.iter().enumerate() {
                                        // inactive channels
                                        if !active | (series[chan].len() < 25) {
                                            continue
                                        }

                                        // filter                            
                                        use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};
                                        let lowpass = ui_state.lowpass_threshold as f32;
                                        let coeffs 
                                            = Coefficients::<f32>::from_params( Type::LowPass, 
                                                                                rate.hz(), 
                                                                                lowpass.hz(), 
                                                                                Q_BUTTERWORTH_F32);
                                        
                                        match coeffs {
                                            Err(e) => println!("{:?}", e),
                                            Ok(coeffs) => {
                                                let mut biquad_lpf 
                                                        = DirectForm1::<f32>::new(coeffs);
                                                        let mut filtered_points: VecDeque<[f64; 2]> = VecDeque::new();
                                                series[chan].iter()
                                                        .for_each(|point| filtered_points
                                                                                        .push_front([point[0], biquad_lpf.run(point[1] as f32) as f64]));
                                                // Calculate lowpass burnin
                                                let burnin = 2 * (rate/lowpass) as usize + 1; // <--- formula for low pass burnin
                                                // removing the burnin period, sadly changes scrolling speed
                                                for _ in 0..(burnin as usize) {
                                                    filtered_points.pop_back();
                                                }
                                                // PLot the line
                                                let filtered_line = egui_plot::Line::new(PlotPoints::new(filtered_points.to_owned().into()))
                                                    .color(LINE_COLORS[chan]);
                                                plot_ui.line(filtered_line);
                    
                                            }
                                        }     
                                    }
                            });
                    },
                }

                
                
                // Plot lines: CLOSURE
                
            },
            _ => {},
        }
    });
}


/// updates bottom panel with FFT
pub fn update_bottom_panel(ctx: &egui::Context, ystud: &mut Ystudio) {
    
    egui::TopBottomPanel::bottom("bottom_panel")
        .show(ctx, |ui| {
            let ylab_state = ystud.ylab_state.lock().unwrap().clone();
            let mut ui_state = ystud.ui.lock().unwrap();
            // First bank is events
            if ui_state.selected_bank == 0 {return}
            
            match (ylab_state, ui_state.view) {
                // Plot a spectrogramm
                (YLabState::Reading {version, port_name:_}, DataView::PlotFft)
                => {ui.heading("Distribution of Frequencies");
                    // fetching data from YLab
                    let incoming 
                        = &ystud.ytf_wind.lock().unwrap().clone()
                            [ui_state.selected_bank as usize]; 
                    let duration = incoming.duration() as f64;
                    let n_samples = incoming.len();
                    let sample_rate= n_samples as f64/duration;
                    let nyquist = sample_rate/2.; 
                    let lowest_freq = 1./(duration * 2.);
                    // Handling buffer under-runs
                    
                    let fft_size = version.fft_size();
                    if n_samples < fft_size {
                        ui.label(format!("still buffering ... {:.1}%", n_samples as f32/fft_size as f32 * 100.0));
                        return
                    }
                    // Sliders for FFT range
                    ui.label("min (Hz)");
                    let min_range_max =   // This because max() not implemented for float
                        match nyquist > ui_state.fft_max {
                            true => ui_state.fft_max,
                            false => nyquist
                        };
                    let min_range = lowest_freq ..= (min_range_max - 3.);
                    let fft_min_slider 
                        = egui::widgets::Slider::new(&mut ui_state.fft_min, min_range)
                        .clamping(egui::SliderClamping::Always)
                        .logarithmic(true)
                        .fixed_decimals(3);
                    ui.add(fft_min_slider);
                    ui.label("max (Hz)");
                    let max_range = (ui_state.fft_min + 3.)..=(nyquist);
                    let fft_max_slider 
                        = egui::widgets::Slider::new(&mut ui_state.fft_max, max_range)
                        .clamping(egui::SliderClamping::Always)
                        .logarithmic(true)
                        .fixed_decimals(1);
                    ui.add(fft_max_slider);
                    ui.separator();

                    // configuring the plot
                    use spectrum_analyzer::scaling::divide_by_N;
                    use spectrum_analyzer::windows::hann_window;
                    use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
                    // Creating the plotter
                    // exact size of data window for FFT
                    
                    // Collect the FFT window per channel
                    // Vector of channels of samples
                    let mut samples: Vec<Vec<f32>> = vec![vec![]; 8];
                    for ytf8 in incoming.values().collect::<Vec<Ytf8>>()[0..fft_size].iter() {
                        for (chan, active) in ui_state.selected_channels.iter().enumerate() {
                            if *active {samples[chan].push(ytf8.read[chan] as f32)}
                        }
                    } 
                    
                    let mut plot = egui_plot::Plot::new("FFT");
                    plot = plot
                            .auto_bounds([true, true].into())
                            //.include_x(ui_state.fft_min)
                            .include_x(ui_state.fft_max)
                            .legend(egui_plot::Legend::default());
                    
                    // CLOSURE TIME!! Mind the brackets.
                    plot.show(ui, |plot_ui| {
                        for (chan, sample) in samples.iter().enumerate(){
                            let mut points = Vec::new();
                            // empty line for inactive channels
                            if !ui_state.selected_channels[chan] {
                                let line = egui_plot::Line::new(PlotPoints::new(points));
                                plot_ui.line(line);
                                continue
                            }

                            // Acive channel
                            //if sample.len() < 10 {return};
                            let hann_window = hann_window(sample.as_slice());
                            // get frequency limits from ui
                            /*if ui_state.fft_min > ui_state.fft_max {
                                ui.label("Incorrect range");
                                return
                            }*/
                            if ui_state.fft_max <= ui_state.fft_min + 2. {
                                panic!("Incorrect frequency band {} .. {}", ui_state.fft_min, ui_state.fft_max)
                            }
                            let freq_range
                                = FrequencyLimit::Range(ui_state.fft_min as f32, ui_state.fft_max as f32);
                            // compute the possible power spectrum
                            let spectrum 
                                = samples_fft_to_spectrum(
                                    &hann_window,
                                    incoming.rate().unwrap_or(1.0) as u32,
                                    freq_range,
                                    Some(&divide_by_N),);
                            // plotting with error handling
                            match spectrum {
                                Err(e) => {println!("FFT: {:?}", e);},
                                Ok(spectrum) => {
                                    for (freq, ampl) 
                                    in spectrum.data().iter() {
                                        points.push([freq.val() as f64, ampl.val() as f64]);
                                    }
                                    let line = egui_plot::Line::new(PlotPoints::new(points));
                                    plot_ui.line(line);
                                }
                            }
                        }                            
                    });
                    // Plot distribution
                    
                    

                },
                _   => {ui.label("Idle");},
            }
        }
    );
}





/// Updates left control area (unused)
/// 
/// 

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
        


#[allow(unused_imports)]
use egui_file::FileDialog;

