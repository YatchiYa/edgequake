import { describe, expect, it } from 'bun:test';
import {
  DEFAULT_VISION_EXTRACT_DRAFT,
  shouldShowVisionExtractControls,
  summarizeVisionExtract,
} from '@/components/settings/vision-extract-controls';

describe('SPEC-015V vision extract controls', () => {
  it('defaults all modalities on', () => {
    expect(DEFAULT_VISION_EXTRACT_DRAFT.extractImages).toBe(true);
    expect(DEFAULT_VISION_EXTRACT_DRAFT.extractCharts).toBe(true);
    expect(DEFAULT_VISION_EXTRACT_DRAFT.extractFigures).toBe(true);
  });

  it('hides for EdgeParse and shows for Vision / default→Vision', () => {
    expect(shouldShowVisionExtractControls('edgeparse', true)).toBe(false);
    expect(shouldShowVisionExtractControls('vision', false)).toBe(true);
    expect(shouldShowVisionExtractControls('default', true)).toBe(true);
    expect(shouldShowVisionExtractControls('none', true)).toBe(true);
    expect(shouldShowVisionExtractControls('default', false)).toBe(false);
  });

  it('summarizes non-default extract for panel scent', () => {
    expect(summarizeVisionExtract(DEFAULT_VISION_EXTRACT_DRAFT).isDefault).toBe(
      true,
    );
    const custom = {
      ...DEFAULT_VISION_EXTRACT_DRAFT,
      extractCharts: false,
      chartSystemPrompt: 'axis labels',
    };
    const s = summarizeVisionExtract(custom);
    expect(s.isDefault).toBe(false);
    expect(s.modalityOff).toEqual(['Charts']);
    expect(s.promptOverrides).toBe(1);
  });
});
