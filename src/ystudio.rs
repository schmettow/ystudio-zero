
pub use eframe::egui;
pub use egui::util::History;
pub use egui_plot::{PlotPoint, PlotPoints};
pub use std::sync::mpsc::Sender;
pub use std::{thread, sync::*};
pub use crate::ylab::{YLabState, YLabCmd, YLabVersion, data::*};

//use crate::ylab::AvailablePorts;

pub use crate::yui::*;
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
        update_left_panel(ctx, self);
        update_right_panel(ctx, self);
        update_central_panel(ctx, self);
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
    //opened_file: Option<PathBuf>,
    //open_file_dialog: Option<FileDialog>,
}



#[derive(Debug, Clone)]
pub struct Ystudio {
    pub ylab_state: Arc<Mutex<YLabState>>, // shared state 
    pub ylab_cmd: Sender<YLabCmd>, // sending commands to ylab
    pub yldest_state: Arc<Mutex<YldestState>>, // shared state
    pub yldest_cmd: mpsc::Sender<YldestCmd>, // sending commands to 
    pub yld_wind: Arc<Mutex<History<Yld>>>, // data stream, sort of temporal vecdeque
    pub ui: Yui, // user interface value buffer
}
