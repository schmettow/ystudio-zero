use crate::measurements;
use crate::measurements::MeasurementWindow;
use crate::ylab::{PossPort, PossReader, AvailablePorts};
use crate::ylab::{YLabState as State, yld::{Sample, self}, YLabCmd};

use serialport;
use std::collections::HashMap;
use std::io::{BufReader,BufRead};
use std::sync::*;
use std::thread;
use std::time::{Duration, Instant, };
use egui::emath::History;


pub fn serial_thread(
    ylab: Arc<Mutex<State>>,
    ylab_data: Arc<Mutex<History<Sample>>>,
    ylab_listen: mpsc::Receiver<YLabCmd>,
    ) -> ! {
    
    let mut poss_port: PossPort;
    let mut poss_reader: PossReader;
    let mut reader: BufReader<Box<dyn serialport::SerialPort>>;
    
    let mut ylab_state = ylab.lock().unwrap();
    loop {
        let this_state = ylab_state.clone();
        let this_cmd = ylab_listen.try_recv();
        match this_state {
            State::Disconnected {ports: _}
                => {let ports = serialport::available_ports();
                    match ports {
                        Err(_) => { // trying again in 500ms
                            thread::sleep(Duration::from_millis(500));
                            *ylab_state = State::Disconnected{ports: AvailablePorts::None}},
                        Ok(found) => {
                            let port_names 
                                = found.iter().map(|p| p.port_name.clone())
                                    .collect::<Vec<String>>();
                            // State is updating itself by collecting available ports
                            *ylab_state = State::Disconnected{ports: Some(port_names)}};
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
                                            => {*ylab_state = State::Connected {start_time: Instant::now(),
                                                                                version: version, 
                                                                                port: port.clone()}}
                                    }
                                },
                            }
                        }
                    },
            State::Connected {  start_time, version, ref port} 
                => {match this_cmd {
                        _ => {},
                        Ok(YLabCmd::Read {  }) 
                            => {reader = BufReader::new(poss_port.unwrap());
                                *ylab_state = State::Reading {version: version, start_time: start_time, port: port.to_string()};},
            State::Reading {version, start_time, port:_} 
                => {let mut got_first_line: bool = false;
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
                                    {window.add(
                                        measurements::Measurement::new(sample.time as f64,
                                              sample.to_unit()[chn]));
                                     },
                                None => 
                                    {eprintln!("No window {}", chn);}
                                };
                            }    
                    }   
                },
        }
    }
}
    }}