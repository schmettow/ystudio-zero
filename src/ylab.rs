/// YLab Data (YLD)
/// 
/// provides structures to hold YLab data streams 
/// and methods to convert from and into 

pub mod yld{
    /// Sample of YLab data
    /// 
    /// YLabs keeps data with a time stamp, 
    /// a device identifier and a vector of eight readings. 
    /// The time stamp is relative to the startup of the Ylab device.
    pub use std::error::Error;
    pub static CHAN_IDS: [&str; 8] = [ "y0", "y1", "y2", "y3",
    "y4", "y5", "y6", "y7"];

    #[derive(Copy, Clone, Debug)]
    pub struct Sample {
        pub dev: i8,
        pub time: i64,
        pub read: [u16;8],
    }
    
    #[derive(Debug)]
    pub enum ParseError {
        Len(usize), 
        Dev(String), 
        Time(String)}
    
    pub type FailableSample = Result<Sample, ParseError>;

    impl Sample {
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
