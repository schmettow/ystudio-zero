//use crate::helpers;
use crate::measurements;
use crate::measurements::MeasurementWindow;
use crate::ylab::{YLabState as State, yld::{Sample, self}};
//use crate::app::self;

use serialport;
use std::collections::HashMap;
use std::io::{BufReader, BufRead};
use std::sync::*;
use std::thread;
use std::time::{Duration, Instant};
use egui::emath::History;


pub fn serial_thread(
    ylab: Arc<Mutex<State>>,
    measurements: Arc<Mutex<HashMap<String, MeasurementWindow>>>,
    //connected: Arc<Mutex<bool>>,
    _history: Arc<Mutex<History<Sample>>>,
    //serial_port: Arc<Mutex<String>>,
    //available_ports: Arc<Mutex<Vec<String>>>,
    //serial_data: Arc<Mutex<Vec<String>>>,
    ) -> ! {
    
    // Connecting to serial port
    let ylab_state = ylab.lock().unwrap();
    loop {
        match *ylab_state {
            State::Disconnected => {
                let ports = serialport::available_ports();
                match ports {
                    Err(_) => {thread::sleep(Duration::from_millis(500))},
                    Ok(n) => {
                        let port_names 
                            = n.iter().map(|p| p.port_name.clone()).collect::<Vec<String>>();
                        *ylab_state = State::Available {ports: port_names}
                        }
                    }
            },
            State::Available {ports} => {
                // wait for user input
            },
            State::ConnRequest {version, port} => {
                let poss_port: Result<Box<dyn serialport::SerialPort>, serialport::Error> 
                    = serialport::new(port.clone(),
                        version.baud() as u32)
                        .timeout(Duration::from_millis(1))
                        .flow_control(serialport::FlowControl::Software)
                        .open();
                match poss_port {
                    Err(_) => {thread::sleep(Duration::from_millis(10))},
                    Ok(port) => {
                        *ylab_state = State::Connected {version: version, port: port}
                        }
                    }
                },
            State::Connected {version, port} 
                => {
                let std_start_time: Instant = Instant::now();
                let reader: BufReader<_> = BufReader::new(port);
                *ylab_state = State::Reading {version: version, start_time: std_start_time}
            },
            State::Reading {version, start_time, reader} 
                => {let mut got_first_line: bool = false;} 
                //println!("Sending");
                //thread::sleep(Duration::from_millis(100));
            },
        }
        //let serial_port = &serial_port.lock().unwrap().to_owned();
        // Look for serial ports
        }
        let baud_rate = ylab.lock().unwrap().baud();
        
        match port {
            Err(_e) => {
                //eprintln!("Failed to open {}", _e);
                // ::std::process::exit(1);
            },
            Ok(port) => {
                //let std_start_time = Instant::now();
                //let mut lab_start_time = Duration::ZERO;
                *connected.lock().unwrap() = true;
                
                // Reading serial input by line
                for line in reader.lines() {
                    if line.is_err() {continue;};
                    let line = line.unwrap();
                    // try parsing a sample from line
                    let possible_sample 
                        = Sample::from_csv_line(&line);
                    if possible_sample.is_err() {eprintln!("."); continue;}
                    // collect sample
                    let mut sample = possible_sample.unwrap();
                    // check if this is the first line
                    if !got_first_line {
                        //lab_start_time = Duration::from_micros(sample.time as u64);
                        //println!("{}", lab_start_time.as_micros());
                        got_first_line = true;
                    }

                    let run_time = Instant::now() - std_start_time;
                    sample.time = run_time.as_millis() as i64;
                    println!("{}", sample.to_csv_line());
                        
                    for chn in 0..8 {
                        let chan_id  = yld::CHAN_IDS[chn];
                        let mut sensory =
                            measurements
                            .lock()
                            .unwrap();
                        let possible_window 
                            = sensory.get_mut(chan_id);
                        match possible_window {
                            Some(window) => 
                                {window.add(measurements::Measurement
                                    ::new(sample.time as f64,
                                          sample.to_unit()[chn]));
                                 },
                            None => 
                                {eprintln!("No window {}", chn);}
                            };
                        }

                    serial_data
                    .lock()
                    .unwrap()
                    .push(line.to_string().trim().to_owned());

                    }
                    //thread::sleep(Duration::from_millis(10));
                }
            }
                            
        }

 