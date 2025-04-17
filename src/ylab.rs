/// YLab Connection
///
/// provides structures and methods to connect
/// and read from YLab devices
pub use std::fmt;
#[allow(unused_imports)]
pub use std::path::PathBuf;
pub use std::sync::{Arc, Mutex};
pub use std::time::Instant;

pub const _YLAB_EPOCH: usize = 1704063600;

/// YLab version

#[derive(PartialEq, Debug, Clone)]
pub struct YLab {
    board: YLabVersion,
    sensories: Vec<Sensory>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Sensory {
    Moi(u8),
    Adc(u8, usize),
    Yxz(u8, usize),
    Air(u8),
}

impl fmt::Display for Sensory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Sensory {
    pub fn label(self) -> String {
        let out = format!("{}", self);
        return out;
    }

    pub fn fft_size(self) -> Option<usize> {
        match self {
            Sensory::Adc(_, hz) | Sensory::Yxz(_, hz) => match hz {
                0..=10 => None,
                11..=32 => Some(32 * 2),
                33..=64 => Some(64 * 2),
                65..=128 => Some(128 * 2),
                129..=256 => Some(256 * 2),
                257..=512 => Some(512 * 2),
                _ => Some(1024 * 2),
            },
            _ => None,
        }
    }

    pub fn fft_low(self) -> Option<f32> {
        match self {
            Sensory::Adc(_, hz) | Sensory::Yxz(_, hz) => {
                Some(self.fft_size().unwrap() as f32 / hz as f32)
            }
            _ => None,
        }
    }

