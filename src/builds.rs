    const N: usize = 8;

    /// A board with baud rate
    #[derive(PartialEq, Debug, Clone)]
    
    
    
    pub struct Board (usize);
    
    /// Bank with number, interval, sensor array and optional labels
    #[derive(PartialEq, Debug, Copy, Clone)]
    pub enum Sensory { 
        MOI([Option<Option<&'static str>>; N]),
        AIR([Option<Option<&'static str>>; N]),
        ADC(f32, [Option<Option<&'static str>>; N]), // Sensors can be de-activbated and label is optional
        ACC(f32, [Option<Option<&'static str>>; N]), 
        YXZ(f32, [Option<Option<&'static str>>; N]),
    }

    impl Sensory {
        pub fn label(&self) -> String {
            let out = format!("{:?}", *self);
            return out.into();
        }
    }
    
    /// Build
    /// 
    /// a Build is a Board with up to N Sensories (or None)
    /// 

    #[derive(PartialEq, Debug, Clone)]
    pub struct Build (&'static str, Board, [Option<Sensory>;N]);

    const GO: Board = Board(1_000_000);
    const PRO: Board = Board(2_000_000);
    
    pub static BUILDS: [Build; 7] =
        [
        Build("Pro Zero", PRO, 
            [ 
                Some(Sensory::MOI(           [  Some(None), Some(None), Some(None), Some(None), Some(None), Some(None), Some(None), Some(None)])), 
                Some(Sensory::ADC(1.0/500.0, [  Some(None), Some(None), Some(None), Some(None), Some(None), Some(None), Some(None), Some(None)])), // 8 ADC
                None, None, None, None, None, None]),
        Build("Go Zero", GO, 
            [ 
            Some(Sensory::MOI(           [  Some(None), Some(None), Some(None), Some(None), None, None, None, None])), 
            Some(Sensory::ADC(1.0/500.0, [  Some(None), Some(None), Some(None), None, None, None, None, None])), // 3 ADC
            None, None, None, None, None, None]),
        Build("Go Mo", GO, 
            [ 
            Some(Sensory::MOI(           [  Some(None), Some(None), Some(None), Some(None), None, None, None, None])), 
            Some(Sensory::ADC(1.0/100.0, [  Some(None), Some(None), Some(None), None, None, None, None, None])), 
            Some(Sensory::YXZ(1.0/200.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])),  
            None, None, None, None, None
            ]),
        Build("Go Mo 4", GO, 
            [ 
            Some(Sensory::MOI(           [  Some(None), Some(None), Some(None), Some(None), None, None, None, None])), 
            Some(Sensory::ADC(1.0/100.0, [  Some(None), Some(None), Some(None), None, None, None, None, None])), 
            Some(Sensory::YXZ(1.0/200.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/200.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/200.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/200.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])),  
            None, None
            ]),
        Build("Go Mo 6", GO, [ 
            Some(Sensory::MOI(           [  Some(None), Some(None), Some(None), Some(None), None, None, None, None])), 
            Some(Sensory::ADC(1.0/100.0, [  Some(None), Some(None), Some(None), None, None, None, None, None])), 
            Some(Sensory::YXZ(1.0/150.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/150.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/150.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/150.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])),  
            Some(Sensory::YXZ(1.0/150.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/150.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None]))
            ]),
        Build("Go Mo 8", GO, [ 
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])),  
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])),
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            Some(Sensory::YXZ(1.0/100.0, [  Some(None), Some(None), Some(None), Some(None),Some(None), Some(None), None, None])), 
            ]),
        Build("Go Emo", GO, 
            [ 
            Some(Sensory::MOI(           [  Some(None), Some(None), Some(None), Some(None), None, None, None, None])), 
            Some(Sensory::ADC(1.0/500.0, [  Some(None), Some(None), Some(None), None, None, None, None, None])), 
            Some(Sensory::AIR(           [  Some(Some("CO2")), Some(Some("Humid")), Some(Some("Temp")), None, None, None, None, None])),  
            None, None, None, None, None
            ]),
        ];
    
    