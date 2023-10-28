mod gui;
mod threads;
mod ylab;
mod ystudio;
use ystudio::*;

fn main() {
    let (ylab_cmd, ylab_listen) 
        = mpsc::channel();
    let ystudio = ystudio::Ystudio::new(ylab_cmd);
    // starting the serial thread, 
    // consuming all mutexes
    thread::spawn(move || {
        threads::ylab_thread(
            ystudio.ylab_state,
            ystudio.ylab_data,
            ylab_listen
        );
    });

    // starting the egui
    gui::egui_init(ystudio);
}
