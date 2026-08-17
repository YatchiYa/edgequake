import { describe, expect, it } from 'vitest';
import {
  formatServerDefaultExtractionLanguageLabel,
  getServerDefaultExtractionLanguage,
} from '../extraction-languages';

describe('extraction language server default labels', () => {
  it('defaults to English when env unset', () => {
    expect(getServerDefaultExtractionLanguage()).toBe('English');
  });

  it('never-silent label includes resolved language', () => {
    const label = formatServerDefaultExtractionLanguageLabel(
      (_key, defaultValue) => defaultValue,
      'English',
    );
    expect(label).toBe('Server default (English)');
    expect(label.toLowerCase()).not.toBe('server default');
  });
});
