//use crate::helpers;
use crate::measurements;
use crate::measurements::MeasurementWindow;
use crate::ylab::yld::{Sample, self};

use serialport;
use std::collections::HashMap;
use std::io::Write;
use std::io::{self};
use std::io::{BufReader, BufRead};
use std::sync::*;
use std::thread;
use std::time::{Duration};
use egui::emath::History;
const baud_rate: u32 = 2_000_000;


pub fn serial_thread(
    measurements: Arc<Mutex<HashMap<String, MeasurementWindow>>>,
    history: Arc<Mutex<History<Sample>>>,
    serial_port: Arc<Mutex<String>>,
    available_ports: Arc<Mutex<Vec<String>>>,
    _variables: Arc<Mutex<Vec<String>>>,
    serial_data: Arc<Mutex<Vec<String>>>,
    send_serial: Arc<Mutex<bool>>,
    serial_write: Arc<Mutex<String>>,) -> ! {
    
    // Connecting to serial port
    loop {
        let serial_port = &serial_port.lock().unwrap().to_owned();
        // Look for serial ports
        match serialport::available_ports() {
            Err(e) => println!("{}", e),
            Ok(n) => {
                available_ports.lock().unwrap().clear();
                for i in n {
                    available_ports.lock().unwrap().push(i.port_name);
                }}
        }
        let port 
            = serialport::new(serial_port, baud_rate)
                .timeout(Duration::from_millis(1))
                .flow_control(serialport::FlowControl::Software)
                .open();
        match port {
            Err(_e) => {
                //eprintln!("Failed to open {}", _e);
                // ::std::process::exit(1);
            },
            Ok(port) => {
                //let std_start_time = Instant::now();
                let mut lab_start_time = Duration::ZERO;
                let mut got_first_line = false;
                let reader = BufReader::new(port);
                
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
                        lab_start_time = Duration::from_micros(sample.time as u64);
                        //println!("{}", lab_start_time.as_micros());
                        got_first_line = true;
                    }

                    sample.time = (sample.time - lab_start_time.as_micros() as i64) / 1_000_000;
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
                                /* let check_value = window.values.back();
                                if check_value.is_some(){
                                    println!("{}:{}", check_value.unwrap().x, check_value.unwrap().y)
                                }*/
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
    }

 