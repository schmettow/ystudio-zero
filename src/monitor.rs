/*
Created by: Andrei Litvin
https://github.com/andy31415/rs-value-plotter

*/

use crate::gui;
use crate::measurements::MeasurementWindow;
use eframe::egui;
use std::collections::HashMap;
use std::sync::*;

pub struct AppUserInput {
    pub vars: String,
    pub y_include: String,
    pub y_include_prev: String,
    pub port: String,
    pub vars_prev: String,
    pub log_name: String,
}

pub struct MonitorApp {
    pub y_include: Arc<Mutex<f32>>,
    pub measurements: Arc<Mutex<HashMap<String, MeasurementWindow>>>,
    pub variables: Arc<Mutex<Vec<String>>>,
    pub port: Arc<Mutex<String>>,
    pub available_ports: Arc<Mutex<Vec<String>>>,
    pub ui: AppUserInput,
    pub port2: String,
    pub serial_data: Arc<Mutex<Vec<String>>>,
    pub send_serial: Arc<Mutex<bool>>,
    pub serial_write: Arc<Mutex<String>>,
}

impl MonitorApp {
    pub fn new() -> Self {
        Self {
            y_include: Arc::new(Mutex::new(0.0)),
            measurements: Arc::new(Mutex::new(HashMap::new())),
            //variables: Arc::new(Mutex::new(Vec::new())),
            variables: Arc::new(Mutex::new(vec!( "y0".to_string(), "y1".to_string(), "y2".to_string(), "y3".to_string(),
            "y4".to_string(), "y5".to_string(), "y6".to_string(), "y7".to_string()))),
            port: Arc::new(Mutex::new(String::new())),
            available_ports: Arc::new(Mutex::new(Vec::new())),
            port2: String::new(),
            ui: AppUserInput {
                vars: String::new(),
                vars_prev: "Y0".into(),
                port: String::new(),
                y_include: String::new(),
                y_include_prev: String::new(),
                log_name: String::new(),
            },
            serial_data: Arc::new(Mutex::new(Vec::new())),
            send_serial: Arc::new(Mutex::new(false)),
            serial_write: Arc::new(Mutex::new(String::new())),
        }
    }


}

impl eframe::App for MonitorApp {
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
