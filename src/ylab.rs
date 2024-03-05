/// YLab Connection
/// 
/// provides structures and methods to connect 
/// and read from YLab devices

pub use std::fmt;
pub use std::time::Instant;

#[allow(unused_imports)]
pub use std::path::PathBuf;

pub const YLAB_EPOCH: usize = 1704063600;

/// YLab version

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum YLabVersion {Pro, Go, Mini}

impl YLabVersion {
    pub fn baud(&self) -> u32 {
        match *self {
            YLabVersion::Pro => 2_000_000,
            YLabVersion::Go => 1_000_000,
            YLabVersion::Mini => 125_200,
        }
    }

    pub fn fft_size(&self) -> usize {
        match *self {
            YLabVersion::Pro => 1024,
            YLabVersion::Go => 512,
            YLabVersion::Mini => 128,
        }
    }

    pub fn bank_labels(&self) -> Vec<&str> {
        match *self {
            YLabVersion::Pro => vec!["ADC", "I2C0", "I2C1", "I2C2"],
            YLabVersion::Go => vec!["ADC", "I2C0", "I2C1"],
            YLabVersion::Mini => vec!["ADC"],
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


/// YLab States and Commands
/// 
/// provides the states and control commands of YLab devices
/// YLab states are organized hierarchically to make it
/// easier to pass on objects.

use serialport;
use std::io::{BufReader,BufRead};
pub type LockedSerial = Arc<Mutex<Option<Box<dyn serialport::SerialPort + 'static>>>>;
pub type LockedBufReader = Arc<Mutex<Option<BufReader<Box<dyn serialport::SerialPort + 'static>>>>>;

/// YLab state
/// + Optional list of serial port names
pub type AvailablePorts = Option<Vec<String>>;
/// + set of states for the YLab reader
#[derive(PartialEq, Debug, Clone)]
pub enum YLabState {
    Disconnected {ports: AvailablePorts},
    Connected {version: YLabVersion, port_name: String},
    Reading {version: YLabVersion, port_name: String},
}
/// + set of commands to control the YLab
#[derive(PartialEq, Debug, Clone)]
pub enum YLabCmd {
    Disconnect,
    Connect {version: YLabVersion, port_name: String},
    Read {},
    Stop {}
}


/// YLab thread
use std::sync::*;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use egui::emath::History;


/// Task for reading data from serial port
/// 
/// ylab_state is used for state transitions
/// yld_wind is used for storing data
/// ylab_listen is used for listening to commands
/// 



pub fn ylab_thread(
    ylab_state: Arc<Mutex<YLabState>>, // shared state
    ylab_listen: mpsc::Receiver<YLabCmd>,  // receiving comands
    yld_wind: Arc<Mutex<History<data::Yld>>>, // Yld history shared with UI and storage
    ytf_wind: Arc<Mutex<Vec<History<data::Ytf8>>>>, // Ytf8 history to share with UI
    yld_st: mpsc::Sender<data::Yld>, // sending data to storage
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
                        // no ports: try again in 500ms, no transition
                    },
                    Some(found) => {
                        // ports found: transition to Disconnected with available ports
                        let port_names 
                            = found.iter().map(|p| p.clone().port_name)
                                .collect::<Vec<String>>();
                        // automatically proceed to Disconnected with available ports
                        *ylab_state.lock().unwrap() = YLabState::Disconnected{
                                                        ports: Some(port_names)};
                    },
                }
                thread::sleep(Duration::from_millis(100));
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
            (YLabState::Connected {version, ref port_name},
            Some(YLabCmd::Read {})) 
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
                        None => {continue},
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
                                        Err(_) => {continue}
                                        // Ytf8 line,
                                        Ok(sample) => {
                                            let ystudio_time = Instant::now().duration_since(start_time);
                                            let bank = sample.dev;
                                            if (bank as usize) < version.bank_labels().len() {
                                                ytf_wind.lock().unwrap()[bank as usize].add(ystudio_time.as_secs_f64(), sample.clone());
                                                //ytf_out.send(sample).unwrap();
                                                }
                                            let yld = sample.to_unit().to_yld(ystudio_time);
                                            for measure in yld.iter() {
                                                yld_wind.lock().unwrap().add(ystudio_time.as_secs_f64(), measure.clone());
                                                yld_st.send(measure.clone()).unwrap();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                    },
                
                        

                   
            (YLabState::Reading {version, port_name},
            Some(YLabCmd::Stop {  })) 
            => {*ylab_state.lock().unwrap() = YLabState::Connected{version, port_name};//YLabState::Disconnected{ports: None};
                let this_serial = bufreader.lock().unwrap().take().unwrap().into_inner();
                *serialport.lock().unwrap() = Some(this_serial); // It has been taken, so we put it back
                *bufreader.lock().unwrap() = None;
                println!("Stopped reading");
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
        }}}
      

/// YLab DATA

pub mod data {
    use std::time::Duration;
    use egui::util::History;
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
        pub sensory : u8,
        pub chan: u8,
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
    /// Multi lines are a fixed array of vectors of xy points.
    pub type MultiLines<const N: usize> = [Vec<[f64; 2]>; 8];

    pub fn new_multi_lines() -> MultiLines<8> {
        [vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!(), vec!()]
    }

    /// Splitting a Yld History into a vector of point series
    /// This is used to plot a bunch of lines in egui_plot
    pub trait SplitByChan {
        fn split(&self) -> MultiLines<8>;
    }

    impl SplitByChan for History<Yld>{
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
    }

    /// YLab transport format (YTF)
    /// 
    /// YLabs send data with a time stamp,
    /// a device identifier and a vector of eight readings.
    ///
    /// 
    
    #[derive(Copy, Clone, Debug)]
    pub struct Ytf<const N: usize, T>  {
        pub dev: u8,
        pub sensory: u8,
        pub time: Duration,
        pub read: [T;N],
    }

    /// YTF 8 implementation
    pub type Ytf8 = Ytf<8, f64>;

    /*#[derive(Copy, Clone, Debug)]
    pub struct Ytf8 {
        pub dev: u8,
        pub time: i64,
        pub read: [f64;8],
    }*/
    
    /// Error types for parsing CSV lines
    #[derive(Clone, Debug,)]
    pub enum ParseError {
        Len(usize), 
        Dev(String),
        Sensory(String), 
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
            //let millis = cols[0].parse::<i64>();
            let time: Duration;
            match cols[0].parse::<i64>() {
                Ok(millis) => {time = Duration::from_millis(millis.try_into().unwrap())},
                                Err(_) => return Err(ParseError::Time(cols[0].to_string())),
                }
            
            // extract dev number
            let dev = cols[1].parse::<u8>();
            if dev.is_err() {return Err(ParseError::Dev(cols[1].to_string()))}
            // extract sensory number
            let sensory = cols[1].parse::<u8>();
            if sensory.is_err() {return Err(ParseError::Sensory(cols[1].to_string()))}
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
            Ok(Ytf8{dev: dev.unwrap(), sensory: sensory.unwrap(), time: time, read: read})
        }

        #[allow(dead_code)]
        pub fn to_csv_line(&self) -> String {
            let out: String = 
                [[self.time.as_millis().to_string(), self.dev.to_string()]
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
            let mut chan: u8 = 0;
            for value in self.read.iter() {
                out.push(Yld{time, dev: self.dev, sensory:self.sensory, chan: chan, value: *value as f64});
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
            Self {dev: 0, sensory:0, time: Duration::from_millis(0), read: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}
        }
    }
}
