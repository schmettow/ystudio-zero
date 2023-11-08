/// YLab Connection
/// 
/// provides structures and methods to connect 
/// and read from YLab devices

pub use std::fmt;
pub use std::time::Instant;
pub use std::path::PathBuf;

//use egui::epaint::tessellator::Path;
//use serialport::SerialPort;

/// YLab version

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum YLabVersion {Pro, Go, Mini}

impl YLabVersion {
    pub fn baud(&self) -> i32 {
        match *self {
            YLabVersion::Pro => 2_000_000,
            YLabVersion::Go => 1_000_000,
            YLabVersion::Mini => 125_200,
        }
    }
}


impl fmt::Display for YLabVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YLabVersion::Pro => write!(f, "Pro"),
            YLabVersion::Go => write!(f, "Go"),
            YLabVersion::Mini => write!(f, "Mini"),
        }
    }
}

/// Optional list of serial port names
pub type AvailablePorts = Option<Vec<String>>;

/// YLab State
/// 
/// provides the states of YLab devices
/// Connect and Read actually are for sending commands to the YLab. 
/// That's the flexibility of a mutex, it is bidirectional.
/// A cleaner way would be to use separate types for 
/// commands and states, use signals for commands and channels for data.
/// Similar to how it is done in YLab-Edge.

#[derive(PartialEq, Debug, Clone)]
pub enum Recording {
    Raw {start_time: Instant, file: PathBuf},
    Yld {start_time: Instant, file: PathBuf},
    Paused {start_time: Instant, file: PathBuf},
}

#[derive(PartialEq, Debug, Clone)]
pub enum YLabState {
    Disconnected {ports: AvailablePorts},
    Connected {start_time: Instant, version: YLabVersion, port: String},
    Reading {start_time: Instant, version: YLabVersion, port: String, recording: Option<Recording> },
}

#[derive(PartialEq, Debug, Clone)]
pub enum YLabCmd {
    Disconnect,
    Connect {version: YLabVersion, port: String},
    Read {},
    Pause {},
    Record {file: PathBuf},
}


/// YLab thread

use serialport;
use std::io::{BufReader,BufRead};
use std::sync::*;
use std::thread;
use std::time::Duration;
use egui::emath::History;

