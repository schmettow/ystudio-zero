use crate::gui;
use crate::measurements::MeasurementWindow;
use eframe::egui;
use egui::util::History;
use std::collections::HashMap;
use std::sync::*;

pub use crate::ylab::{YLab, yld::Sample};

pub struct UserInput {
    pub vars: String,
    pub y_include: String,
    pub y_include_prev: String,
    pub port: String,
    pub vars_prev: String,
    pub log_name: String,
}

pub struct Monitor {
    //pub ylab_state: Arc<Mutex<YLabState>>,
    pub ylab_version: Arc<Mutex<YLab>>,
    pub connected: Arc<Mutex<bool>>,
    pub y_include: Arc<Mutex<f32>>,
    pub measurements: Arc<Mutex<HashMap<String, MeasurementWindow>>>,
    // Alternative: history
    pub history: Arc<Mutex<History<Sample>>>,
    //pub variables: Arc<Mutex<Vec<String>>>,
    pub port: Arc<Mutex<String>>,
    pub available_ports: Arc<Mutex<Vec<String>>>,
    pub ui: UserInput,
    pub port2: String,
    pub serial_data: Arc<Mutex<Vec<String>>>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            //ylab_state: Arc::new(Mutex::new(YLabState::Disconnected)),
            ylab_version: Arc::new(Mutex::new(YLab::Mini)),
            connected: Arc::new(Mutex::new(false)),
            y_include: Arc::new(Mutex::new(0.0)),
            measurements: Arc::new(Mutex::new(HashMap::new())),
            // alternativ implementation for measurement windows
            history: Arc::new(Mutex::new(History::new(0..200,100.0))),
            port: Arc::new(Mutex::new(String::new())),
            available_ports: Arc::new(Mutex::new(Vec::new())),
            port2: String::new(),
            ui: UserInput {
                vars: String::new(),
                vars_prev: "Y0".into(),
                port: String::new(),
                y_include: String::new(),
                y_include_prev: String::new(),
                log_name: String::new(),
            },
            serial_data: Arc::new(Mutex::new(Vec::new())),
        }
    }


}

impl eframe::App for Monitor {
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
