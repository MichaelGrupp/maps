use std::collections::VecDeque;
use std::time;

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

/// Tool for measuring and plotting a fixed buffer of durations for tracing.
/// Not thread-safe, not for wasm.
#[derive(Default)]
pub struct Tracing {
    pub name: String,
    current: Option<time::Instant>,
    max_buffer_size: usize,
    buffer: VecDeque<time::Duration>,
}

impl Tracing {
    pub fn new(name: &str, max_buffer_size: usize) -> Tracing {
        Tracing {
            name: name.into(),
            current: None,
            max_buffer_size,
            buffer: VecDeque::with_capacity(max_buffer_size),
        }
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Start a new measurement.
    pub fn start(&mut self) {
        self.current = Some(time::Instant::now());
    }

    /// Measures and stores the duration since the last start() in the buffer.
    /// Requires start() before.
    pub fn measure(&mut self) {
        if self.buffer.len() == self.max_buffer_size {
            self.buffer.pop_front();
        }
        let current = self.current.expect("missing start()");
        self.buffer.push_back(current.elapsed());
    }

    /// Plots the buffered measurements as f64 seconds.
    pub fn plot(&self, ui: &mut egui::Ui) {
        let points: PlotPoints = (0..self.buffer_size())
            .map(|i| {
                [
                    i as f64,
                    self.buffer.get(i).expect("missing duration").as_secs_f64(),
                ]
            })
            .collect();
        Plot::new(self.name.as_str())
            .view_aspect(2.0)
            .show(ui, |plot_ui| plot_ui.line(Line::new(points)));
    }
}