/// Task for reading data from serial port
/// 
/// ylab_state is used for state transitions
/// yld_wind is used for storing data
/// ylab_listen is used for listening to commands
/// 
pub fn ylab_thread(
    ylab_state: Arc<Mutex<YLabState>>,
    ylab_listen: mpsc::Receiver<YLabCmd>,
    yld_wind: Arc<Mutex<History<data::Yld>>>,
    yldest: mpsc::Sender<data::Yld>,
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
                            = data::Ytf8::from_csv_line(&line);
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
                        //let hist_len = yld_wind.lock().unwrap().len();
                        println!("{}", sample.to_csv_line());
                        //println!("{}: {} | {}", ystudio_time, sample.read[0], hist_len);
                        for measure in sample.to_yld(Duration::from_millis(ystudio_time as u64)).iter() {
                            yld_wind.lock().unwrap().add(ystudio_time, *measure);
                            yldest.send(*measure).unwrap();
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


/// YLab DATA

pub mod data {
    pub use std::error::Error;
    use std::{time::Duration};

    use egui::util::History;
    /// YLab Long Data
    /// 
    /// YLD keeps data one row per measure with 
    /// + a time stamp, 
    /// + a device identifier 
    /// + a sensory index (position in the bank)
    /// a value

    pub const YLD_HEAD: &str = "time,dev,sensory,chan,value\r\n";

    #[derive(Copy, Clone, Debug)]
    pub struct Yld {
        pub time: Duration,
        pub dev: i8,
        pub sensory : i8,
        pub chan: i8,
        pub value: f64,}
    
    impl Yld {
        pub fn to_csv_line(&self) -> String {
            let mut out = String::new();
            out.push_str(&self.time.as_secs_f64().to_string());
            out.push_str(",");
            out.push_str(&self.dev.to_string());
            out.push_str(",");
            out.push_str(&self.sensory.to_string());
            out.push_str(",");
            out.push_str(&self.chan.to_string());
            out.push_str(",");
            out.push_str(&self.value.to_string());
            out.push_str("\r\n");
            return out
        }

    }

    pub type _YldBuf = Vec<Yld>;
    pub use egui_plot::PlotPoints;
    //pub type MultiLines<const N: usize> = [Vec<[f64; 2]>; N];
    /// Multi lines are a fixed array of vectors of points
    pub type YldHistory = History<Yld>;
    pub type MultiLines<const N: usize> = [Vec<[f64; 2]>; 8];

    pub fn new_multi_lines(hist: &YldHistory) -> MultiLines<8> {
        [vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!()]
    }

    /// Splitting a Yld History into a vector of point series
    /// This is used to plot a bunch of lines in egui_plot
    pub trait SplitByChan {
        fn split(&self) -> MultiLines<8>;
    }

    impl SplitByChan for History<Yld>{
        fn split(&self) -> MultiLines<8> {
            let mut out = new_multi_lines(&self);
            for measure in self.iter() {
                let time = measure.0;
                let chan = measure.1.chan as usize;
                let value = measure.1.value;
                let point = [time, value];
                out[chan].push(point);
            }
            return out;
        }
    }

    /// YLab transport format (YTF)
    /// 
    /// YLabs send data with a time stamp,
    /// a device identifier and a vector of eight readings.
    ///
    /// 
    
    #[derive(Copy, Clone, Debug)]
    pub struct Ytf<const N: usize, T>  {
        pub dev: i8,
        pub time: Duration,
        pub read: [T;N],
    }

    /// YTF 8 implementation


    #[derive(Copy, Clone, Debug)]
    pub struct Ytf8 {
        pub dev: i8,
        pub time: i64,
        pub read: [f64;8],
    }
    
    /// Error types for parsing CSV lines
    #[derive(Clone, Debug,)]
    pub enum ParseError {
        Len(usize), 
        Dev(String), 
        Time(String)}
    
    /// Result type for parsing CSV lines
    pub type FailableSample = Result<Ytf8, ParseError>;
    

    /// Methods for YLab data samples
    impl Ytf8 {
        /// Create a new sample from a CSV line as String
        /// 
        /// The CSV line is expected to have 10 columns:
        /// time, device number, 8 readings.
        ///
        /// The time stamp is expected to be an integer, better would be Duration.
        /// Not Instant, because you cannot create Instant from numbers. 
        /// Also, the timestamp sent by the YLab is usually since startup of the YLab.
        /// Duration *since startup* is slightly stronger than Duration. 
        /// It's like every run has their own epoch.
        /// 
        /// Collecting a sample fro a CSV line is a fallible operation.
        pub fn from_csv_line(line: &String) -> FailableSample {
            // splitting
            let cols: Vec<&str> = line.split(",").collect();
            // check correct length
            if cols.len() != 10 {return Err(ParseError::Len(cols.len()))};
            // extract time stamp
            let time = cols[0].parse::<i64>();
            if time.is_err() {return Err(ParseError::Time(cols[0].to_string()))}
            // extract dev number
            let dev = cols[1].parse::<i8>();
            if dev.is_err() {return Err(ParseError::Dev(cols[1].to_string()))}
            // reading the remaining 8 cols
            let mut read: [f64; 8] = [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0];
            for chn in 0..8 {
                // parse value
                let value 
                    = cols[chn + 2].parse::<f64>();
                match value {
                    Ok(v) => read[chn] = v,
                    Err(_) => read[chn] = 0.0
                }
            }
            Ok(Ytf8{dev: dev.unwrap(), 
                        time: time.unwrap() as i64, 
                        read: read})
        }


        pub fn to_csv_line(&self) -> String {
            let out: String = 
                [[self.time.to_string(), self.dev.to_string()]
                    .join(","),
                  self.read
                    .map(|r|{r.to_string()})
                    .join(",")]
                .join(",");
            //out.push_str("\r\n");
            return out
        }

        pub fn to_yld(&self, time: Duration) -> Vec<Yld> {
            let mut out: Vec<Yld> = Vec::new();
            let mut chan: i8 = 0;
            for value in self.read.iter() {
                out.push(Yld{time, dev: 0, sensory: 0, chan: chan, value: *value as f64});
                chan += 1;
            };
            return out
        }
        pub fn to_unit(mut self) -> Self {
            const MAX: f64 = 32_768.0;
            self.read = self.read.map(|r| r / MAX);
            self
        }

    }

    
    impl Default for Ytf8 {
        fn default() -> Self {
            Self {dev: 0, time: 0, read: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}
        }
    }
}
