//mod yui;
mod yldest;
mod ylab;
mod ystudio;

use ystudio::*;
use ylab::ylab_thread;
use yldest::yldest_thread;
pub use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use log::{info, warn, debug, error};


/// Creating the channels and shared states for 
/// thread-safe communication with YLab and Yldest
/// 1. mutexed states
/// 2. command channels, cmd i used in gui, YLab/Yldest threads are listening
/// 3. a Yld channel for sending data from Ylab to to Yldest
/// 4. a Yld History for sharing a sliding window with the GUI

/// fixed window sizes, could be made dynamic at a later point
const YLD_WIND_LEN:usize = 20_000;
/// used for FFT, so must be power of two
const YTF_WIND_LEN:usize = 1024;

fn main() {
    // states
    let ylab_state 
        = Arc::new(Mutex::new(ylab::YLabState::Disconnected {ports: None}));
    let yldest_state 
        = Arc::new(Mutex::new(yldest::YldestState::Idle{dir: std::env::current_dir().ok()}));
    
    // command channels
    let (ylab_cmd, ylab_listen) 
        = channel();
    let (yldest_cmd, yldest_listen) = channel();
    
    // data channel for storage
    let (yldest_send, yldest_rec) 
        = channel();
    
    // data sliding window for plotting
    let yld_wind 
        = Arc::new(Mutex::new(History::<Yld>::new(0..YLD_WIND_LEN,5.0)));
    let ytf_wind 
        = Arc::new(Mutex::new(History::<Ytf8>::new(0..YTF_WIND_LEN, 5.0)));
    
    //let (mut ytf_out, ytf_in) = spmc::channel();

    let ystud = Ystudio {
        ylab_state: ylab_state.clone(),
        ylab_cmd,
        yldest_state: yldest_state.clone(),
        yldest_cmd,
        yld_wind: yld_wind.clone(),
        ytf_wind: ytf_wind.clone(),
        ui: Yui {
            selected_port: Arc::new(Mutex::new(None)),
            selected_version: Arc::new(Mutex::new(None)),
            selected_channels: Arc::new(Mutex::new([false; 8])),
            lowpass_threshold: Arc::new(Mutex::new(55.0)),
            lowpass_burnin: Arc::new(Mutex::new(0.0)),
            frequency_range: Arc::new(Mutex::new(spectrum_analyzer::FrequencyLimit::Range(1.0, 55.0))), // <- not used
        },
        // experimental: Can we go with a global lock?
        ui2: Arc::new(Mutex::new(Yui2 {
            selected_port: None,
            selected_version: None,
            selected_channels: [false; 8],
            lowpass_threshold: 40.0,
            lowpass_burnin: 0.0,
            frequency_range: spectrum_analyzer::FrequencyLimit::Range(1.0, 55.0), // <-- global lock works
        })),
    };

    // The ystudio object contains three coponents in a thread safe manner
    // + ylab_state, which is a mutexed YLabState
    // + yld_wind, which is a egui History of YLab Samples in Yld format
    // + ytf_wind, which is a egui History of samples in Ytf8 format
    // + ui, which captures UI related variables
    // let ystudio_1 = Ystudio::new(ylab_cmd, yldest_cmd);
    // let ystudio_2 = ystudio_1.clone();
    
    // The thread to collect Ylab data is started
    // consuming copies of ylab state, data and command listener
    thread::spawn(move || {
        ylab_thread(
            ylab_state,
            ylab_listen,
            yld_wind,
            ytf_wind,
            yldest_send,
        );
    });


    // Storage thread
    thread::spawn(move || {
        yldest_thread( 
            yldest_state,
            yldest_listen,
            yldest_rec,
        );
    });

    // starting the egui, consuming the ystudio object.
    // The details of the GUI are in ystudio.rs.
    // The below works, because Ystudio objects implement eframe::App.

    ystudio::egui_init(ystud.clone());
}
