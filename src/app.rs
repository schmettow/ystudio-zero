use crate::gui;
use crate::ylab::{YLabState, YLabCmd, yld::Sample};
use eframe::egui;
use egui::util::History;
use std::sync::*;

#[derive(Debug, Clone)]
pub struct YUI {
    ylab_state: Arc<Mutex<YLabState>>, // shared state 
    ylab_data: Arc<Mutex<History<Sample>>>, // data stream, advanced vecdeque
    ylab_cmd: mpsc::Sender<YLabCmd>, // control channel
}

/*impl YUI {
    pub fn new(ylab_state) -> Self {
        YUI {history: Arc::new(Mutex::new(History::new(0..200,10.0))),
                ui: UserInput {},
                serial_data: Arc::new(Mutex::new(Vec::new())),
        }
    }


}*/

impl eframe::App for YUI {
    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        gui::update_left_panel(ctx, self);
        gui::update_right_panel(ctx, self);
        gui::update_central_panel(ctx, self);
        ctx.request_repaint();
    }
}
