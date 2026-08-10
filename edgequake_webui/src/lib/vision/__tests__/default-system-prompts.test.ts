import { describe, expect, it } from 'bun:test';
import {
  DEFAULT_VISION_CHART_SYSTEM_PROMPT,
  DEFAULT_VISION_IMAGE_SYSTEM_PROMPT,
  DEFAULT_VISION_PAGE_SYSTEM_PROMPT,
  displayVisionSystemPrompt,
  storeVisionSystemPrompt,
} from '@/lib/vision/default-system-prompts';

describe('SPEC-015V default vision system prompts', () => {
  it('mirrors Rust page Pass A SSOT markers', () => {
    expect(DEFAULT_VISION_PAGE_SYSTEM_PROMPT).toContain('CHARTS / PLOTS');
    expect(DEFAULT_VISION_PAGE_SYSTEM_PROMPT).toContain('EVERY readable data point');
    expect(DEFAULT_VISION_PAGE_SYSTEM_PROMPT).toContain('**Key values:**');
  });

  it('mirrors Rust Pass B image/chart prompts', () => {
    expect(DEFAULT_VISION_IMAGE_SYSTEM_PROMPT).toContain('expert image analyzer');
    expect(DEFAULT_VISION_CHART_SYSTEM_PROMPT).toContain('key_values');
    expect(DEFAULT_VISION_CHART_SYSTEM_PROMPT).toContain('data_table_md');
  });

  it('shows built-in when stored empty and stores empty when edited equals default', () => {
    expect(displayVisionSystemPrompt('chartSystemPrompt', '')).toBe(
      DEFAULT_VISION_CHART_SYSTEM_PROMPT,
    );
    expect(
      storeVisionSystemPrompt('chartSystemPrompt', DEFAULT_VISION_CHART_SYSTEM_PROMPT),
    ).toBe('');
    expect(storeVisionSystemPrompt('chartSystemPrompt', 'custom axis rule')).toBe(
      'custom axis rule',
    );
  });
});
