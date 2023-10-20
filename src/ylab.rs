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

    #[derive(Copy, Clone, Debug)]
    pub struct Sample {
        pub dev: i8,
        pub time: i32,
        pub read: [u16;8],
    }
    
    #[derive(Debug)]
    pub enum ParseError {
        Len(usize), 
        Dev(String), 
        Time(String), 
        Read(String)}
    
    pub type FailableSample = Result<Sample, ParseError>;

    impl Sample {
        pub fn from_csv_line(line: &String) -> FailableSample {
            let cols: Vec<&str> = line.split(",").collect();
            if cols.len() != 10 {return Err(ParseError::Len(cols.len()))};
            
            let time = cols[0].parse::<i64>();
            if time.is_err() {return Err(ParseError::Time(cols[0].to_string()))}
            
            let dev = cols[1].parse::<i8>();
            if dev.is_err() {return Err(ParseError::Dev(cols[1].to_string()))}
            
            let mut read: [u16; 8] = [0,0,0,0,0,0,0,0];
            for chn in 0..8 {
                let value 
                    = cols[chn + 2].parse::<u16>();
                match value {
                    Ok(v) => read[chn] = v,
                    Err(_) => read[chn] = 0
                }
                read[chn] = cols[chn + 2]
                    .parse::<u16>()
                    .or::<u16>(Ok(0))
                    .unwrap()
            }
            Ok(Sample{dev: dev.unwrap(), 
                        time: time.unwrap() as i32, 
                        read: read})
        }

        pub fn to_csv_line(&self) -> String {
            let mut out: String = 
                [[self.time.to_string(), self.dev.to_string()]
                    .join(","),
                  self.read
                    .map(|r|{r.to_string()})
                    .join(",")]
                .join(",");
            //out.push_str("\r\n");
            return out
        }
    }
    
    impl Default for Sample {
        fn default() -> Self {
            Self {dev: 0, time: 0, read: [0, 0, 0, 0, 0, 0, 0, 0]}
        }
    }
    }
    