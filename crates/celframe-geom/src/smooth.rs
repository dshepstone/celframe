//! Turns raw pen samples into the smoothed centreline stored in the document.

use celframe_core::StrokeSample;

/// Smooths samples as they arrive, so a stroke in progress and a finished stroke match.
///
/// Each interior sample becomes a weighted average of itself and its neighbours (1-2-1), run
/// twice. A sample's output is only final once enough later samples exist, so `push` returns the
/// samples that became final. The first and last input samples are always kept exactly.
#[derive(Clone, Debug)]
pub struct StrokeSmoother {
    min_distance: f64,
    filtered: Vec<StrokeSample>,
    emitted: usize,
}

/// Later samples needed before a sample stops changing: one per smoothing pass.
const LOOKAHEAD: usize = 2;

impl StrokeSmoother {
    /// Samples closer than `min_distance` scene units to the last kept sample are dropped.
    pub fn new(min_distance: f64) -> Self {
        Self {
            min_distance,
            filtered: Vec::new(),
            emitted: 0,
        }
    }

    /// Adds a raw sample and returns any samples that are now final.
    /// Samples with non-finite values are ignored, and pressure is clamped to 0.0..=1.0.
    pub fn push(&mut self, sample: StrokeSample) -> Vec<StrokeSample> {
        if !sample.x.is_finite() || !sample.y.is_finite() || !sample.pressure.is_finite() {
            return Vec::new();
        }
        let sample = StrokeSample {
            pressure: sample.pressure.clamp(0.0, 1.0),
            ..sample
        };
        if let Some(last) = self.filtered.last()
            && (sample.x - last.x).hypot(sample.y - last.y) < self.min_distance
        {
            return Vec::new();
        }
        self.filtered.push(sample);
        let ready = self.filtered.len().saturating_sub(LOOKAHEAD);
        self.emit_up_to(ready)
    }

    /// Returns the remaining samples. The stroke ends exactly on the last sample it kept.
    pub fn finish(mut self) -> Vec<StrokeSample> {
        let len = self.filtered.len();
        self.emit_up_to(len)
    }

    fn emit_up_to(&mut self, end: usize) -> Vec<StrokeSample> {
        let output: Vec<StrokeSample> = (self.emitted..end).map(|i| self.smoothed(i)).collect();
        self.emitted = end;
        output
    }

    fn smoothed(&self, index: usize) -> StrokeSample {
        pass(index, 2, &self.filtered)
    }
}

fn pass(index: usize, passes: u32, samples: &[StrokeSample]) -> StrokeSample {
    // Ends stay fixed in every pass, which also makes the window shrink near them.
    if passes == 0 || index == 0 || index + 1 >= samples.len() {
        return samples[index];
    }
    let previous = pass(index - 1, passes - 1, samples);
    let current = pass(index, passes - 1, samples);
    let next = pass(index + 1, passes - 1, samples);
    StrokeSample {
        x: 0.25 * previous.x + 0.5 * current.x + 0.25 * next.x,
        y: 0.25 * previous.y + 0.5 * current.y + 0.25 * next.y,
        pressure: 0.25 * previous.pressure + 0.5 * current.pressure + 0.25 * next.pressure,
    }
}

/// Smooths a complete list of raw samples.
pub fn smooth(samples: &[StrokeSample], min_distance: f64) -> Vec<StrokeSample> {
    let mut smoother = StrokeSmoother::new(min_distance);
    let mut output = Vec::with_capacity(samples.len());
    for sample in samples {
        output.extend(smoother.push(*sample));
    }
    output.extend(smoother.finish());
    output
}
