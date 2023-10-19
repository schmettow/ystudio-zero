//use crate::helpers;
use crate::measurements;
use crate::measurements::MeasurementWindow;
use crate::ylab::yld::Sample;

use serialport;
use std::collections::HashMap;
use std::io::Write;
use std::io::{self};
use std::sync::*;
use std::thread;
use std::time::{Duration, Instant};




pub fn serial_thread(
    measurements: Arc<Mutex<HashMap<String, MeasurementWindow>>>,
    serial_port: Arc<Mutex<String>>,
    available_ports: Arc<Mutex<Vec<String>>>,
    _variables: Arc<Mutex<Vec<String>>>,
    serial_data: Arc<Mutex<Vec<String>>>,
    send_serial: Arc<Mutex<bool>>,
    serial_write: Arc<Mutex<String>>,
) {
    loop {
        let serial_port = &serial_port.lock().unwrap().to_owned();

        let port = serialport::new(serial_port, 115_200)
            .timeout(Duration::from_millis(1))
            .flow_control(serialport::FlowControl::Software)
            .open();
        // Look for serial ports
        match serialport::available_ports() {
            Ok(n) => {
                available_ports.lock().unwrap().clear();
                for i in n {
                    available_ports.lock().unwrap().push(i.port_name);
                }
            }
            Err(e) => println!("{}", e),
        }

        match port {
            Ok(mut port) => {
                let _start_time = Instant::now();
                println!("Connected to port: {}", serial_port);
                let mut serial_buf: Vec<u8> = vec![0; 1000];
                loop {
                    {
                        let mut b = send_serial.lock().unwrap();
                        if *b {
                            let mut msg = serial_write.lock().unwrap();
                            match port.write(&*msg.as_bytes()) {
                                Ok(_) => {
                                    std::io::stdout().flush().unwrap();
                                    msg.clear();
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                Err(e) => eprintln!("{:?}", e),
                            }
                            *b = false;
                        }
                    }
                    match port.read(serial_buf.as_mut_slice()) {
                        Ok(_t) => {
                            let s = 
                                match String::from_utf8(serial_buf.to_owned()) {
                                    Ok(x) => x,
                                    Err(e) => {
                                        println!("{:?}", e);
                                        "CE,,,,,,,,,".to_string()
                                    }
                                };
                        let t = s.split("\r\n").collect::<Vec<&str>>();
                        for line in t {
                            let possible_sample = Sample::from_csv_line(&line.to_string());
                            match possible_sample {
                                None =>   {continue;},
                                Some(_) => {}}
                            let sample = possible_sample.unwrap();
                            let new_time = sample.time as f64;
                            let channels = [ "y0", "y1", "y2", "y3",
                                                        "y4", "y5", "y6", "y7"];
                            serial_data
                                    .lock()
                                    .unwrap()
                                    .push(s.to_string().trim().to_owned());
                            
                            for chn in channels {
                                measurements
                                .lock()
                                .unwrap()
                                .get_mut(chn)
                                .unwrap()
                                .add(measurements::Measurement::new(new_time, sample.read[0]));
                            }

                        }
                        

                        /*measurements
                            .lock()
                            .unwrap()
                            .get_mut(&*var)
                            .unwrap()
                            .add(measurements::Measurement::new(new_time, this_sample.read[2]));    
                            
                            
                            let t = s.split("\r\n").collect::<Vec<&str>>();
                            let s = t[0].to_string();
                            let variables = variables.lock().unwrap().clone();
                            serial_data
                                .lock()
                                .unwrap()
                                .push(s.to_string().trim().to_owned());

                            for var in variables {
                                if s.contains(&*var) {
                                    match helpers::parse_console(s.clone(), &*var) {
                                        Some(y) => {
                                            let new_time = start_time.elapsed().as_millis() as f64;
                                            measurements
                                                .lock()
                                                .unwrap()
                                                .get_mut(&*var)
                                                .unwrap()
                                                .add(measurements::Measurement::new(new_time, y));
                                        }

                                        None => continue,
                                    };
                                }
                            }*/
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }

                    thread::sleep(Duration::from_millis(1));
                }
            }
            Err(_e) => {
                // eprintln!("Failed to open {}", _e);
                // ::std::process::exit(1);
            }
        }
        thread::sleep(Duration::from_millis(1000));
    }
}
