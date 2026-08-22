fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("read pdf");
        let t0 = std::time::Instant::now();
        let a = edgequake_pdf::analyze_modality_blocking(&bytes).expect("analyze");
        let modality = edgequake_pdf::classify_document_from_signals(&a.pages, a.orientation_mixed);
        println!(
            "== {} → {} (orientation_mixed={}) in {:?}",
            path.rsplit('/').next().unwrap(),
            modality.as_str(),
            a.orientation_mixed,
            t0.elapsed()
        );
        for s in a.pages.iter().take(5) {
            println!(
                "  page {}: image_area={:.2} glyph_density={:.2} ink={:.3}",
                s.page_num, s.image_area_frac, s.glyph_text_density, s.ink_frac
            );
        }
    }
}
