//! Scratch: dump embedded-figure geometry per page to calibrate scan-tiling detection.
fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("read pdf");
        let dir = std::env::temp_dir().join(format!("tiling-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let written = rt
            .block_on(edgequake_pdf::write_embedded_figure_assets(
                &bytes, &dir, None,
            ))
            .expect("write figures");
        let map = edgequake_pdf::figures_by_page(&written);
        println!(
            "== {} ({} figs)",
            path.rsplit('/').next().unwrap(),
            written.len()
        );
        for (page, figs) in map.iter() {
            let areas: Vec<f64> = figs
                .iter()
                .map(|f| {
                    f.bbox
                        .map(|b| ((b.2 - b.0).abs() as f64) * ((b.3 - b.1).abs() as f64))
                        .unwrap_or((f.width as f64) * (f.height as f64))
                })
                .collect();
            let sum: f64 = areas.iter().sum();
            let max = areas.iter().cloned().fold(0.0, f64::max);
            let mut sorted = areas.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = sorted[sorted.len() / 2];
            let p25 = sorted[sorted.len() / 4];
            let p90 = sorted[(sorted.len() * 9) / 10];
            println!(
                "  page {page}: n={} tiling={} dominance={:.3} median={:.0} p25={:.0} p90={:.0} sum={:.0}",
                edgequake_pdf::is_scan_tiling_page(figs),
                figs.len(),
                if sum > 0.0 { max / sum } else { 0.0 },
                median,
                p25,
                p90,
                sum
            );
            for f in figs.iter().take(4) {
                println!(
                    "    fig-{:02} {}x{} bbox={:?}",
                    f.index, f.width, f.height, f.bbox
                );
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
