mod gui;
mod threads;
mod ylab;
mod ystudio;
use ystudio::*;

fn main() {
    // creating a bi-directional channel.
    // the commander sends commands to the YLab thread
    // the listener receives data from the YLab thread
    let (ylab_cmd, ylab_listen) 
        = mpsc::channel();
    // The ystudio object contains three coponents in a thread safe manner
    // + ylab_state, which is a mutexed YLabState
    // + ylab_data, which is a egui History of YLab Samples
    // + ui, which captures UI related variables
    let ystudio = ystudio::Ystudio::new(ylab_cmd);
    // The thread to collect Ylab data is started
    // consuming copies of ylab state, data and command listener
    thread::spawn(move || {
        threads::ylab_thread(
            ystudio.ylab_state,
            ystudio.ylab_data,
            ylab_listen
        );
    });

    // starting the egui, consuming the ystudio object.
    // The details of the GUI are in gui.rs.
    // The below works, because Ystudio objects implement eframe::App.
    gui::egui_init(ystudio);
}
