pub use std::sync::mpsc::Sender;
pub use std::{thread, sync::*};
pub use eframe::egui;
pub use egui::util::History;
pub use egui_plot::{PlotPoint, PlotPoints};

//use crate::ylab::AvailablePorts;
pub use crate::ylab::{YLabState, YLabCmd, YLabVersion, ydata::*};
pub use crate::gui::*;



/// Data for the UI
/// 
/// This is sometimes necessary to hold several values in the UI 
/// (port, Version) before submitting the command to YLab
/// 
#[derive(Debug, Clone)]
pub struct Yui {
    pub y_include: Arc<Mutex<f32>>,
    pub selected_port: Arc<Mutex<Option<String>>>,
    pub selected_version: Arc<Mutex<Option<YLabVersion>>>,
    pub selected_channels: Arc<Mutex<[bool; 8]>>,
}
impl Yui {
    pub fn new () -> Self {
        Self {  y_include: Arc::new(Mutex::new(1.0)),
                selected_port: Arc::new(Mutex::new(None)),
                selected_version: Arc::new(Mutex::new(None)),
                selected_channels: Arc::new(Mutex::new([true; 8])),
        }
    }
}


#[derive(Debug)]
pub struct Ystudio {
    pub ylab_state: Arc<Mutex<YLabState>>, // shared state 
    pub yld_hist: Arc<Mutex<History<Yld>>>, // data stream, sort of temporal vecdeque
    pub ylab_cmd: mpsc::Sender<YLabCmd>, // sending commands to ylab
    pub ui: Yui, // user interface value buffer
}

impl Ystudio {
    pub fn new(ylab_cmd: Sender<YLabCmd>) -> Self {
        let ylab_state = Arc::new(Mutex::new(
                                                YLabState::Disconnected {ports: None}));
        let yld_hist = Arc::new(Mutex::new(
                                                    History::new(0..10_000,5.0)));
        let ui = Yui::new();
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
