//! Runner statistics and metrics

use std::time::Duration;

/// Statistics for a SymbolRunner
#[derive(Debug, Clone)]
pub struct RunnerStats {
    /// Total ticks processed
    pub ticks_processed: u64,

    /// Total actions executed
    pub actions_executed: u64,

    /// Total errors encountered
    pub errors: u64,

    /// Average tick processing time
    pub avg_tick_duration: Duration,

    /// Minimum tick processing time
    pub min_tick_duration: Duration,

    /// Maximum tick processing time
    pub max_tick_duration: Duration,

    /// Total processing time
    total_duration: Duration,
}

impl RunnerStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self {
            ticks_processed: 0,
            actions_executed: 0,
            errors: 0,
            avg_tick_duration: Duration::ZERO,
            min_tick_duration: Duration::MAX,
            max_tick_duration: Duration::ZERO,
            total_duration: Duration::ZERO,
        }
    }

    /// Record a processed tick
    pub fn record_tick(&mut self, duration: Duration) {
        self.ticks_processed += 1;
        self.total_duration += duration;

        // Update min/max
        if duration < self.min_tick_duration {
            self.min_tick_duration = duration;
        }
        if duration > self.max_tick_duration {
            self.max_tick_duration = duration;
        }

        // Update average
        self.avg_tick_duration = self.total_duration / self.ticks_processed as u32;
    }

    /// Record an executed action
    pub fn record_action(&mut self) {
        self.actions_executed += 1;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    /// Get error rate (errors per 1000 ticks)
    pub fn error_rate(&self) -> f64 {
        if self.ticks_processed == 0 {
            return 0.0;
        }
        (self.errors as f64 / self.ticks_processed as f64) * 1000.0
    }

    /// Get action rate (actions per 100 ticks)
    pub fn action_rate(&self) -> f64 {
        if self.ticks_processed == 0 {
            return 0.0;
        }
        (self.actions_executed as f64 / self.ticks_processed as f64) * 100.0
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for RunnerStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_recording() {
        let mut stats = RunnerStats::new();

        stats.record_tick(Duration::from_millis(1));
        stats.record_tick(Duration::from_millis(2));
        stats.record_tick(Duration::from_millis(3));

        assert_eq!(stats.ticks_processed, 3);
        assert_eq!(stats.avg_tick_duration, Duration::from_millis(2));
        assert_eq!(stats.min_tick_duration, Duration::from_millis(1));
        assert_eq!(stats.max_tick_duration, Duration::from_millis(3));
    }

    #[test]
    fn test_error_rate() {
        let mut stats = RunnerStats::new();

        for _ in 0..1000 {
            stats.record_tick(Duration::from_millis(1));
        }
        for _ in 0..10 {
            stats.record_error();
        }

        assert_eq!(stats.error_rate(), 10.0); // 10 errors per 1000 ticks
    }

    #[test]
    fn test_action_rate() {
        let mut stats = RunnerStats::new();

        for _ in 0..100 {
            stats.record_tick(Duration::from_millis(1));
        }
        for _ in 0..5 {
            stats.record_action();
        }

        assert_eq!(stats.action_rate(), 5.0); // 5 actions per 100 ticks
    }
}
