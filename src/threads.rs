//use crate::ylab::PossPort;

pub mod yld_store {
    use crate::ylab::*;
    use crate::ylab::ydata::*;
    use std::io::Write;
    use std::sync::*;
    use std::thread;
    use std::fs;
    use std::io::BufWriter;

    #[derive(Clone)]
    pub enum YldStoreState {
        Idle,
        Recording {path: PathBuf},
        Pausing {path: PathBuf}}

    /*impl Clone for State {
        fn clone(&self) -> State {
            match self {
                State::Idle => State::Idle,
                State::Recording {file} => State::Recording {file: file.to_owned()},
                State::Pausing {file} => State::Pausing {file: file.clone()},
            }
        }
    }*/

    #[derive(Clone)]
    pub enum Cmd {
        New {path: PathBuf},
        Pause,
        Resume,
        Stop,}

    pub fn yld_store_thread(
        state: Arc<Mutex<YldStoreState>>,
        incoming: mpsc::Receiver<Yld>,
        listen: mpsc::Receiver<Cmd>,
    ) -> ! {
        loop {
            // getting the current state, incoming commands and incoming data
            let this_state = state.lock().unwrap().clone();
            let this_cmd = listen.try_recv().ok();
            let measure = incoming.try_recv().ok();
            // match the current state and do the transitions
            match (this_state, this_cmd, measure) {
                // start recording
                (YldStoreState::Idle, Some(Cmd::New { path }), _) 
                    => {
                    let mut file = fs::File::create(path.clone());
                    match file {
                        Ok(mut file) => {
                            file.write_all(ydata::YLD_HEAD.as_bytes()).unwrap();
                            *state.lock().unwrap() = YldStoreState::Recording {path: path};},
                        Err(_) => {
                            eprintln!("Could not create file");
                        },
                    }
                },
                // Do the recording
                (YldStoreState::Recording{path}, _, Some(measure))
                    => {
                        let file = fs::File::open(&path);
                        match file {
                            Ok(mut file) => {
                                file.write_all(measure.to_csv_line().as_bytes()).unwrap();},
                            Err(_) => {
                                eprintln!("Could not open file");
                            },
                        }
                    },
                // pause recording
                (YldStoreState::Recording{path}, Some(Cmd::Pause), _) 
                    => {
                        *state.lock().unwrap() = YldStoreState::Pausing{path};
                    },

                // resume recording after pause
                (YldStoreState::Pausing{path}, Some(Cmd::Resume), _) 
                    => {
                        let file = fs::File::open(&path);
                        match file {
                            Ok(mut file) => {
                                file.write_all(ydata::YLD_HEAD.as_bytes()).unwrap();
                                *state.lock().unwrap() = YldStoreState::Recording {path: path};},
                            Err(_) => {
                                eprintln!("Could not open file. Please close all other programs that might be using it.");
                            },
                        }
                    },
                // stop recording
                (YldStoreState::Recording{path}, Some(Cmd::Stop), _) 
                    => {
                        *state.lock().unwrap() = YldStoreState::Idle;
                    },
                
                _ => {},
            }
        }
    }
}


pub mod ylab {

    use crate::ylab::*;
    use crate::ylab::ydata::*;
    use serialport;
    use std::io::{BufReader,BufRead};
    use std::sync::*;
    use std::thread;
    use std::time::{Duration, Instant, };
    use egui::emath::History;

    /// Task for reading data from serial port
    /// 
    /// ylab_state is used for state transitions
    /// yld_hist is used for storing data
    /// ylab_listen is used for listening to commands
    /// 
    pub fn ylab_thread(
        ylab_state: Arc<Mutex<YLabState>>,
        yld_hist: Arc<Mutex<History<Yld>>>,
        yld_store_send: mpsc::Sender<Yld>,
        ylab_listen: mpsc::Receiver<YLabCmd>,
        ) -> ! {
        
        loop {
            // capture YLab state and incoming commands from the UI
            let this_ylab_state = ylab_state.lock().unwrap().clone();
            let this_cmd = ylab_listen.try_recv().ok();
            // match the current state and do the transitions
            match this_ylab_state {
                YLabState::Connected {  start_time, version, ref port} 
                    => {match this_cmd {
                        Some(YLabCmd::Read {}) 
                            => {*ylab_state.lock().unwrap() = YLabState::Reading {version, start_time, port: port.to_string(), recording: None};},        
                        _   => {},}},
                YLabState::Reading {  start_time, version, ref port, recording} 
                    => {let mut got_first_line: bool = false;
                        // In the previous state we've checked that the port works, so we can unwrap
                        let port 
                            = serialport::new(port, version.baud() as u32)
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
                            if possible_sample.is_err() {
                                //eprintln!(".");
                                continue;}
                            // collect sample
                            let sample = possible_sample.unwrap().to_unit();
                            // check if this is the first line
                            if !got_first_line {
                                let _lab_start_time = Duration::from_micros(sample.time as u64);
                                // here we can dynamically infer the data format (YTF or YLD).
                                got_first_line = true
                            }
                            let ystudio_time = (Instant::now() - start_time).as_secs_f64();
                            //let hist_len = yld_hist.lock().unwrap().len();
                            println!("{}", sample.to_csv_line());
                            //println!("{}: {} | {}", ystudio_time, sample.read[0], hist_len);
                            for measure in sample.to_yld(Duration::from_millis(ystudio_time as u64)).iter() {
                                yld_hist.lock().unwrap().add(ystudio_time, *measure);
                                yld_store_send.send(*measure).unwrap();
                            }
                            
                            //println!("{}", sample.to_csv_line());
                        
                        }},

                YLabState::Disconnected {ports: _} 
                    // read list of port names from serial
                    => {let avail_ports = serialport::available_ports().ok();
                        
                        match avail_ports {
                            None => { 
                                // no ports: try again in 500ms, no transition
                                thread::sleep(Duration::from_millis(500));//},
                                //ylab_state = YLabState::Disconnected{ports: AvailablePorts::None}},
                                *ylab_state.lock().unwrap() = YLabState::Disconnected{ports: None};},
                            Some(found) => {
                                // ports found: transition to Disconnected with available ports
                                let port_names 
                                    = found.iter().map(|p| p.clone().port_name)
                                        .collect::<Vec<String>>();
                                // State is updating itself by collecting available ports
                                *ylab_state.lock().unwrap() = YLabState::Disconnected{ports: Some(port_names)};
                                match this_cmd {
                                    Some(YLabCmd::Connect { version, port })
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
                                                => {*ylab_state.lock().unwrap() = YLabState::Connected {start_time: Instant::now(),
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
}