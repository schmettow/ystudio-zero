
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

type LockedPath = Arc<Mutex<Option<PathBuf>>>;
type LockedFile = Arc<Mutex<Option<File>>>;
type LockedState = Arc<Mutex<YldestState>>;

use std::fs::File;
pub fn yldest_thread(
    state: LockedState,
    listen: mpsc::Receiver<YldestCmd>,
    incoming: mpsc::Receiver<Yld>,
) -> ! {
    let locked_path: LockedPath = Arc::new(Mutex::new(None));
    let locked_file: LockedFile = Arc::new(Mutex::new(None));
    let mut write_buffer = String::new();
    loop {
        // getting the current state, incoming commands and incoming data
        let this_state = state.lock().unwrap().clone();
        let this_cmd = listen.try_recv().ok();
        let measure = incoming.try_recv().ok();
        // match the current state, command and data stream to do transitions
        match (this_state, this_cmd, measure) {
            
            // start recording on command
            (YldestState::Idle, 
            Some(YldestCmd::New { path }),
            _) 
            => {
                *locked_path.lock().unwrap() = Some(path.clone());
                *locked_file.lock().unwrap() = Some(fs::File::create(&path).unwrap());
                *state.lock().unwrap() = YldestState::Recording {path: path.clone()};
                println!("Recording to {}", path.to_str().unwrap());},
            
            // do recording when new data arrived
            (YldestState::Recording{path}, 
            _, 
            Some(measure))
            => {
                write_buffer.push_str(&measure.to_csv_line());
                if write_buffer.len() > 1000 {                    
                    locked_file
                        .lock().unwrap().as_ref().unwrap()
                        .write_all(write_buffer.as_bytes()).unwrap();
                    write_buffer.clear();
                }},
            
            // pause recording on command
            (YldestState::Recording{path}, 
            Some(YldestCmd::Pause), 
            _) 
            => {
                *state.lock().unwrap() = YldestState::Pausing{path};
            },

            // resume recording on command after pause
            (YldestState::Pausing{path}, Some(YldestCmd::Resume), _) 
                => {*state.lock().unwrap() = YldestState::Recording {path: path};},

            // stop recording on command
            (YldestState::Recording{path}, Some(YldestCmd::Stop), _) 
                => {
                    *state.lock().unwrap() = YldestState::Idle;
                },
            
            _ => {},
        }
    }
}