    pub fn fft_high(self) -> Option<f32> {
        match self {
            Sensory::Adc(_, hz) | Sensory::Yxz(_, hz) => Some(hz as f32 / 2.0),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum YLabVersion {
    Zet,
    Pro,
    Go,
    GoMotion(u8),
    GoStress,
    Mini,
}

impl YLabVersion {
    pub fn baud(&self) -> u32 {
        match *self {
            YLabVersion::Pro => 2_000_000,
            YLabVersion::Zet => 2_000_000,
            YLabVersion::Go => 1_000_000,
            YLabVersion::GoMotion(_) => 1_000_000,
            YLabVersion::GoStress => 1_000_000,
            YLabVersion::Mini => 125_200,
        }
    }

    pub fn fft_size(&self) -> usize {
        match *self {
            YLabVersion::Pro => 512,
            YLabVersion::Zet => 256,
            YLabVersion::Go => 256,
            YLabVersion::GoMotion(_) => 128,
            YLabVersion::GoStress => 256,
            YLabVersion::Mini => 128,
        }
    }

    pub fn history_size(&self) -> usize {
        match *self {
            YLabVersion::Pro => 512,
            YLabVersion::Zet => 256,
            YLabVersion::Go => 256,
            YLabVersion::GoMotion(_) => 128,
            YLabVersion::GoStress => 256,
            YLabVersion::Mini => 128,
        }
    }

    pub fn bank_labels(&self) -> Vec<&str> {
        match *self {
            YLabVersion::Pro => vec!["MOI", "ADC"],
            YLabVersion::Zet => vec!["MOI", "ADC1", "ADC2", "Mo1", "Mo2"],
            YLabVersion::Go => vec!["MOI", "ADC"],
            YLabVersion::GoMotion(1) => vec!["MOI", "Analog", "Yxz"],
            YLabVersion::GoMotion(4) => vec!["MOI", "Analog", "Yxz_0", "Yxz_1", "Yxz_2", "Yxz_3"],
            YLabVersion::GoMotion(7) => vec![
                "MOI", "ADC", "Yxz_0", "Yxz_1", "Yxz_2", "Yxz_3", "Yxz_4", "Yxz_5", "Yxz_6",
                "Yxz_7",
            ],
            YLabVersion::GoMotion(_) => todo!(),
            YLabVersion::GoStress => vec!["MOI", "ADC", "Air"],
            YLabVersion::Mini => vec!["ADC"],
        }
    }

    pub fn n_banks(&self) -> u8 {
        self.bank_labels().len().clone() as u8
    }
}

impl fmt::Display for YLabVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YLabVersion::Pro => write!(f, "Pro"),
            YLabVersion::Zet => write!(f, "Zet"),
            YLabVersion::Go => write!(f, "Go"),
            YLabVersion::GoMotion(n) => write!(f, "Go Motion {}", n),
            YLabVersion::GoStress => write!(f, "Go Stress"),
            YLabVersion::Mini => write!(f, "Mini"),
        }
    }
}

/// YLab States and Commands
///
/// provides the states and control commands of YLab devices
/// YLab states are organized hierarchically to make it
/// easier to pass on objects.
use serialport::{self, UsbPortInfo};
use std::io::{BufRead, BufReader};
pub type LockedSerial = Arc<Mutex<Option<Box<dyn serialport::SerialPort + 'static>>>>;
pub type LockedBufReader = Arc<Mutex<Option<BufReader<Box<dyn serialport::SerialPort + 'static>>>>>;

/// YLab state
/// + Optional list of serial port names
pub type AvailablePorts = Option<Vec<String>>;
/// + set of states for the YLab reader
#[derive(PartialEq, Debug, Clone)]
pub enum YLabState {
    Disconnected {
        ports: AvailablePorts,
    },
    Connected {
        version: YLabVersion,
        port_name: String,
    },
    Reading {
        version: YLabVersion,
        port_name: String,
    },
}
/// + set of commands to control the YLab
#[derive(PartialEq, Debug, Clone)]
pub enum YLabCmd {
    Disconnect,
    Connect {
        version: YLabVersion,
        port_name: String,
    },
    Read {},
    Stop {},
}

use egui::emath::History;
/// YLab thread
use std::sync::*;
use std::thread;
#[allow(unused_imports)]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Task for reading data from serial port
///
/// ylab_state is used for state transitions
/// yld_wind is used for storing data
/// ylab_listen is used for listening to commands
///

pub fn ylab_thread(
    ylab_state: Arc<Mutex<YLabState>>,        // shared state
    ylab_listen: mpsc::Receiver<YLabCmd>,     // receiving comands
    yld_wind: Arc<Mutex<History<data::Yld>>>, // Yld history shared with UI (FFT) and storage
    ytf_wind: Arc<Mutex<data::Banks>>,        // Ytf8 history to share with UI (plot)
    yld_st: mpsc::Sender<data::Yld>,          // sending data to storage
) -> ! {
    // Preparing serial port and buffer
    let serialport: LockedSerial = Arc::new(Mutex::new(None));
    let bufreader: LockedBufReader = Arc::new(Mutex::new(None));

    // Unique time stamp and offset
    let start_time = Instant::now();

    loop {
        // capture YLab state and incoming commands from the UI
        let this_ylab_state = ylab_state.lock().unwrap().clone();
        let this_cmd = ylab_listen.try_recv().ok();
        // match the current state and do the transitions

        // state changes on command
        // beautiful!
        match (this_ylab_state, this_cmd){
            // Waiting for available ports and command
            (YLabState::Disconnected { ports: _ },
             None)
            => {
                let avail_ports = serialport::available_ports().ok();
                // incorrect! problem: There is often a keyboard on serial, so it is never empty.
                match avail_ports {
                    None => {
                        thread::sleep(Duration::from_millis(100));
                        // no ports: try again in 100ms, no transition
                    },
                    Some(found) => {
                        // ports found: transition to Disconnected with available ports
                        let port_names= found.iter()
                                .map(|p| p.clone().port_name)
                                .filter(|p| p.starts_with("/dev/ttyACM"))
                                .collect::<Vec<String>>();
                        // automatically proceed to Disconnected with available ports
                        *ylab_state.lock().unwrap() = YLabState::Disconnected{
                                                        ports: Some(port_names)};
                    },
                }
                },

            (YLabState::Disconnected { ports: Some(_) },
             Some(YLabCmd::Connect { version, port_name }))
            => {
                // We make one connection attempt to verify the port
                // later we can add code for sending commands to the
                // YLab, e.g. which sensors of a bank to collect.
                // If Rust holds its promise
                // the serial port is properly closed when going out of scope
                let poss_port =
                    serialport::new(port_name.clone(),
                        version.baud() as u32)
                        .timeout(Duration::from_millis(1))
                        .flow_control(serialport::FlowControl::Software)
                        .open(); // ok() turns a Result into an Option
                match poss_port {
                    Err(_) => {eprintln!("connection failed"); thread::sleep(Duration::from_millis(500))},
                    Ok(real_port)
                        => {*serialport.lock().unwrap() = Some(real_port);

                            // transition to Connected
                            *ylab_state.lock().unwrap() = YLabState::Connected {
                                                            version: version,
                                                            port_name: port_name.clone()};
                            println!("Connected to {}", port_name.clone());},

                    };
                },

            // Start reading on command
            (YLabState::Connected {version, ref port_name}, _)  // jumping over connect state
            //Some(YLabCmd::Read {}))
            => {*bufreader.lock().unwrap()
                    = Some(BufReader::new(serialport.lock().unwrap().take().unwrap()));
                *ylab_state.lock().unwrap() = YLabState::Reading {version: version.clone(),
                    port_name: port_name.clone()};
                },

            (YLabState::Reading {version, port_name:_, },
            None)
                // We are already in a fast loop, so we read one line at a time.
                =>  {let mut reader = bufreader.lock().unwrap();
                    match reader.as_mut().unwrap().lines().next(){
                        // buffer empty
                        None => {eprint!("No line"); continue},
                        // line found
                        Some(line)
                            => match line {
                                // conversion error
                                Err(_) => {continue},
                                // conversion success
                                Ok(line) => {
                                    // parse line into Ytf8
                                    match data::Ytf8::from_csv_line(&line) {
                                        // not a Ytf8 line
                                        Err(e) => {eprintln!("Not Ytf8: {:?}", e); continue}
                                        // Ytf8 line,
                                        Ok(sample) => {
                                            let ystudio_time = Instant::now().duration_since(start_time);
                                            let bank = sample.sensory;
                                            if (bank as usize) < version.bank_labels().len() {
                                                ytf_wind.lock().unwrap()[bank as usize]
                                                        .add(ystudio_time.as_secs_f64(), sample.clone());
                                            //ytf_out.send(sample).unwrap();
}
                                            let yld = sample.to_yld(ystudio_time);
                                            for measure in yld.iter() {
                                                yld_wind.lock().unwrap()
                                                        .add(ystudio_time.as_secs_f64(), measure.clone());
                                                yld_st.send(measure.clone()).unwrap();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                    },




            (YLabState::Reading {version, port_name},
            Some(YLabCmd::Disconnect {  }))
            => {*ylab_state.lock().unwrap() = YLabState::Connected{version, port_name};//YLabState::Disconnected{ports: None};
                let this_serial = bufreader.lock().unwrap().take().unwrap().into_inner();
                *serialport.lock().unwrap() = Some(this_serial); // It has been taken, so we put it back
                *bufreader.lock().unwrap() = None;
                println!("Stopped reading");
                *ylab_state.lock().unwrap() = YLabState::Disconnected { ports: None };
                *bufreader.lock().unwrap() = None;
                *serialport.lock().unwrap() = None;
                println!("Disconnected");
                },

            // Disconnect on command
            (YLabState::Connected{version:_, port_name:_},
                Some(YLabCmd::Disconnect{}))
                => {
                    *ylab_state.lock().unwrap() = YLabState::Disconnected { ports: None };
                    *bufreader.lock().unwrap() = None;
                    *serialport.lock().unwrap() = None;
                    println!("Disconnected");
                },
            (_,_)   => {},
        }
    }
}

/// YLab DATA

pub mod data {
    use super::*;
    use egui::util::History;
    use std::time::Duration;
    /// YLab Long Data
    ///
    /// YLD keeps data one row per measure with
    /// + a time stamp,
    /// + a device identifier
    /// + a sensory index (position in the bank)
    /// + one measurement value
    ///
    #[derive(Copy, Clone, Debug)]
    pub struct Yld {
        pub time: Duration,
        pub dev: u8,
        pub sensory: u8,
        pub chan: u8,
        pub value: f64,
    }

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
            return out;
        }
    }

    pub type _YldBuf = Vec<Yld>;
    /// Multi lines are a fixed array of vectors of xy points.
    pub type MultiLines<const N: usize> = [Vec<[f64; 2]>; 8];

    pub fn new_multi_lines() -> MultiLines<8> {
        [
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]
    }

    /// Splitting a Yld History into a vector of point series
    /// This is used to plot a bunch of lines in egui_plot
    pub trait SplitByChan {
        fn split(&self) -> MultiLines<8>;
    }

    /*impl SplitByChan for History<Yld>{
        fn split(&self) -> MultiLines<8> {
            let mut out = new_multi_lines();
            for measure in self.iter() {
                let time = measure.0;
                let chan = measure.1.chan as usize;
                let value = measure.1.value;
                let point = [time, value];
                out[chan].push(point);
            }
            return out;
        }
    }*/

    impl SplitByChan for History<Ytf8> {
        fn split(&self) -> MultiLines<8> {
            let mut out = new_multi_lines();
            for sample in self.iter() {
                for (chan, value) in sample.1.read.iter().enumerate() {
                    let point = [sample.0, *value];
                    out[chan].push(point);
                }
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
    pub struct Ytf<const N: usize, T> {
        pub dev: u8,
        pub sensory: u8,
        pub time: Duration,
        pub read: [T; N],
    }

    /// YTF 8 implementation
    pub type Ytf8 = Ytf<8, f64>;
    pub type Banks = Vec<History<Ytf8>>;

    /// Create history per bank
    ///
    /// returns a vector with n histories of Ytf8. Is used to
    /// separate multiple sensor arrays, e.g. by ADC controller or if you
    /// use multi-variate sensors, such as multiple Yxz.

    pub fn make_banks(n: u8, seconds: f32, max_len: usize) -> Banks {
        let mut out = Vec::<History<Ytf8>>::new();
        for _ in 0..n {
            let new_bank = History::<Ytf8>::new(1..max_len, seconds);
            out.push(new_bank);
            eprintln!("Bank {}", out.len())
        }
        out
    }

    /*#[derive(Copy, Clone, Debug)]
    pub struct Ytf8 {
        pub dev: u8,
        pub time: i64,
        pub read: [f64;8],
    }*/

    /// Error types for parsing CSV lines
    #[derive(Clone, Debug)]
    pub enum ParseError {
        Len(usize),
        Dev(String),
        Sensory(String),
        Time(String),
    }

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
            if cols.len() != 10 {
                return Err(ParseError::Len(cols.len()));
            };
            // extract time stamp
            //let millis = cols[0].parse::<i64>();
            let time: Duration;
            match cols[0].trim().parse::<i64>() {
                Ok(millis) => time = Duration::from_millis(millis.try_into().unwrap()),
                Err(_) => return Err(ParseError::Time(cols[0].to_string())),
            }

            // extract dev number
            //let dev = cols[1].trim().parse::<u8>();
            //if dev.is_err() {return Err(ParseError::Dev(cols[1].to_string()))}
            // extract sensory number
            let dev: u8 = 1; // only one YLab per Ystudio, at the moment.
            let sensory = cols[1].trim().parse::<u8>();
            if sensory.is_err() {
                return Err(ParseError::Sensory(cols[1].to_string()));
            }
            // reading the remaining 8 cols
            let mut read: [f64; 8] = [0., 0., 0., 0., 0., 0., 0., 0.];
            for chn in 0..8 {
                // parse value
                let value = cols[chn + 2].parse::<f64>();
                match value {
                    Ok(v) => read[chn] = v,
                    Err(_) => read[chn] = 0.0,
                }
            }
            Ok(Ytf8 {
                dev: dev,
                sensory: sensory.unwrap(),
                time: time,
                read: read,
            })
        }

        #[allow(dead_code)]
        pub fn to_csv_line(&self) -> String {
            let out: String = [
                [self.time.as_millis().to_string(), self.dev.to_string()].join(","),
                self.read.map(|r| r.to_string()).join(","),
            ]
            .join(",");
            //out.push_str("\r\n");
            return out;
        }

        pub fn to_yld(&self, time: Duration) -> Vec<Yld> {
            let mut out: Vec<Yld> = Vec::new();
            let mut chan: u8 = 0;
            for value in self.read.iter() {
                out.push(Yld {
                    time,
                    dev: self.dev,
                    sensory: self.sensory,
                    chan: chan,
                    value: *value as f64,
                });
                chan += 1;
            }
            return out;
        }

        #[allow(dead_code)]
        pub fn to_unit(mut self) -> Self {
            const MAX: f64 = 32_768.0;
            self.read = self.read.map(|r| r / MAX);
            self
        }
    }

    impl Default for Ytf8 {
        fn default() -> Self {
            Self {
                dev: 0,
                sensory: 0,
                time: Duration::from_millis(0),
                read: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            }
        }
    }
}
