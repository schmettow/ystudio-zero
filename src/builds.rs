    const N: usize = 8;

    /// A board with baud rate
    #[derive(PartialEq, Debug, Clone)]
    pub struct Board (usize); // serial port rate
    
    /// Bank with number, interval, sensor array and optional labels
    #[derive(PartialEq, Debug, Copy, Clone)]
    pub enum Sensory { 
        MOI([Sensor; N]),
        AIR([Sensor; N]),
        ADC(f32, [Sensor; N]),
        YXZ(f32, [Sensor; N]),
    }

    type Sensor = Option<&'static str>; // Label or None

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
    pub struct Build<const S: usize> (&'static str, Board, [Sensory; S]);

    const GO: Board = Board(1_000_000);
    const PRO: Board = Board(2_000_000);

    const PRO_ZERO: Build::<2> = Build::<2>("Pro Zero", PRO, 
            [ Sensory::MOI([  Some("Blue Button"), None, None, None, None, None, None, None]), 
              Sensory::ADC(1.0/500.0, [  Some("A_0"), Some("A_1"), Some("A_2"), Some("A_3"), Some("A_4"), Some("Pin"), Some("Pin"), Some("Pin")])
              ]);

    const GO_ZERO: Build::<2> =
        Build::<2>("Pro Zero", PRO, 
            [ Sensory::MOI([Some("GP20"), Some("GP21"), Some("GP22"), None, None, None, None, None]), 
              Sensory::ADC(500.0, [  Some("Grv_1_W"), Some("Grv_1_Y"), Some("Pin"), Some("Int_T"), None, None, None, None]) // ADC
              ]);
    /*const GoMo
        Build("Go Mo", GO, 
            [ 
            Sensory::MOI(           [  None, None, None, None, None, None, None, None]), 
            Sensory::ADC(1.0/100.0, [  None, None, None, None, None, None, None, None]), 
            Sensory::YXZ(1.0/200.0, [  None, None, None, None,None, None, None, None]),
            ]),
        Build("Go Mo 4", GO, 
            [ 
            Sensory::MOI(           [  None, None, None, None, None, None, None, None]), 
            Sensory::ADC(1.0/100.0, [  None, None, None, None, None, None, None, None]), 
            Sensory::YXZ(1.0/200.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/200.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/200.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/200.0, [  None, None, None, None,None, None, None, None]),
            ]),
        Build("Go Mo 6", GO, [ 
            Sensory::MOI(           [  None, None, None, None, None, None, None, None]), 
            Sensory::ADC(1.0/100.0, [  None, None, None, None, None, None, None, None]), 
            Sensory::YXZ(1.0/150.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/150.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/150.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/150.0, [  None, None, None, None,None, None, None, None]),  
            Sensory::YXZ(1.0/150.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/150.0, [  None, None, None, None,None, None, None, None])
            ]),
        Build("Go Mo 8", GO, [ 
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]),  
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]),
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]), 
            Sensory::YXZ(1.0/100.0, [  None, None, None, None,None, None, None, None]), 
            ]),
        Build("Go Emo", GO, 
            [ 
            Sensory::MOI(           [  None, None, None, None, None, None, None, None]), 
            Sensory::ADC(1.0/500.0, [  None, None, None, None, None, None, None, None]), 
            Sensory::AIR(           [  Some("CO2"), Some("Humid"), Some("Temp"), None, None, None, None, None]),  
             None, None, None, None
            ]),
        ];*/
    
    