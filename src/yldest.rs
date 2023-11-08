
use crate::ylab::*;
use crate::ylab::data::*;
use std::io::Write;
use std::sync::*;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum YldestState {
    Idle,
    Recording {path: PathBuf},
    Pausing {path: PathBuf}}

#[derive(Clone)]
pub enum YldestCmd {
    New {path: PathBuf},
    Pause,
    Resume,
    Stop,
}


pub fn yldest_thread(
    state: Arc<Mutex<YldestState>>,
    listen: mpsc::Receiver<YldestCmd>,
    incoming: mpsc::Receiver<Yld>,
) -> ! {
    loop {
        // getting the current state, incoming commands and incoming data
        let this_state = state.lock().unwrap().clone();
        let this_cmd = listen.try_recv().ok();
        let measure = incoming.try_recv().ok();
        // match the current state, command and data stream to do transitions
        match (this_state, this_cmd, measure) {
            // start recording on command
            (YldestState::Idle, Some(YldestCmd::New { path }), _) 
                => {
                let file = fs::File::create(path.clone());
                match file {
                    Ok(mut file) => {
                        file.write_all(data::YLD_HEAD.as_bytes()).unwrap();
                        *state.lock().unwrap() = YldestState::Recording {path};},
                    Err(_) => {
                        eprintln!("Could not create file");
                    },
                }
            },
            // do recording when new data arrived
            (YldestState::Recording{path}, _, Some(measure))
                => {
                    let file = fs::File::open(&path);
                    match file {
                        Ok(mut file) => {
                            file.write_all(measure.to_csv_line().as_bytes()).unwrap();},
                        Err(_) => {
                            eprintln!("Could not open file");
                        },
                    }
                },
            // pause recording on command
            (YldestState::Recording{path}, Some(YldestCmd::Pause), _) 
                => {
                    *state.lock().unwrap() = YldestState::Pausing{path};
                },

            // resume recording on command after pause
            (YldestState::Pausing{path}, Some(YldestCmd::Resume), _) 
                => {
                    let file = fs::File::open(&path);
                    match file {
                        Ok(mut file) => {
                            file.write_all(data::YLD_HEAD.as_bytes()).unwrap();
                            *state.lock().unwrap() = YldestState::Recording {path: path};},
                        Err(_) => {
                            eprintln!("Could not open file. Please close all other programs that might be using it.");
                        },
                    }
                },
            // stop recording on command
            (YldestState::Recording{path}, Some(YldestCmd::Stop), _) 
                => {
                    *state.lock().unwrap() = YldestState::Idle;
                },
            
            _ => {},
        }
    }
}
