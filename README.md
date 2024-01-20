# Ystudio Zero

YLab Edge is a set of firmwares for capturing biosignals with 
easy-to-build micro controller systems. Ystudio is a desktop user interface 
to capture and visualize biosignals. It currently supports YLab Edge  [Pro](../ylab-edge-pro/), [Go](../ylab-edge-go/) and [Mini](../ylab-edge-go/).

The following features are currently supported:

+   painless connection to YLab systems with automatic configuration
+   raw signal view, with adjustable low-pass filter
+   real-time spectrogram using channel-wise FFT
+   Recording in long format.

## Technical description

Ystudio uses Egui for the user interface. The architecture is multi-threaded and 
contains *three main tasks*:

+   the user interface
+  *YLab* connects to the serialport, reads the indoming data and sends 
    it to the ui and the recording compnent
+  *Yst* is the storage task, receiving data and writing it to files.

## Funding

Ystudio is funded by *WSV Innovation Funds, University of Twente*

## Installation 

### Developer

1. Install the Rust tool chain: https://rustup.rs/
2. Clone this repository using your favorite editor or the command line: `git clone
3. Build and run the project: `cargo run`

### End User

Create a directory on your computer, where you want to store the data. Download the .exe file and put it into this directory. Double-click on the program to start.
[Ystudio Zero for Windows](target/release/ystudio-zero.exe)

coming_soon: downloads for Mac and Linux.


## Usage

1. Connect your Ylab sensor to your computer
2. Select the correct serial port in the dropdown menu
3. Select the correct YLab version in the dropdown menu
4. Press the `Connect` button
5. Press the `Read` button
6. Use the check boxes to select the channels you want to see

# Technical details

+ The project is written in Rust
+ The GUI is written in Egui, using egui-plot for plotting
+ The serial communication is handled by the serialport crate
+ The architecture is multi-threaded, using std::sync features

## YLab compatibility

Ystudio Zero is compatible with all YLab Edge versions (Pro, Go, Mini). The YLab version can be selected in the GUI. The YLab version determines the baud rate.

## Data structure and formats

Data from YLabs currently arrive as YLab Transport Format with 8 channels *YTF8* ("why-the-fate"). This format is designed to be very efficient for high-throughput applications, especially EEG. In Ystudio, the data is converted to YLab Long Data *YLD* ("wild") format, which is more convenient for plotting and storage. It has the following signature:
    
    ```rust
    pub struct Yld {
        pub timestamp: u64,
        pub dev: u8,
        pub channel: u8,
        pub value: f32,
    }
    ```
where *timestamp* is the time-of arrival at the Ystudio application, *dev* is the device number (0..7) and will later be used for setups with more than one YLab. *Channel* is the channel number (0..7), based on the order of the value in the original Ytf8.

While Yld is teh prefered format for internal processing and data sharing, *Ytf8* is only used for data transfer up the USB port. Ytf8 has the same signature as Yld, except it delivers a vector of eight channels at once. For larger sensor arrays, e.g. a bank of EEG electrodes, this aves a lot of time stamps, which is a very expensive column (8 Bytes). Ytf8 saves quite some bandwidth on the serial line, which is the bottle neck. 

Yld is basically just the long format of Ytf8. It is also independent of how the data arrives.
For the user the long format is more convenient for plotting, storage and tidy data processing.

## Multi-threaded architecture

The *main thread* initializes channels and other data sharing structures, then starts the other threads and the GUI
The *ylab thread* reads data from the serial port, converts it to YLab long data (YLD). The data is then send to a
History buffer for continuous plotting. Using a channel, the data is also send to the YLD External STorage *Yldest* ("wildest") threat for storage.
The *yldest thread*  receives YLD stream from the Ylab thread and stores it in a csv file.

YLab and Yldest threats are designed as state machines, using enums and match statements. Both have a command channel for control. 
Usually, the GUI thread sends these commands to YLab/Yldest on user event (e.g. button clicked). The YLab/Yldest thread then changes its state (without confirmation).


