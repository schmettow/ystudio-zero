/// YLab Data (YLD)
/// 
/// provides structures to hold YLab data streams 
/// and methods to convert from and into 
pub use std::fmt;
pub use std::time::Instant;

use serialport::SerialPort;

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

    pub fn _ticks_per_sec(&self) -> usize {
        match *self {
            YLabVersion::Pro => 1_000_000,
            YLabVersion::Go => 1_000_000,
            YLabVersion::Mini => 100,
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
/// Possible serial port
pub type PossPort = Option<Box<dyn SerialPort>>;
/// Possible serial port
pub type PossReader = Option<Box<dyn std::io::BufRead>>;


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
    Connected {start_time: Instant, version: YLabVersion, port: String},//port: Box<dyn serialport::SerialPort>},
    Reading {start_time: Instant, version: YLabVersion, port: String},  //reader: BufReader<Box<dyn serialport::SerialPort>>},
}

pub enum YLabCmd {
    Disconnect,
    Connect {version: YLabVersion, port: String},//port: Box<dyn serialport::SerialPort>},
    Read {},  //reader: BufReader<Box<dyn serialport::SerialPort>>},
}

/// YLab data stream (YLD)

pub mod yld {
    /// Sample of YLab data
    /// 
    /// YLabs keeps data with a time stamp, 
    /// a device identifier and a vector of eight readings. 
    /// The time stamp is relative to the startup of the Ylab device.
    pub use std::error::Error;
    pub static CHAN_IDS: [&str; 8] = [ "y0", "y1", "y2", "y3",
    "y4", "y5", "y6", "y7"];

    /// Sample of YLab data
    /// 
    /// YLabs keeps data with a time stamp,
    /// a device identifier and a vector of eight readings.
    ///
    #[derive(Copy, Clone, Debug)]
    pub struct Sample {
        pub dev: i8,
        pub time: i64,
        pub read: [u16;8],
    }
    
    /// Error types for parsing CSV lines
    #[derive(Debug)]
    pub enum ParseError {
        Len(usize), 
        Dev(String), 
        Time(String)}
    
    /// Result type for parsing CSV lines
    pub type FailableSample = Result<Sample, ParseError>;
    
    /// Methods for YLab data samples
    impl Sample {
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
        /// 
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
            let mut read: [u16; 8] = [0,0,0,0,0,0,0,0];
            for chn in 0..8 {
                // parse value
                let value 
                    = cols[chn + 2].parse::<u16>();
                match value {
                    Ok(v) => read[chn] = v,
                    Err(_) => read[chn] = 0
                }
            }
            Ok(Sample{dev: dev.unwrap(), 
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

        pub fn to_unit(&self) -> [f64;8]{
            self.read.map(|r| {r as f64/ (2^15) as f64 })
        }
    }

    
    impl Default for Sample {
        fn default() -> Self {
            Self {dev: 0, time: 0, read: [0, 0, 0, 0, 0, 0, 0, 0]}
        }
    }
}
