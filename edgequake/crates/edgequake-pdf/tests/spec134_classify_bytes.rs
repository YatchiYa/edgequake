//! SPEC-134: byte-level document modality classification.
//!
//! `classify_document_from_bytes` is the entry point the ingestion pipeline
//! (edgequake-api `pdf_processing`) calls when no env override is set. These
//! tests build synthetic PDFs in-memory and assert the end-to-end verdict.

use edgequake_pdf::PageModality;
use lopdf::{dictionary, Document, Object, Stream};

/// One-page PDF fully covered by a single gray image XObject (a scan).
fn full_page_scan_pdf() -> Vec<u8> {
    let mut doc = Document::with_version("1.4");

    let (w, h) = (612i64, 792i64);
    let pixels = vec![250u8; (w * h) as usize];
    let img_id = doc.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => w,
            "Height" => h,
            "ColorSpace" => "DeviceGray",
            "BitsPerComponent" => 8,
        },
        pixels,
    ));

    let content_id = doc.add_object(Stream::new(
        dictionary! {},
        format!("q {w} 0 0 {h} 0 0 cm /Im1 Do Q").into_bytes(),
    ));

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "MediaBox" => vec![0.into(), 0.into(), w.into(), h.into()],
        "Resources" => dictionary! { "XObject" => dictionary! { "Im1" => img_id } },
        "Contents" => content_id,
    });
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => vec![Object::from(page_id)],
        "Count" => 1,
    });
    doc.get_object_mut(page_id)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("Parent", pages_id);
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    let mut buf = Vec::new();
    doc.save_to(&mut buf).unwrap();
    buf
}

/// One-page born-digital PDF: text only, no images.
fn born_digital_text_pdf() -> Vec<u8> {
    let mut doc = Document::with_version("1.4");

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let content_id = doc.add_object(Stream::new(
        dictionary! {},
        b"BT /F1 12 Tf 72 720 Td (Born-digital contract clause 1.1 payment terms) Tj ET".to_vec(),
    ));

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        "Resources" => dictionary! { "Font" => dictionary! { "F1" => font_id } },
        "Contents" => content_id,
    });
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Kids" => vec![Object::from(page_id)],
        "Count" => 1,
    });
    doc.get_object_mut(page_id)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .set("Parent", pages_id);
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);

    let mut buf = Vec::new();
    doc.save_to(&mut buf).unwrap();
    buf
}

#[test]
fn full_page_scan_classified_manuscript() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let modality = rt.block_on(edgequake_pdf::classify_document_from_bytes(
        &full_page_scan_pdf(),
    ));
    assert_eq!(
        modality,
        PageModality::Manuscript,
        "full-page image coverage must classify as manuscript"
    );
}

#[test]
fn born_digital_text_classified_print() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let modality = rt.block_on(edgequake_pdf::classify_document_from_bytes(
        &born_digital_text_pdf(),
    ));
    assert_eq!(
        modality,
        PageModality::Print,
        "text-only page must classify as print"
    );
}

#[test]
fn corrupt_bytes_fail_open_to_print() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let modality = rt.block_on(edgequake_pdf::classify_document_from_bytes(b"not a pdf"));
    assert_eq!(
        modality,
        PageModality::Print,
        "classification failure must fail open to print"
    );
}
