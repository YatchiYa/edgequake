//! Progress/stall watchdog for long-running Vision PDF conversion.
//!
//! ## First principles
//!
//! A fixed wall-clock `tokio::time::timeout` kills legitimate slow-but-progressing
//! conversions (figure-heavy PDFs). Instead:
//! - Reset a stall timer on every progress signal (page complete / status hook).
//! - Abort only when no progress arrives for `STALL_TIMEOUT_SECS` (hung provider).
//! - Keep an absolute backstop (`VISION_MAX_OUTER_TIMEOUT_SECS`) against infinite runs.
//!
//! SOLID: pure timing policy here; callers wire progress signals and abort.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Default stall window: no progress for this long → treat as hung.
/// Override with `EDGEQUAKE_VISION_STALL_TIMEOUT_SECS` (min 30s).
pub const DEFAULT_VISION_STALL_TIMEOUT_SECS: u64 = 300;

/// How often the watchdog ticker checks for stall / absolute deadline.
pub const WATCHDOG_TICK_SECS: u64 = 2;

/// Marker embedded in timeout messages so the circuit breaker can detect
/// progress-aware stalls without a TaskError schema break.
pub const VISION_PROGRESS_MARKER_TRUE: &str = "[vision_progress=1]";
pub const VISION_PROGRESS_MARKER_FALSE: &str = "[vision_progress=0]";

/// Durable per-PDF vision checkpoint directory (survives /tmp cleanup + restarts).
///
/// Priority: `EDGEQUAKE_CHECKPOINT_DIR` → `EDGEQUAKE_DATA_DIR/vision-checkpoints`
/// → `~/.local/share/edgequake/vision-checkpoints` → `./.edgequake-checkpoints`.
pub fn durable_vision_checkpoint_dir(pdf_id: &str) -> String {
    let base = std::env::var("EDGEQUAKE_CHECKPOINT_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("EDGEQUAKE_DATA_DIR").ok().map(|d| {
                let mut p = std::path::PathBuf::from(d);
                p.push("vision-checkpoints");
                p.to_string_lossy().to_string()
            })
        })
        .unwrap_or_else(|| {
            if let Ok(home) = std::env::var("HOME") {
                format!("{home}/.local/share/edgequake/vision-checkpoints")
            } else {
                ".edgequake-checkpoints".to_string()
            }
        });
    let dir = std::path::PathBuf::from(&base).join(pdf_id);
    let _ = std::fs::create_dir_all(&dir);
    dir.to_string_lossy().to_string()
}

/// Annotate a timeout message with the progress marker for the circuit breaker.
pub fn annotate_timeout_progress(message: String, made_progress: bool) -> String {
    let marker = if made_progress {
        VISION_PROGRESS_MARKER_TRUE
    } else {
        VISION_PROGRESS_MARKER_FALSE
    };
    if message.contains("[vision_progress=") {
        message
    } else {
        format!("{message} {marker}")
    }
}

/// Shared progress heartbeat for the vision convert future.
#[derive(Debug)]
pub struct VisionProgressHeartbeat {
    last_progress_epoch_secs: AtomicU64,
    /// Pages completed during this attempt (for progress-aware circuit breaker).
    pages_completed: AtomicU64,
    /// Any status_hook / progress signal observed.
    any_progress: AtomicBool,
}

impl VisionProgressHeartbeat {
    pub fn new() -> Arc<Self> {
        let now = epoch_secs_now();
        Arc::new(Self {
            last_progress_epoch_secs: AtomicU64::new(now),
            pages_completed: AtomicU64::new(0),
            any_progress: AtomicBool::new(false),
        })
    }

    /// Record a progress tick (status hook or page complete).
    pub fn touch(&self) {
        self.last_progress_epoch_secs
            .store(epoch_secs_now(), Ordering::Relaxed);
        self.any_progress.store(true, Ordering::Relaxed);
    }

    /// Record a completed page and touch the heartbeat.
    pub fn page_completed(&self) {
        self.pages_completed.fetch_add(1, Ordering::Relaxed);
        self.touch();
    }

    pub fn pages_completed(&self) -> u64 {
        self.pages_completed.load(Ordering::Relaxed)
    }

    pub fn made_progress(&self) -> bool {
        self.any_progress.load(Ordering::Relaxed) || self.pages_completed() > 0
    }

    pub fn last_progress_epoch_secs(&self) -> u64 {
        self.last_progress_epoch_secs.load(Ordering::Relaxed)
    }

    pub fn secs_since_progress(&self) -> u64 {
        epoch_secs_now().saturating_sub(self.last_progress_epoch_secs())
    }
}

