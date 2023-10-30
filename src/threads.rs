use crate::ylab::PossPort;
use crate::ylab::{YLabState as State, ydata::*, YLabCmd};

use serialport;
use std::io::{BufReader,BufRead};
use std::sync::*;
use std::thread;
use std::time::{Duration, Instant, };
use egui::emath::History;

/// Task for reading data from serial port
/// 
/// ylab_state is used for state transitions
/// ylab_data is used for storing data
/// ylab_listen is used for listening to commands
/// 
pub fn ylab_thread(
    ylab_state: Arc<Mutex<State>>,
    ylab_data: Arc<Mutex<History<Ytf8>>>,
    ylab_listen: mpsc::Receiver<YLabCmd>,
    ) -> ! {
    

    let mut poss_port: PossPort = None;
    //let mut poss_reader: PossReader;
    let mut reader: BufReader<Box<dyn serialport::SerialPort>>;
    loop {
        // capture YLab state and incoming commands from the UI
        let this_ylab_state = ylab_state.lock().unwrap().clone();
        let this_cmd = ylab_listen.try_recv();
        // match the current state and do the transitions
        match this_ylab_state {
            State::Connected {  start_time, version, ref port} 
                => {match this_cmd {
                    Ok(YLabCmd::Read {  }) 
                        => {*ylab_state.lock().unwrap() = State::Reading {version, start_time, port: port.to_string()};},        
                      _ => {},}},
            State::Reading {  start_time, version, ref port} 
                => {let mut got_first_line: bool = false;
                    // In the previous state we've checked that the port works, so we can unwrap
                    let port = serialport::new(port.clone(), version.baud() as u32)
                                        .timeout(Duration::from_millis(1))
                                        .flow_control(serialport::FlowControl::Software)
                                        .open()
                                        .unwrap();
                    let reader = BufReader::new(port);
                    
                    for line in reader.lines() {
                        // ignore faulty lines
                        if line.is_err() {continue;};
                        // get the line
                        let line = line.unwrap();
                        // try parsing a sample from line
                        let possible_sample 
                            = Ytf8::from_csv_line(&line);
                        // print a dot when sample not valid
                        if possible_sample.is_err() {eprintln!("."); continue;}
                        // collect sample
                        let sample = possible_sample.unwrap();
                        // check if this is the first line
                        if !got_first_line {
                            let _lab_start_time = Duration::from_micros(sample.time as u64);
                            // here we can dynamically infer the data format (YTF or YLD).
                            got_first_line = true
                        }
                        let ystudio_time = (Instant::now() - start_time).as_millis() as f64;
                        ylab_data.lock().unwrap().add(ystudio_time, sample);
                        println!("{}", sample.to_csv_line());
                    
                    }},
            // Disconnected, no ports available (yet)
            State::Disconnected {ports} 
                // read list of port names from serial
                => {let avail_ports = serialport::available_ports().ok();
                    match avail_ports {
                        None => { 
                            // no ports: try again in 500ms, no transition
                            thread::sleep(Duration::from_millis(500));},
                            //ylab_state = State::Disconnected{ports: AvailablePorts::None}},
                        Some(found) => {
                            // ports found: transition to Disconnected with available ports
                            let port_names 
                                = found.iter().map(|p| p.clone().port_name)
                                    .collect::<Vec<String>>();
                            // State is updating itself by collecting available ports
                            *ylab_state.lock().unwrap() = State::Disconnected{ports: Some(port_names)};
                            match this_cmd {
                                Ok(YLabCmd::Connect { version, port })
                                => {// We make one connection attempt to verify the port
                                    // later we can add code for sending commands to the 
                                    // YLab, e.g. which sensors of a bank to collect.
                                    // If Rust holds its promise 
                                    // the serial port is properly closed when going out of scope
                                    let poss_port = 
                                        serialport::new(port.clone(),
                                            version.baud() as u32)
                                            .timeout(Duration::from_millis(1))
                                            .flow_control(serialport::FlowControl::Software)
                                            .open()
                                            .ok(); // ok() turns a Result into an Option
                                    match poss_port {
                                        None => {thread::sleep(Duration::from_millis(10))},
                                        Some(_) 
                                            => {*ylab_state.lock().unwrap() = State::Connected {start_time: Instant::now(),
                                                                                version: version, 
                                                                                port: port.clone()}}
                                    };
                                },
                                _ => {},
                            };
                        }
                    }
                }
            };
        }
    }