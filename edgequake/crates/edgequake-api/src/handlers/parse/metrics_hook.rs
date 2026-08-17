//! Progress callback that records per-page timings for SPEC-094 metrics.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use edgequake_pdf2md::ConversionProgressCallback;

use super::types::PageTiming;

/// Collects wall-clock page timings from pdf2md progress callbacks.
#[derive(Debug)]
pub struct ParseMetricsHook {
    started: Instant,
    page_starts: Mutex<std::collections::HashMap<usize, Instant>>,
    timings: Mutex<Vec<PageTiming>>,
    render_ms: AtomicU64,
    ocr_ms: AtomicU64,
    first_page_at: Mutex<Option<Instant>>,
    last_page_at: Mutex<Option<Instant>>,
}

impl ParseMetricsHook {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            started: Instant::now(),
            page_starts: Mutex::new(std::collections::HashMap::new()),
            timings: Mutex::new(Vec::new()),
            render_ms: AtomicU64::new(0),
            ocr_ms: AtomicU64::new(0),
            first_page_at: Mutex::new(None),
            last_page_at: Mutex::new(None),
        })
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    pub fn page_timings(&self) -> Vec<PageTiming> {
        self.timings.lock().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn render_ms(&self) -> u64 {
        self.render_ms.load(Ordering::Relaxed)
    }

    pub fn ocr_ms(&self) -> u64 {
        self.ocr_ms.load(Ordering::Relaxed)
    }

    /// Heuristic assemble window: from last page complete to now, if available.
    pub fn assemble_ms_hint(&self) -> Option<u64> {
        let last = self.last_page_at.lock().ok()?.as_ref().copied()?;
        Some(last.elapsed().as_millis() as u64)
    }
}

impl ConversionProgressCallback for ParseMetricsHook {
    fn on_conversion_start(&self, _total_pages: usize) {
        // Treat time until first page start as approximate render.
        if let Ok(mut guard) = self.first_page_at.lock() {
            *guard = Some(Instant::now());
        }
    }

    fn on_page_start(&self, page_num: usize, _total_pages: usize) {
        if let Ok(mut starts) = self.page_starts.lock() {
            starts.insert(page_num, Instant::now());
        }
        if let Ok(mut first) = self.first_page_at.lock() {
            if first.is_none() {
                let now = Instant::now();
                *first = Some(now);
                let render = self.started.elapsed().as_millis() as u64;
                self.render_ms.store(render, Ordering::Relaxed);
            }
        }
    }

    fn on_page_complete(&self, page_num: usize, _total_pages: usize, markdown_len: usize) {
        let ms = self
            .page_starts
            .lock()
            .ok()
            .and_then(|mut starts| starts.remove(&page_num))
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);
        self.ocr_ms.fetch_add(ms, Ordering::Relaxed);
        if let Ok(mut timings) = self.timings.lock() {
            timings.push(PageTiming {
                page: page_num as u32,
                ms,
                chars: markdown_len as u64,
            });
        }
        if let Ok(mut last) = self.last_page_at.lock() {
            *last = Some(Instant::now());
        }
    }
}