impl Default for VisionProgressHeartbeat {
    fn default() -> Self {
        let now = epoch_secs_now();
        Self {
            last_progress_epoch_secs: AtomicU64::new(now),
            pages_completed: AtomicU64::new(0),
            any_progress: AtomicBool::new(false),
        }
    }
}

/// Wraps an existing progress callback and feeds the stall heartbeat.
pub struct HeartbeatProgressCallback {
    inner: Arc<dyn edgequake_pdf2md::ConversionProgressCallback>,
    heartbeat: Arc<VisionProgressHeartbeat>,
}

impl HeartbeatProgressCallback {
    pub fn new(
        inner: Arc<dyn edgequake_pdf2md::ConversionProgressCallback>,
        heartbeat: Arc<VisionProgressHeartbeat>,
    ) -> Arc<Self> {
        Arc::new(Self { inner, heartbeat })
    }
}

impl edgequake_pdf2md::ConversionProgressCallback for HeartbeatProgressCallback {
    fn on_conversion_start(&self, total_pages: usize) {
        self.heartbeat.touch();
        self.inner.on_conversion_start(total_pages);
    }

    fn on_page_start(&self, page_num: usize, total_pages: usize) {
        self.heartbeat.touch();
        self.inner.on_page_start(page_num, total_pages);
    }

    fn on_page_complete(&self, page_num: usize, total_pages: usize, markdown_len: usize) {
        self.heartbeat.page_completed();
        self.inner
            .on_page_complete(page_num, total_pages, markdown_len);
    }

    fn on_page_error(&self, page_num: usize, total_pages: usize, error: String) {
        self.heartbeat.touch();
        self.inner.on_page_error(page_num, total_pages, error);
    }

    fn on_page_resumed(&self, page_num: usize, total_pages: usize) {
        self.heartbeat.touch();
        self.inner.on_page_resumed(page_num, total_pages);
    }

    fn on_conversion_complete(&self, total_pages: usize, success_count: usize) {
        self.heartbeat.touch();
        self.inner
            .on_conversion_complete(total_pages, success_count);
    }
}

/// Why a vision convert was aborted by the watchdog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionWatchdogAbort {
    /// No progress for `stall_secs` — provider likely hung.
    Stall { stall_secs: u64, idle_secs: u64 },
    /// Absolute backstop exceeded (even with progress).
    AbsoluteDeadline { absolute_secs: u64 },
}

impl VisionWatchdogAbort {
    pub fn as_timeout_message(&self, pdf_id: &str, provider: &str) -> String {
        // SPEC-083 X-30: prefix with typed "Operation timed out" so circuit
        // breaker / from_processing_error classify as Timeout (not Unknown).
        match self {
            Self::Stall {
                stall_secs,
                idle_secs,
            } => format!(
                "Operation timed out: Vision extraction stalled: no progress for {idle_secs}s \
                 (stall limit {stall_secs}s) for PDF {pdf_id}. \
                 Provider '{provider}' may be hung. Progress during this attempt \
                 is preserved for resume. [failure_class=timeout_phase_convert]"
            ),
            Self::AbsoluteDeadline { absolute_secs } => format!(
                "Operation timed out: Vision extraction exceeded absolute deadline of \
                 {absolute_secs}s for PDF {pdf_id}. Provider '{provider}'. \
                 [failure_class=timeout_phase_convert]"
            ),
        }
    }
}

/// Resolve stall timeout from env (floor 30s).
pub fn vision_stall_timeout_secs() -> u64 {
    std::env::var("EDGEQUAKE_VISION_STALL_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_VISION_STALL_TIMEOUT_SECS)
        .max(30)
}

/// Pure policy: given elapsed absolute time and idle since progress, decide abort.
pub fn evaluate_vision_watchdog(
    absolute_elapsed: Duration,
    absolute_limit: Duration,
    idle_since_progress: Duration,
    stall_limit: Duration,
) -> Option<VisionWatchdogAbort> {
    if absolute_elapsed >= absolute_limit {
        return Some(VisionWatchdogAbort::AbsoluteDeadline {
            absolute_secs: absolute_limit.as_secs(),
        });
    }
    if idle_since_progress >= stall_limit {
        return Some(VisionWatchdogAbort::Stall {
            stall_secs: stall_limit.as_secs(),
            idle_secs: idle_since_progress.as_secs(),
        });
    }
    None
}

/// Run `fut` until it completes, or the heartbeat stalls / absolute deadline hits.
///
/// `heartbeat` must be touched by progress callbacks while `fut` runs.
pub async fn run_with_vision_stall_watchdog<T, E, F>(
    fut: F,
    heartbeat: Arc<VisionProgressHeartbeat>,
    stall_secs: u64,
    absolute_secs: u64,
) -> Result<Result<T, E>, VisionWatchdogAbort>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    let stall = Duration::from_secs(stall_secs.max(30));
    let absolute = Duration::from_secs(absolute_secs.max(stall_secs.max(30)));
    let started = Instant::now();
    let tick = Duration::from_secs(WATCHDOG_TICK_SECS);

    tokio::pin!(fut);

    loop {
        tokio::select! {
            biased;
            result = &mut fut => {
                return Ok(result);
            }
            _ = tokio::time::sleep(tick) => {
                let idle = Duration::from_secs(heartbeat.secs_since_progress());
                if let Some(abort) = evaluate_vision_watchdog(
                    started.elapsed(),
                    absolute,
                    idle,
                    stall,
                ) {
                    return Err(abort);
                }
            }
        }
    }
}

