//! Per-page convert grouping for SPEC-134 Slice E.
//!
//! pdf2md 0.9.11 takes one `ConversionConfig` per `convert_from_bytes`. Mixed
//! documents therefore convert in **two groups** (`PageSelection::Set`) and
//! stitch markdown by page marker — print pages keep Acc English + figure
//! filter; manuscript pages get the MS profile (LAW-134-20).

use crate::page_modality::{
    classify_document_majority, classify_page_heuristic, PageClassResult, PageClassification,
    PageModality,
};
use crate::page_signals::PageSignals;

/// Print vs manuscript page lists for grouped Vision convert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageConvertPlan {
    /// 1-indexed print pages (Acc path).
    pub print_pages: Vec<usize>,
    /// 1-indexed manuscript-like pages (MS profile).
    pub manuscript_pages: Vec<usize>,
    /// Document-majority modality (metadata / verify gating).
    pub document_modality: PageModality,
}

impl PageConvertPlan {
    pub fn all_print(total_pages: usize) -> Self {
        let n = total_pages.max(1);
        Self {
            print_pages: (1..=n).collect(),
            manuscript_pages: Vec::new(),
            document_modality: PageModality::Print,
        }
    }

    /// Build a plan from sampled classifications. Pages missing from the sample
    /// inherit [`document_modality`](Self::document_modality) (majority vote).
    pub fn from_classifications(pages: &[PageClassification], total_pages: usize) -> Self {
        if pages.is_empty() {
            return Self::all_print(total_pages);
        }
        let results: Vec<PageClassResult> = pages.iter().map(|p| p.result).collect();
        let document_modality = classify_document_majority(&results);
        let mut by_num: std::collections::HashMap<usize, PageModality> =
            pages.iter().map(|p| (p.page_num, p.modality())).collect();
        let n = total_pages
            .max(pages.iter().map(|p| p.page_num).max().unwrap_or(0))
            .max(1);
        let mut print_pages = Vec::new();
        let mut manuscript_pages = Vec::new();
        for page_num in 1..=n {
            let modality = by_num.remove(&page_num).unwrap_or(document_modality);
            if modality.is_manuscript_like() {
                manuscript_pages.push(page_num);
            } else {
                print_pages.push(page_num);
            }
        }
        Self {
            print_pages,
            manuscript_pages,
            document_modality,
        }
    }

    pub fn from_signals(
        signals: &[PageSignals],
        orientation_mixed: bool,
        total_pages: usize,
    ) -> Self {
        let pages = classifications_from_signals(signals, orientation_mixed);
        Self::from_classifications(&pages, total_pages)
    }

    /// Convert groups: `(modality, optional 1-indexed page set)`.
    /// `None` page set means convert the whole document (homogeneous).
    pub fn groups(&self) -> Vec<(PageModality, Option<Vec<usize>>)> {
        match (
            self.print_pages.is_empty(),
            self.manuscript_pages.is_empty(),
        ) {
            (false, true) => vec![(PageModality::Print, None)],
            (true, false) => vec![(PageModality::Manuscript, None)],
            (false, false) => vec![
                (PageModality::Print, Some(self.print_pages.clone())),
                (
                    PageModality::Manuscript,
                    Some(self.manuscript_pages.clone()),
                ),
            ],
            (true, true) => vec![(PageModality::Print, None)],
        }
    }

    pub fn is_split(&self) -> bool {
        !self.print_pages.is_empty() && !self.manuscript_pages.is_empty()
    }

    /// LAW-134-12: Auto EdgeParse is forbidden when any page is manuscript-like
    /// and the operator/profile skip flag is on.
    pub fn should_skip_edgeparse(&self, skip_flag: bool) -> bool {
        skip_flag && !self.manuscript_pages.is_empty()
    }
}

pub fn classifications_from_signals(
    signals: &[PageSignals],
    orientation_mixed: bool,
) -> Vec<PageClassification> {
    signals
        .iter()
        .map(|s| PageClassification {
            page_num: s.page_num,
            result: classify_page_heuristic(
                s.image_area_frac,
                s.glyph_text_density,
                s.ink_frac,
                orientation_mixed,
            ),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cls(page: usize, modality: PageModality) -> PageClassification {
        PageClassification {
            page_num: page,
            result: PageClassResult {
                modality,
                score: 0.9,
            },
        }
    }

    #[test]
    fn homogeneous_manuscript_converts_all_pages_once() {
        let plan = PageConvertPlan::from_classifications(
            &[
                cls(1, PageModality::Manuscript),
                cls(2, PageModality::Manuscript),
            ],
            2,
        );
        assert!(!plan.is_split());
        assert_eq!(plan.groups(), vec![(PageModality::Manuscript, None)]);
        assert!(plan.should_skip_edgeparse(true));
    }

    #[test]
    fn mixed_splits_print_and_manuscript_sets() {
        // 1/3 MS → document Mixed; page 1 still converts on the MS group.
        let plan = PageConvertPlan::from_classifications(
            &[
                cls(1, PageModality::Manuscript),
                cls(2, PageModality::Print),
                cls(3, PageModality::Print),
            ],
            3,
        );
        assert!(plan.is_split());
        assert_eq!(plan.document_modality, PageModality::Mixed);
        assert_eq!(plan.print_pages, vec![2, 3]);
        assert_eq!(plan.manuscript_pages, vec![1]);
        let groups = plan.groups();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], (PageModality::Print, Some(vec![2, 3])));
        assert_eq!(groups[1], (PageModality::Manuscript, Some(vec![1])));
    }

    #[test]
    fn unsampled_pages_inherit_majority() {
        let plan = PageConvertPlan::from_classifications(
            &[cls(1, PageModality::Print), cls(4, PageModality::Print)],
            4,
        );
        assert_eq!(plan.print_pages, vec![1, 2, 3, 4]);
        assert!(plan.manuscript_pages.is_empty());
    }

    #[test]
    fn empty_classifications_are_all_print() {
        let plan = PageConvertPlan::from_classifications(&[], 3);
        assert_eq!(plan.document_modality, PageModality::Print);
        assert_eq!(plan.print_pages, vec![1, 2, 3]);
        assert!(!plan.should_skip_edgeparse(true));
    }
}
