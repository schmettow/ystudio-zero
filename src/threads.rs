use crate::ylab::PossPort;
use crate::ylab::{YLabState as State, yld::Sample, YLabCmd};

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
    ylab_data: Arc<Mutex<History<Sample>>>,
    ylab_listen: mpsc::Receiver<YLabCmd>,
    ) -> ! {
    

    let mut poss_port: PossPort;
    //let mut poss_reader: PossReader;
    let mut reader: BufReader<Box<dyn serialport::SerialPort>>;
    
    //let mut ylab_state = ylab.lock().unwrap();
    loop {
        // see if there is an incoming command from the UI
        let this_cmd = ylab_listen.try_recv();
        // match the current state, inside match statements are used for state transitions
        match *ylab_state.clone().lock().unwrap() {
            State::Connected {  start_time, version, ref port} 
                => {match this_cmd {
                    _ => {},
                    Ok(YLabCmd::Read {  }) 
                        => {reader = BufReader::new(poss_port.unwrap());
                            *ylab_state.lock().unwrap() 
                            = State::Reading {version, start_time, port: port.to_string()};},}},
            State::Reading {  start_time, version, ref port} => {let mut got_first_line: bool = false;
                // INSERT: Opening the global buffer reader
                for line in reader.lines() {
                    // ignore faulty lines
                    if line.is_err() {continue;};
                    // get the line
                    let line = line.unwrap();
                    // try parsing a sample from line
                    let possible_sample 
                        = Sample::from_csv_line(&line);
                    // print a dot when sample not valid
                    if possible_sample.is_err() {eprintln!("."); continue;}
                    // collect sample
                    let mut sample = possible_sample.unwrap();
                    // check if this is the first line
                    if !got_first_line {
                        //lab_start_time = Duration::from_micros(sample.time as u64);
                        //println!("{}", lab_start_time.as_micros());
                        got_first_line = true
                    }

                    let run_time = Instant::now() - start_time;
                    sample.time = run_time.as_millis() as i64;
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
                            // ports found: transition to Disconnected with ports
                            let port_names 
                                = found.iter().map(|p| p.port_name)
                                    .collect::<Vec<String>>();
                            // State is updating itself by collecting available ports
                            *ylab_state.lock().unwrap() 
                                = State::Disconnected{ports: Some(port_names)};
                            match this_cmd {
                                _ => {},
                                Ok(YLabCmd::Connect { version, port })
                                => {poss_port = 
                                    serialport::new(port.clone(),
                                        version.baud() as u32)
                                        .timeout(Duration::from_millis(1))
                                        .flow_control(serialport::FlowControl::Software)
                                        .open()
                                        .ok(); // ok() turns a Result into an Option
                                    match poss_port {
                                        None => {thread::sleep(Duration::from_millis(10))},
                                        Some(_) 
                                            => {*ylab_state.lock().unwrap() 
                                                = State::Connected {start_time: Instant::now(),
                                                                                version: version, 
                                                                                port: port.clone()}}
                                    };
                                },
                            };
                        }
                    }
                }
            };
        }
    }