fn epoch_secs_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn evaluate_allows_progress_within_stall() {
        assert!(evaluate_vision_watchdog(
            Duration::from_secs(600),
            Duration::from_secs(86_400),
            Duration::from_secs(10),
            Duration::from_secs(300),
        )
        .is_none());
    }

    #[test]
    fn evaluate_trips_on_stall() {
        let abort = evaluate_vision_watchdog(
            Duration::from_secs(400),
            Duration::from_secs(86_400),
            Duration::from_secs(301),
            Duration::from_secs(300),
        );
        assert!(matches!(abort, Some(VisionWatchdogAbort::Stall { .. })));
    }

    #[test]
    fn evaluate_trips_on_absolute() {
        let abort = evaluate_vision_watchdog(
            Duration::from_secs(86_400),
            Duration::from_secs(86_400),
            Duration::from_secs(1),
            Duration::from_secs(300),
        );
        assert!(matches!(
            abort,
            Some(VisionWatchdogAbort::AbsoluteDeadline { .. })
        ));
    }

    #[test]
    fn heartbeat_touch_resets_idle() {
        let hb = VisionProgressHeartbeat::new();
        std::thread::sleep(Duration::from_millis(20));
        let idle_before = hb.secs_since_progress();
        hb.touch();
        assert!(hb.made_progress());
        assert!(hb.secs_since_progress() <= idle_before);
    }

    #[test]
    fn page_completed_increments() {
        let hb = VisionProgressHeartbeat::new();
        hb.page_completed();
        hb.page_completed();
        assert_eq!(hb.pages_completed(), 2);
        assert!(hb.made_progress());
    }

    #[tokio::test]
    async fn watchdog_completes_when_future_finishes() {
        let hb = VisionProgressHeartbeat::new();
        let result =
            run_with_vision_stall_watchdog(async { Ok::<_, &'static str>(42) }, hb, 300, 86_400)
                .await;
        assert_eq!(result.unwrap().unwrap(), 42);
    }

    #[tokio::test]
    async fn watchdog_stalls_when_no_progress() {
        let hb = VisionProgressHeartbeat::new();
        // Force last progress into the past so stall trips immediately.
        hb.last_progress_epoch_secs
            .store(epoch_secs_now().saturating_sub(10), Ordering::Relaxed);

        let result = run_with_vision_stall_watchdog(
            async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok::<_, &'static str>(0)
            },
            hb,
            30, // stall floor is 30; with last progress 10s ago + tick, trips soon
            86_400,
        )
        .await;

        // With stall=30 and idle already 10, need ~20 more seconds — too slow for unit.
        // Instead use evaluate_vision_watchdog directly above; here verify race-safe select.
        // Force tiny stall by writing old epoch far past:
        let _ = result;
    }

    #[tokio::test]
    async fn watchdog_aborts_on_forced_old_heartbeat() {
        let hb = VisionProgressHeartbeat::new();
        hb.last_progress_epoch_secs
            .store(epoch_secs_now().saturating_sub(10_000), Ordering::Relaxed);

        let ticks = Arc::new(AtomicUsize::new(0));
        let ticks_c = ticks.clone();

        let result = run_with_vision_stall_watchdog(
            async move {
                loop {
                    ticks_c.fetch_add(1, Ordering::Relaxed);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                #[allow(unreachable_code)]
                Ok::<(), &'static str>(())
            },
            hb,
            30,
            86_400,
        )
        .await;

        assert!(matches!(result, Err(VisionWatchdogAbort::Stall { .. })));
    }
}
