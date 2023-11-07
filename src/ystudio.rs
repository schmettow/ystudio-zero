
pub use eframe::egui;
pub use egui::util::History;
pub use egui_plot::{PlotPoint, PlotPoints};

//use crate::ylab::AvailablePorts;

pub use crate::gui::*;



pub mod yui{
    pub use std::sync::mpsc::Sender;
    pub use std::{thread, sync::*};
    pub use crate::ylab::{YLabState, YLabCmd, YLabVersion, ydata::*};
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
}
impl Yui {
    pub fn new () -> Self {
        Self {  selected_port: Arc::new(Mutex::new(None)),
                selected_version: Arc::new(Mutex::new(None)),
                selected_channels: Arc::new(Mutex::new([true; 8])),
        }
    }
}
}

pub mod yldstor {
    use std::path::PathBuf;
    #[derive(Clone)]
    pub enum YldStoreState {
        Idle,
        Recording {path: PathBuf},
        Pausing {path: PathBuf}}

    /*impl Clone for State {
        fn clone(&self) -> State {
            match self {
                State::Idle => State::Idle,
                State::Recording {file} => State::Recording {file: file.to_owned()},
                State::Pausing {file} => State::Pausing {file: file.clone()},
            }
        }
    }*/

    #[derive(Clone)]
    pub enum YldStoreCmd {
        New {path: PathBuf},
        Pause,
        Resume,
        Stop,
    }

}


pub use std::sync::mpsc::Sender;
pub use std::{thread, sync::*};
pub use crate::ylab::{YLabState, YLabCmd, YLabVersion, ydata::*};

#[derive(Debug)]
pub struct Ystudio {
    pub ylab_state: Arc<Mutex<YLabState>>, // shared state 
    pub yld_hist: Arc<Mutex<History<Yld>>>, // data stream, sort of temporal vecdeque
    pub ylab_cmd: mpsc::Sender<YLabCmd>, // sending commands to ylab
    pub ui: yui::Yui, // user interface value buffer
}

impl Ystudio {
    pub fn new(ylab_cmd: Sender<YLabCmd>) -> Self {
        let ylab_state = Arc::new(Mutex::new(
                                                YLabState::Disconnected {ports: None}));
        let yld_hist = Arc::new(Mutex::new(
                                                    History::new(0..10_000,5.0)));
        let ui = yui::Yui::new();
        Self {  ylab_state,
                yld_hist,
                ylab_cmd,
                ui,
        }
    }
}

impl Clone for Ystudio {
    fn clone(&self) -> Self {
        Self {  ylab_state: self.ylab_state.clone(),
                yld_hist: self.yld_hist.clone(),
                ylab_cmd: self.ylab_cmd.clone(),
                ui: self.ui.clone(),
        }
    }
}

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
        update_left_panel(ctx, self);
        update_right_panel(ctx, self);
        update_central_panel(ctx, self);
        ctx.request_repaint();
    }
}
