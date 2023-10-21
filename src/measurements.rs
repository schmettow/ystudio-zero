/// A vecdeque is a queue with two ends.
/// It is used here to update the gliding window 
/// of the plot.
use std::collections::VecDeque;

/// Defining a dedicated type for one measurement point 
pub type Measurement = egui_plot::PlotPoint;
//pub type Measurement = Sample;

/// The measurement window is a gliding range of measures
/// that is used to keep the plot gliding (and not condense)
/// 
/// NOTES: 
/// 
/// + egui::util::History seems to implement the same functionality
/// + The width is given as a bare number, where it should use std::time::Duration
/// 
pub struct MeasurementWindow {
    pub values: VecDeque<Measurement>,
    pub look_behind: usize,
}

impl MeasurementWindow {
    /// Constructor
    /// look_behind is the width of the time window
    pub fn new_with_look_behind(look_behind: usize) -> Self {
        Self {
            values: VecDeque::new(),
            look_behind,
        }
    }
    /// Adding a measurement to the window
    /// x is the time axis. When the new time stamp is earlier than the last,
    /// the window is cleared.
    pub fn add(&mut self, measurement: Measurement) {
        // Checking the time stamp
        if let Some(last) = self.values.back() {
            if measurement.x < last.x {
                self.values.clear()
            }
        }
        // Adding the new measure
        self.values.push_back(measurement);
        // Collecting the time stamp from the just added measure
        let this_time = self.values.back().unwrap().x;
        // Re-calculating the lower time limit of the window
        let lower_time_limit = this_time - (self.look_behind as f64);
        // Removing points that are too old
        while let Some(front) = self.values.front() {
            if front.x >= lower_time_limit {
                break;
            }
            self.values.pop_front();
        }
    }


    /// A function to hand over the PlotPoints to the UI
    /// I can only guess, but it seems to create a deep copy.
    pub fn plot_values(&self) -> egui_plot::PlotPoints {
        egui_plot::PlotPoints::Owned(Vec::from_iter(self.values.iter().copied()))
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_measurements() {
        let w = MeasurementWindow::new_with_look_behind(123);
        assert_eq!(w.values.len(), 0);
        assert_eq!(w.look_behind, 123);
    }

    #[test]
    fn appends_one_value() {
        let mut w = MeasurementWindow::new_with_look_behind(100);

        w.add(Measurement::new(10.0, 20.0));
        assert_eq!(
            w.values.into_iter().eq(vec![Measurement::new(10.0, 20.0)]),
            true
        );
    }

    #[test]
    fn clears_on_out_of_order() {
        let mut w = MeasurementWindow::new_with_look_behind(100);

        w.add(Measurement::new(10.0, 20.0));
        w.add(Measurement::new(20.0, 30.0));
        w.add(Measurement::new(19.0, 100.0));
        assert_eq!(
            w.values.into_iter().eq(vec![Measurement::new(19.0, 100.0)]),
            true
        );
    }

    #[test]
    fn appends_several_values() {
        let mut w = MeasurementWindow::new_with_look_behind(100);

        for x in 1..=20 {
            w.add(Measurement::new((x as f64) * 10.0, x as f64));
        }

        assert_eq!(
            w.values.into_iter().eq(vec![
                Measurement::new(100.0, 10.0),
                Measurement::new(110.0, 11.0),
                Measurement::new(120.0, 12.0),
                Measurement::new(130.0, 13.0),
                Measurement::new(140.0, 14.0),
                Measurement::new(150.0, 15.0),
                Measurement::new(160.0, 16.0),
                Measurement::new(170.0, 17.0),
                Measurement::new(180.0, 18.0),
                Measurement::new(190.0, 19.0),
                Measurement::new(200.0, 20.0),
            ]),
            true
        );
    }
}
