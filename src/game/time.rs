//! Ambient time system
//!
//! Handles time-based effects that tick during exploration.

use std::time::Duration;

/// Manages ambient time effects
pub struct AmbientTime {
    /// Total elapsed time in seconds
    elapsed: f32,
    /// Interval for effect ticks (in seconds)
    tick_interval: f32,
    /// Time since last tick
    since_last_tick: f32,
}

impl AmbientTime {
    /// Create a new ambient time manager
    pub fn new(tick_interval: f32) -> Self {
        Self {
            elapsed: 0.0,
            tick_interval,
            since_last_tick: 0.0,
        }
    }

    /// Update time and return true if a tick should occur
    pub fn update(&mut self, delta: Duration) -> bool {
        let delta_secs = delta.as_secs_f32();
        self.elapsed += delta_secs;
        self.since_last_tick += delta_secs;

        if self.since_last_tick >= self.tick_interval {
            self.since_last_tick -= self.tick_interval;
            true
        } else {
            false
        }
    }

    /// Get total elapsed time
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.since_last_tick = 0.0;
    }
}

impl Default for AmbientTime {
    fn default() -> Self {
        Self::new(1.0) // Default: tick every second
    }
}
