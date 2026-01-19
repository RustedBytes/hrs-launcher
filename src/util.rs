use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Check whether an optional cancellation flag has been raised.
#[must_use]
pub fn cancel_requested(cancel: &Option<Arc<AtomicBool>>) -> bool {
    cancel
        .as_ref()
        .map(|flag| flag.load(Ordering::SeqCst))
        .unwrap_or(false)
}

/// Render a human-friendly transfer speed string.
#[must_use]
pub fn format_speed(bytes_per_sec: f32) -> String {
    const KIB: f32 = 1024.0;
    const MIB: f32 = KIB * 1024.0;

    if bytes_per_sec < KIB {
        format!("{bytes_per_sec:.0} B/s")
    } else if bytes_per_sec < MIB {
        format!("{:.1} KB/s", bytes_per_sec / KIB)
    } else {
        format!("{:.1} MB/s", bytes_per_sec / MIB)
    }
}

/// Compute download progress as a percentage.
#[must_use]
pub fn progress_percent(downloaded: u64, total: Option<u64>) -> f32 {
    match total {
        Some(total) if total > 0 => (downloaded as f32 / total as f32) * 100.0,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn formats_speed_human_readable() {
        assert_eq!(format_speed(512.0), "512 B/s");
        assert_eq!(format_speed(2_048.0), "2.0 KB/s");
        assert_eq!(format_speed(5_242_880.0), "5.0 MB/s");
    }

    #[test]
    fn calculates_progress_percent() {
        assert_eq!(progress_percent(0, Some(10)), 0.0);
        assert_eq!(progress_percent(5, Some(10)), 50.0);
        assert_eq!(progress_percent(10, Some(10)), 100.0);
        assert_eq!(progress_percent(5, None), 0.0);
    }

    #[test]
    fn respects_optional_cancel_flag() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!cancel_requested(&Some(flag.clone())));
        flag.store(true, Ordering::SeqCst);
        assert!(cancel_requested(&Some(flag)));
        assert!(!cancel_requested(&None));
    }
}
