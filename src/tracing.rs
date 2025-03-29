use std::collections::VecDeque;
use std::time;

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

#[cfg(target_arch = "wasm32")]
use web_sys;

/// Monotonic clock timestamp for native builds.
#[cfg(not(target_arch = "wasm32"))]
type TimeInstant = time::Instant;

/// Monotonic web_sys millisecond timestamp, with up to microsecond resolution.
/// https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
#[cfg(target_arch = "wasm32")]
type TimeInstant = f64;

fn now() -> TimeInstant {
    #[cfg(not(target_arch = "wasm32"))]
    return time::Instant::now();

    #[cfg(target_arch = "wasm32")]
    web_sys::window()
        .expect("no window")
        .performance()
        .expect("no performance")
        .now()
}

/// Tracing tool for measuring durations, buffering them, plotting etc.
/// wasm compatible, not thread-safe.
#[derive(Default)]
pub struct Tracing {
    pub name: String,
    current: Option<TimeInstant>,
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
        self.current = Some(now());
    }

    /// Measures and stores the duration since the last start() in the buffer.
    /// Requires start() before.
    pub fn measure(&mut self) {
        if self.buffer.len() == self.max_buffer_size {
            self.buffer.pop_front();
        }
        let current = self.current.expect("missing start()");

        #[cfg(not(target_arch = "wasm32"))]
        self.buffer.push_back(current.elapsed());

        #[cfg(target_arch = "wasm32")]
        self.buffer.push_back(time::Duration::from_micros(
            ((now() - current) * 1e3) as u64,
        ));
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
