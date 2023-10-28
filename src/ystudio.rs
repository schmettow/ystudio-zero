pub use std::sync::mpsc::Sender;
pub use std::{thread, sync::*};
pub use eframe::egui;
pub use egui::util::History;

pub use crate::ylab::{YLabState, YLabCmd, YLabVersion, yld::Sample};
pub use crate::gui::*;

#[derive(Debug)]
pub struct Yui {
    pub y_include: Arc<Mutex<f32>>,
}

impl Yui {
    pub fn new () -> Self {
        Self {
            y_include: Arc::new(Mutex::new(1.0))
        }
    }
}


#[derive(Debug)]
pub struct Ystudio {
    pub ylab_state: Arc<Mutex<YLabState>>, // shared state 
    pub ylab_data: Arc<Mutex<History<Sample>>>, // data stream, advanced vecdeque
    pub ylab_cmd: mpsc::Sender<YLabCmd>, // sending commands to ylab
    pub ui: Yui, // sending commands to ylab
}

impl Ystudio {
    pub fn new(ylab_cmd: Sender<YLabCmd>) -> Self {
        let ylab_state = Arc::new(Mutex::new(YLabState::Disconnected {ports: None}));
        let ylab_data = Arc::new(Mutex::new(History::new(0..200,100.0)));
        let ui = Yui::new();
        Self {
            ylab_state,
            ylab_data,
            ylab_cmd,
            ui,
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
