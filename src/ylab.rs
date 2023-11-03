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

/// YLab DATA

pub mod ydata {
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

    /* impl std::io::Write for Yld {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut out = String::new();
            out.push_str(&self.time.as_secs().to_string());
            out.push_str(",");
            out.push_str(&self.dev.to_string());
            out.push_str(",");
            out.push_str(&self.sensory.to_string());
            out.push_str(",");
            out.push_str(&self.chan.to_string());
            out.push_str(",");
            out.push_str(&self.value.to_string());
            out.push_str("\r\n");
        };
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
*/
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
