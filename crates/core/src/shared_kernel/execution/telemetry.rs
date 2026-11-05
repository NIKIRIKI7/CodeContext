use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

pub const MAX_TELEMETRY_ENTRIES: usize = 512;
pub const SLOW_OPERATION_THRESHOLD_US: u64 = 15_000;
pub const CRITICAL_LAG_THRESHOLD_US: u64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LogLevel {
    #[default]
    Info,
    Perf,
    Slow,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LogFilterMode {
    #[default]
    All,
    SlowAndWarnings,
    ErrorsOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub seq: u64,
    pub elapsed_ms: u64,
    pub level: LogLevel,
    pub subsystem: &'static str,
    pub operation: &'static str,
    pub duration_us: Option<u64>,
    pub message: String,
}

impl LogEntry {
    #[inline]
    pub fn format_timestamp(&self) -> String {
        let total_secs = self.elapsed_ms / 1_000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        let millis = self.elapsed_ms % 1_000;
        format!("+{mins:02}:{secs:02}.{millis:03}")
    }

    #[inline]
    pub fn format_duration(&self) -> String {
        let Some(us) = self.duration_us else {
            return "—".to_owned();
        };
        if us < 1_000 {
            format!("{us} µs")
        } else {
            let whole_ms = us / 1_000;
            let frac_ms = (us % 1_000) / 10;
            format!("{whole_ms}.{frac_ms:02} ms")
        }
    }

    #[inline]
    pub const fn matches_filter(&self, filter: LogFilterMode) -> bool {
        match filter {
            LogFilterMode::All => true,
            LogFilterMode::SlowAndWarnings => {
                matches!(self.level, LogLevel::Slow | LogLevel::Warn | LogLevel::Error)
                    || matches!(self.duration_us, Some(us) if us >= SLOW_OPERATION_THRESHOLD_US)
            }
            LogFilterMode::ErrorsOnly => matches!(self.level, LogLevel::Error | LogLevel::Warn),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TelemetrySummary {
    pub total_count: usize,
    pub slow_or_warn_count: usize,
    pub error_count: usize,
    pub max_duration_us: u64,
}

#[derive(Debug)]
struct TelemetryInner {
    started_at: Instant,
    next_seq: u64,
    entries: VecDeque<LogEntry>,
}

#[derive(Debug, Clone)]
pub struct TelemetryLog {
    inner: Arc<Mutex<TelemetryInner>>,
}

impl Default for TelemetryLog {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryLog {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TelemetryInner {
                started_at: Instant::now(),
                next_seq: 1,
                entries: VecDeque::with_capacity(MAX_TELEMETRY_ENTRIES),
            })),
        }
    }

    pub fn record_timed(
        &self,
        subsystem: &'static str,
        operation: &'static str,
        duration: Duration,
        message: impl Into<String>,
    ) {
        let duration_us = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
        let level = if duration_us >= SLOW_OPERATION_THRESHOLD_US {
            LogLevel::Slow
        } else {
            LogLevel::Perf
        };
        self.push_entry(level, subsystem, operation, Some(duration_us), message.into());
    }

    pub fn record_info(
        &self,
        subsystem: &'static str,
        operation: &'static str,
        message: impl Into<String>,
    ) {
        self.push_entry(LogLevel::Info, subsystem, operation, None, message.into());
    }

    pub fn record_warn(
        &self,
        subsystem: &'static str,
        operation: &'static str,
        duration_opt: Option<Duration>,
        message: impl Into<String>,
    ) {
        let duration_us = duration_opt.map(|d| u64::try_from(d.as_micros()).unwrap_or(u64::MAX));
        self.push_entry(LogLevel::Warn, subsystem, operation, duration_us, message.into());
    }

    pub fn record_error(
        &self,
        subsystem: &'static str,
        operation: &'static str,
        duration_opt: Option<Duration>,
        message: impl Into<String>,
    ) {
        let duration_us = duration_opt.map(|d| u64::try_from(d.as_micros()).unwrap_or(u64::MAX));
        self.push_entry(LogLevel::Error, subsystem, operation, duration_us, message.into());
    }

    fn push_entry(
        &self,
        level: LogLevel,
        subsystem: &'static str,
        operation: &'static str,
        duration_us: Option<u64>,
        message: String,
    ) {
        let Ok(mut guard) = self.inner.lock() else {
            return;
        };
        let elapsed_ms = u64::try_from(guard.started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
        let seq = guard.next_seq;
        guard.next_seq = guard.next_seq.wrapping_add(1);
        if guard.entries.len() >= MAX_TELEMETRY_ENTRIES {
            let _ = guard.entries.pop_front();
        }
        guard.entries.push_back(LogEntry {
            seq,
            elapsed_ms,
            level,
            subsystem,
            operation,
            duration_us,
            message,
        });
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.entries.clear();
        }
    }

    pub fn entries_filtered(&self, filter: LogFilterMode) -> Vec<LogEntry> {
        let Ok(guard) = self.inner.lock() else {
            return Vec::new();
        };
        guard.entries.iter().rev().filter(|e| e.matches_filter(filter)).cloned().collect()
    }

    pub fn summary(&self) -> TelemetrySummary {
        let Ok(guard) = self.inner.lock() else {
            return TelemetrySummary::default();
        };
        let mut slow_or_warn_count = 0_usize;
        let mut error_count = 0_usize;
        let mut max_duration_us = 0_u64;
        for entry in &guard.entries {
            if let Some(us) = entry.duration_us
                && us > max_duration_us
            {
                max_duration_us = us;
            }
            if entry.matches_filter(LogFilterMode::SlowAndWarnings) {
                slow_or_warn_count += 1;
            }
            if matches!(entry.level, LogLevel::Error) {
                error_count += 1;
            }
        }
        TelemetrySummary {
            total_count: guard.entries.len(),
            slow_or_warn_count,
            error_count,
            max_duration_us,
        }
    }

    pub fn export_plain_text(&self, filter: LogFilterMode) -> String {
        let entries = self.entries_filtered(filter);
        let mut out = String::with_capacity(entries.len().saturating_mul(96) + 64);
        for entry in entries.into_iter().rev() {
            let _ = writeln!(
                out,
                "{} [{:<7}] {:>9}  {:<18} {}",
                entry.format_timestamp(),
                entry.subsystem,
                entry.format_duration(),
                entry.operation,
                entry.message
            );
        }
        out
    }
}
