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
pub enum YLabState {
    Disconnected {ports: AvailablePorts},
    Connected {start_time: Instant, version: YLabVersion, port: String},
    Reading {start_time: Instant, version: YLabVersion, port: String},
    Recording {path: PathBuf}
}

#[derive(PartialEq, Debug, Clone)]
pub enum YLabCmd {
    Disconnect,
    Connect {version: YLabVersion, port: String},
    Read {},
    Record {file: PathBuf},
}

/// YLab DATA

pub mod ydata {
    pub use std::error::Error;
    use std::{time::Duration, collections::HashMap};

    use egui::util::History;
    /// YLab Long Data
    /// 
    /// YLD keeps data one row per measure with 
    /// + a time stamp, 
    /// + a device identifier 
    /// + a sensory index (position in the bank)
    /// a value

    #[derive(Copy, Clone, Debug)]
    pub struct Yld {
        pub time: Duration,
        pub dev: i8,
        pub sensory : i8,
        pub probe: i8,
        pub value: f64,}

    pub type _YldBuf = Vec<Yld>;
    pub use egui_plot::PlotPoints;
    pub type MultiLines = Vec<Vec<[f64; 2]>>;
    
    /// Splitting a Yld History into a vector of point series
    /// This is used to plot a bunch of lines in egui_plot
    pub trait ToMultiLines {
        fn split(&self) -> MultiLines;
    }

    impl ToMultiLines for History<Yld>{
        fn split (&self) -> MultiLines {
            let mut point_map: MultiLines = Vec::new();
            for measure in self.iter() {
                let time = measure.0;
                let _dev = measure.1.dev as usize;
                let _sensory = measure.1.sensory as usize;
                let probe = measure.1.probe as usize;
                let value = measure.1.value;
                let point = [time, value];
                let point_id = probe as usize;
                match point_map.get_mut(point_id) {
                    Some(points) => points.push(point),
                    None => {point_map.insert(probe, vec!(point));},
                }           
            }
            return point_map;
        }
    }


    /// YLab transport format
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
    
    impl ToMultiLines for Ytf8 {
        fn split (&self) -> MultiLines {
            let mut point_map: MultiLines = Vec::new();
            let time = self.time as f64;
            let dev = self.dev as usize;
            let read = self.read;
            for probe in 0..8 {
                let value = read[probe];
                let point = [time, value];
                //let point_id = probe as usize;
                match point_map.get_mut(probe) {
                    Some(points) => points.push(point),
                    None => {point_map.insert(probe, vec!(point));},
                }           
            }
            return point_map;
        }
    }
    
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
            let mut probe: i8 = 0;
            for value in self.read.iter() {
                out.push(Yld{time, dev: 0, sensory: 0, probe: self.dev, value: *value as f64});
                probe += 1;
            };
            return out
        }
        pub fn to_unit(mut self) -> Self {
            const MAX: f64 = 32_768.0;
            self.read = self.read.map(|r| r / MAX);
            self
        }

/*        pub fn to_unit(&mut self) -> (){
            const MAX: f64 = 32_768.0;
            let read_unit = self.read.map(|r| {r/MAX});
            self.read = read_unit;
        }*/
    }

    
    impl Default for Ytf8 {
        fn default() -> Self {
            Self {dev: 0, time: 0, read: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}
        }
    }
}
