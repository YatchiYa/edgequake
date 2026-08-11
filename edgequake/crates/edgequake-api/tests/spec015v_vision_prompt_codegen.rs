//! SPEC-015V — codegen + drift check for FE Vision system-prompt mirror.
//!
//! Refresh:
//! ```bash
//! cargo test -p edgequake-api spec015v_write_vision_prompt_codegen --test spec015v_vision_prompt_codegen -- --ignored --nocapture
//! ```
//! Or: `make codegen-vision-prompts`

use edgequake_api::services::multimodal::{
    CHART_ANALYSIS_SYSTEM_PROMPT, FIGURE_ANALYSIS_SYSTEM_PROMPT, IMAGE_ANALYSIS_SYSTEM_PROMPT,
};
use edgequake_pdf::RAG_PAGE_VISION_SYSTEM_PROMPT;
use std::path::PathBuf;

fn webui_prompts_ts() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../edgequake_webui/src/lib/vision/default-system-prompts.ts")
}

fn render_ts() -> String {
    format!(
        r#"/**
 * SPEC-015V — Built-in Vision system prompts (UI SSOT mirror).
 *
 * MUST stay byte-aligned with Rust (generated — do not hand-edit prompt bodies):
 * - Page → `edgequake_pdf::RAG_PAGE_VISION_SYSTEM_PROMPT`
 * - Image/Chart/Figure → `edgequake_api::services::multimodal::*_ANALYSIS_SYSTEM_PROMPT`
 *
 * Regenerate: `make codegen-vision-prompts`
 */

export const DEFAULT_VISION_PAGE_SYSTEM_PROMPT = {page};

export const DEFAULT_VISION_IMAGE_SYSTEM_PROMPT = {image};

export const DEFAULT_VISION_CHART_SYSTEM_PROMPT = {chart};

export const DEFAULT_VISION_FIGURE_SYSTEM_PROMPT = {figure};

export type VisionPromptFieldKey =
  | 'pageSystemPrompt'
  | 'imageSystemPrompt'
  | 'chartSystemPrompt'
  | 'figureSystemPrompt';

export const DEFAULT_VISION_SYSTEM_PROMPTS: Record<
  VisionPromptFieldKey,
  string
> = {{
  pageSystemPrompt: DEFAULT_VISION_PAGE_SYSTEM_PROMPT,
  imageSystemPrompt: DEFAULT_VISION_IMAGE_SYSTEM_PROMPT,
  chartSystemPrompt: DEFAULT_VISION_CHART_SYSTEM_PROMPT,
  figureSystemPrompt: DEFAULT_VISION_FIGURE_SYSTEM_PROMPT,
}};

/** Map stored override → what the textarea shows (empty → built-in). */
export function displayVisionSystemPrompt(
  key: VisionPromptFieldKey,
  stored: string,
): string {{
  const trimmed = stored.trim();
  return trimmed.length > 0 ? stored : DEFAULT_VISION_SYSTEM_PROMPTS[key];
}}

/**
 * Map textarea edit → stored override.
 * Empty or byte-equal to built-in → '' so future SSOT updates still apply.
 */
export function storeVisionSystemPrompt(
  key: VisionPromptFieldKey,
  edited: string,
): string {{
  const trimmed = edited.trim();
  if (!trimmed) return '';
  if (edited === DEFAULT_VISION_SYSTEM_PROMPTS[key]) return '';
  return edited;
}}

export function isCustomVisionSystemPrompt(
  key: VisionPromptFieldKey,
  stored: string,
): boolean {{
  return stored.trim().length > 0;
}}
"#,
        page = serde_json::to_string(RAG_PAGE_VISION_SYSTEM_PROMPT).unwrap(),
        image = serde_json::to_string(IMAGE_ANALYSIS_SYSTEM_PROMPT).unwrap(),
        chart = serde_json::to_string(CHART_ANALYSIS_SYSTEM_PROMPT).unwrap(),
        figure = serde_json::to_string(FIGURE_ANALYSIS_SYSTEM_PROMPT).unwrap(),
    )
}

#[test]
fn spec015v_fe_prompt_mirror_matches_rust_ssot() {
    let path = webui_prompts_ts();
    let existing =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let expected = render_ts();
    assert_eq!(
        existing, expected,
        "FE prompt mirror drifted from Rust SSOT. Run: make codegen-vision-prompts"
    );
}

#[test]
#[ignore = "manual codegen refresh for SPEC-015V prompt mirror"]
fn spec015v_write_vision_prompt_codegen() {
    let path = webui_prompts_ts();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir vision");
    }
    std::fs::write(&path, render_ts()).expect("write prompts ts");
    eprintln!("wrote {}", path.display());
}
