
    /// A board with baud rate
    #[derive(PartialEq, Debug, Clone)]
    pub struct Board (usize);
    
    /// Bank with number, interval, sensor array and optional labels
    #[derive(PartialEq, Debug, Copy, Clone)]
    pub enum Bank { ADC(u8, f32, [Option<Option<&'static str>>; 8]), 
                    I2C(u8, f32, [Option<Option<&'static str>>; 8]),
                    YXZ(u8, f32, [Option<Option<&'static str>>; 8]),}
    
    impl Bank {
        pub fn label(&self) -> &'static str{
            let out = format!("{:?}", *self);
            return &*out;
        }
    }
    
    
    #[derive(PartialEq, Debug, Clone)]
    pub struct Build (&'static str, Board, [Option<Bank>;8]);
    
    static BUILDS: [Option<Build>; 8] =
        [
            Some(Build("None", 
                        Board(0), [None;8])), 
            Some(Build("Go", 
                        Board(1_000_000), [Some(Bank::ADC(0, 1.0/500.0, [  Some(None), Some(None), Some(None), Some(Some("temp_MPU")),
                                                                            None, None, None, None])), 
                                            None, None, None, None, None, None, None])),
            Some(Build("Pro", 
                        Board(2_000_000), [Some(Bank::ADC(0, 1.0/500.0, [ Some(None), Some(None), Some(None), Some(None),
                                                                                Some(None), Some(None), Some(None), Some(None)])), 
                                                None, None, None, None, None, None, None])),
            Some(Build("Go Motion 4", 
                        Board(1_000_000), [ Some(Bank::ADC(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(Some("temp_MPU")),
                                                                            None, None, None, None])), 
                                            Some(Bank::YXZ(0, 1.0/250.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/250.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/250.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/250.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])),  
                                            None, None, None])),
            Some(Build("Go Motion 7", 
                        Board(1_000_000), [ Some(Bank::ADC(0, 1.0/50.0, [  Some(None), Some(None), Some(None), Some(Some("temp_MPU")),
                                                                            None, None, None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])),  
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])),  
                                            ])),
            Some(Build("Go Motion 8", 
                        Board(1_000_000), [ Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(Some("temp_MPU")),
                                                                            None, None, None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])),  
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])), 
                                            Some(Bank::YXZ(0, 1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None),
                                                                            None, None])),  
                                            ])),
            None,
            None,
        ];
    
    