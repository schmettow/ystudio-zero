
use crate::ylab::*;
use crate::ylab::data::*;
use std::io::Write;
use std::sync::*;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum YldestState {
    Idle {dir:Option<PathBuf>},
    Connected {path:PathBuf},
    Recording {path: PathBuf}}

#[derive(Clone)]
pub enum YldestCmd {
    New {change_dir: Option<PathBuf>, file_name: Option<PathBuf>},
    Record, // add let _ = yld, so that collection truly goes on
    Pause,
    Stop,
}

use std::time::SystemTime;
pub fn auto_file_name() -> PathBuf {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => {
            let id = n.as_secs() - 1699743807;
            PathBuf::from(format!("{}.yld", id))
            },
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

type LockedPath = Arc<Mutex<Option<PathBuf>>>;
type LockedFile = Arc<Mutex<Option<File>>>;
type LockedState = Arc<Mutex<YldestState>>;

use std::fs::File;
//use std::time::Duration;
//use std::time::UNIX_EPOCH;
pub fn yldest_thread(
    state: LockedState,
    listen: mpsc::Receiver<YldestCmd>,
    incoming: mpsc::Receiver<Yld>,
) -> ! {
    let locked_path: LockedPath = Arc::new(Mutex::new(None));
    let locked_dir: LockedPath = Arc::new(Mutex::new(None));
    let locked_file: LockedFile = Arc::new(Mutex::new(None));
    let mut write_buffer = String::new();
    loop {
        // getting the current state, incoming commands and incoming data
        let this_state = state.lock().unwrap().clone();
        let this_cmd = listen.try_recv().ok();
        let measure = incoming.try_recv().ok();
        // match the current state, command and data stream to do transitions
        match (this_state, this_cmd, measure) {

            // start recording on command, None is default path
            (YldestState::Idle{dir}, 
            Some(YldestCmd::New {change_dir, file_name }),
            _) 
            => { 
                match (change_dir, file_name, dir) {
                    (None, _, None) 
                    => {eprintln! ("No path given, no default path set, no recording started.")},
                    // both dir and name are given
                    (Some(chdir) , Some(file_name), _) 
                    => {*locked_dir.lock().unwrap() = Some(chdir.clone());
                        let path = chdir.join(file_name);
                        *locked_path.lock().unwrap() = Some(path.clone());
                        *locked_file.lock().unwrap() = Some(fs::File::create(&path).unwrap());
                        *state.lock().unwrap() = YldestState::Connected {path: path.clone()};
                        println!("Recording to {:?}", path);},
                    // a file name is given, but no directory: try using existin locked_dir
                    (None , Some(path), Some(dir))
                    => {
                        let path = dir.join(path);
                        *locked_path.lock().unwrap() = Some(path.clone()); 
                        *locked_file.lock().unwrap() = Some(fs::File::create(&path).unwrap());
                        *state.lock().unwrap() = YldestState::Connected {path: path.clone()};
                        println!("Recording to {:?}", path);},
                    // a directory is given, but no file name -> auto naming
                    (Some(dir), None, _) | (None, None, Some(dir))
                    => {
                        *locked_dir.lock().unwrap() = Some(dir.clone()); 
                        let path = auto_file_name();
                        *locked_path.lock().unwrap() = Some(path.clone()); 
                        *locked_file.lock().unwrap() = Some(fs::File::create(&path).unwrap());
                        *state.lock().unwrap() = YldestState::Connected {path: path.clone()};
                        println!("Recording to {:?}", path);},
                    }},
            
            // on command switch to recording state
            (YldestState::Connected{path},  
            _, //Some(YldestCmd::Record), 
            _ )
            => {*state.lock().unwrap() = YldestState::Recording {path: path.clone()}},  

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
                *state.lock().unwrap() = YldestState::Connected{path};
            },

            // stop recording on command, keep path
            (YldestState::Recording{path}, Some(YldestCmd::Stop), _) 
                => {
                    *state.lock().unwrap() = YldestState::Idle{dir: locked_dir.lock().unwrap().clone()};
                },
            
            _ => {},
        }
    }
}
