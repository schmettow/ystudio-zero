/// YLab Data (YLD)
/// 
/// provides structures to hold YLab data streams 
/// and methods to convert from and into 

pub mod yld{
    use core::fmt;

    #[derive(Copy, Clone)]
    pub struct Sample {
        pub dev: i8,
        pub time: i32,
        pub read: [u16;8],
    }
    
    impl Sample {
        pub fn from_csv_line(line: &String) -> Option<Self> {
            let cols: Vec<&str> = line.split(",").collect();
            if cols.len() != 10 {println!("{}", cols.len()); return None};
            let time = cols[1].parse::<i32>();
            if time.is_err() {return None}
            let dev = cols[0].parse::<i8>();
            if dev.is_err() {return None}
            let mut read: [u16; 8] = [0,0,0,0,0,0,0,0];
            for chn in 0..8 {
                read[chn] = cols[chn + 2]
                    .parse::<u16>()
                    .or::<u16>(Ok(0))
                    .unwrap()
            }
            Some(Sample{dev: dev.unwrap(), 
                        time: time.unwrap(), 
                        read: read})
        }
    }
    
    impl Default for Sample {
        fn default() -> Self {
            Self {dev: 0, time: 0, read: [0, 0, 0, 0, 0, 0, 0, 0]}
        }
    }
    }
    