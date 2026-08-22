//! SPEC-134 contract: Graphic-as-unit Pass-B suppression for manuscript pages.

use edgequake_pdf::{should_suppress_crop_manuscript, CropDescriptor, PageModality};

#[test]
fn tick_strip_suppressed_on_manuscript() {
    let crop = CropDescriptor {
        area_frac: 0.05,
        ink_frac: 0.05,
        aspect_ratio: 0.08, // tall thin strip = axis tick
        is_chart_fragment: false,
    };
    assert!(
        should_suppress_crop_manuscript(PageModality::Manuscript, &crop),
        "Tick strip must be suppressed on manuscript pages"
    );
}

#[test]
fn single_bar_fragment_suppressed() {
    let crop = CropDescriptor {
        area_frac: 0.02,
        ink_frac: 0.06,
        aspect_ratio: 0.3,
        is_chart_fragment: true, // child of larger chart
    };
    assert!(
        should_suppress_crop_manuscript(PageModality::Manuscript, &crop),
        "Chart fragment must be suppressed"
    );
}

#[test]
fn scribble_suppressed_low_ink() {
    let crop = CropDescriptor {
        area_frac: 0.01,
        ink_frac: 0.005, // nearly empty
        aspect_ratio: 1.2,
        is_chart_fragment: false,
    };
    assert!(
        should_suppress_crop_manuscript(PageModality::Manuscript, &crop),
        "Low-ink scribble must be suppressed"
    );
}

#[test]
fn whole_chart_not_suppressed() {
    let crop = CropDescriptor {
        area_frac: 0.25,
        ink_frac: 0.12,
        aspect_ratio: 1.8,
        is_chart_fragment: false,
    };
    assert!(
        !should_suppress_crop_manuscript(PageModality::Manuscript, &crop),
        "Whole chart must NOT be suppressed"
    );
}

#[test]
fn print_modality_no_suppression() {
    let crop = CropDescriptor {
        area_frac: 0.001,
        ink_frac: 0.001,
        aspect_ratio: 0.05,
        is_chart_fragment: true,
    };
    assert!(
        !should_suppress_crop_manuscript(PageModality::Print, &crop),
        "Print modality must never suppress"
    );
}

#[test]
fn mixed_modality_suppresses_like_manuscript() {
    let crop = CropDescriptor {
        area_frac: 0.005,
        ink_frac: 0.05,
        aspect_ratio: 1.0,
        is_chart_fragment: false,
    };
    assert!(
        should_suppress_crop_manuscript(PageModality::Mixed, &crop),
        "Mixed modality must suppress like manuscript"
    );
}
