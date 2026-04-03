/**
 * @module MermaidBlock sanitizeMermaidCode tests
 * @description Unit tests for the Mermaid sanitiser — issue #141.
 *
 * These tests are pure string-transformation tests and do NOT require a
 * browser or the mermaid library itself.  They only import the exported
 * `sanitizeMermaidCode` helper.
 */
import { describe, expect, it } from 'vitest';
import { sanitizeMermaidCode } from '../MermaidBlock';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function sanitized(code: string): string {
  return sanitizeMermaidCode(code).sanitized;
}

function hasIssue(code: string, fragment: string): boolean {
  return sanitizeMermaidCode(code).issues.some((i) => i.includes(fragment));
}

// ---------------------------------------------------------------------------
// Regression guard: already-valid diagrams must not be modified
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – no-op on valid diagrams', () => {
  it('leaves a plain simple flowchart untouched', () => {
    const code = 'graph TD\n  A --> B';
    expect(sanitized(code)).toBe(code);
  });

  it('leaves already-quoted labels untouched', () => {
    const code = 'graph TD\n  A["Hello World"] --> B';
    expect(sanitized(code)).toBe(code);
  });

  it('leaves a sequence diagram untouched', () => {
    const code = 'sequenceDiagram\n  Alice->>Bob: Hello';
    expect(sanitized(code)).toBe(code);
  });

  // First-principles contract: sanitizeMermaidCode must not reject diagram
  // types it doesn't recognise — that gate belongs to mermaid.parse(), not here.
  it('does not modify a diagram starting with an unrecognised keyword (future types)', () => {
    // e.g. a hypothetical new diagram type the sanitiser was never told about
    const code = 'packet-beta\n  0-7: "Source Port"\n  8-15: "Destination Port"';
    expect(sanitized(code)).toBe(code);
  });

  it('does not modify a C4Context diagram (keyword not in old heuristic list)', () => {
    const code = 'C4Context\n  Person(personAlias, "Actor", "Description")';
    expect(sanitized(code)).toBe(code);
  });
});

// ---------------------------------------------------------------------------
// Existing behaviour: parentheses / CJK inside [] labels
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – existing square-bracket special chars', () => {
  it('quotes labels with parentheses inside []', () => {
    const code = 'graph TD\n  A[action (note)] --> B';
    const result = sanitized(code);
    expect(result).toContain('A["action (note)"]');
  });

  it('quotes labels with CJK characters inside []', () => {
    const code = 'graph TD\n  A[动作模型] --> B';
    const result = sanitized(code);
    expect(result).toContain('["动作模型"]');
  });
});

// ---------------------------------------------------------------------------
// NEW: forward slashes inside [] labels (issue #141)
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – forward slash in [] labels', () => {
  it('quotes a label containing a forward slash', () => {
    const code = 'graph TD\n  A[yes/no] --> B';
    const result = sanitized(code);
    expect(result).toContain('A["yes/no"]');
    expect(result).not.toContain('A[yes/no]');
  });

  it('quotes a label containing multiple slashes', () => {
    const code = 'graph TD\n  A[input/output/error] --> B';
    const result = sanitized(code);
    expect(result).toContain('A["input/output/error"]');
  });

  it('reports an issue for slash-containing label', () => {
    const code = 'graph TD\n  A[yes/no] --> B';
    expect(hasIssue(code, 'Quoted label')).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// NEW: backslash inside [] labels (issue #141)
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – backslash in [] labels', () => {
  it('quotes a label containing a backslash', () => {
    const code = 'graph TD\n  A[C:\\path\\file] --> B';
    const result = sanitized(code);
    expect(result).toContain('["C:\\path\\file"]');
  });
});

// ---------------------------------------------------------------------------
// NEW: pipe | inside [] labels (already in char class, regression guard)
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – pipe in [] labels', () => {
  it('quotes a label containing a pipe character', () => {
    const code = 'graph TD\n  A[left|right] --> B';
    const result = sanitized(code);
    expect(result).toContain('A["left|right"]');
  });
});

// ---------------------------------------------------------------------------
// NEW: angle brackets inside [] labels
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – angle brackets in [] labels', () => {
  it('quotes a label containing angle brackets', () => {
    const code = 'graph TD\n  A[a<b>c] --> B';
    const result = sanitized(code);
    expect(result).toContain('A["a<b>c"]');
  });
});

// ---------------------------------------------------------------------------
// NEW: rhombus NodeId{label} with special chars (issue #141)
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – rhombus node special char labels', () => {
  it('quotes a rhombus label containing a forward slash', () => {
    const code = 'graph TD\n  A --> B{yes/no}';
    const result = sanitized(code);
    expect(result).toContain('B{"yes/no"}');
    expect(result).not.toContain('B{yes/no}');
  });

  it('quotes a rhombus label containing a pipe', () => {
    const code = 'graph TD\n  A --> B{left|right}';
    const result = sanitized(code);
    expect(result).toContain('B{"left|right"}');
  });

  it('leaves a simple rhombus label (no special chars) untouched', () => {
    const code = 'graph TD\n  A --> B{decision}';
    const result = sanitized(code);
    expect(result).toBe(code);
  });

  it('reports an issue for rhombus with slash', () => {
    const code = 'graph TD\n  A --> B{yes/no}';
    expect(hasIssue(code, 'Quoted rhombus label')).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// NEW: bare {label} without node ID — the core crash case (issue #141)
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – bare curly-brace nodes (DIAMOND_START crash)', () => {
  it('converts `A --> {Personnes/Gens}` to a valid quoted node', () => {
    const code = 'graph TD\n  People --> {Personnes/Gens}';
    const result = sanitized(code);
    // Should NOT contain the bare `{Personnes/Gens}` any more
    expect(result).not.toContain('{Personnes/Gens}');
    // Should contain a generated node id with the label quoted
    expect(result).toMatch(/_bare_\d+\["Personnes\/Gens"\]/);
  });

  it('converts a simple bare {Yes} node to a valid quoted node', () => {
    const code = 'graph TD\n  A --> {Yes}';
    const result = sanitized(code);
    expect(result).not.toContain('{Yes}');
    expect(result).toMatch(/_bare_\d+\["Yes"\]/);
  });

  it('assigns different IDs when multiple bare nodes appear', () => {
    const code = 'graph TD\n  A --> {foo}\n  B --> {bar}';
    const result = sanitized(code);
    // Both should have been replaced with unique IDs
    expect(result).not.toContain('{foo}');
    expect(result).not.toContain('{bar}');
    const matches = result.match(/_bare_(\d+)/g) ?? [];
    const ids = new Set(matches);
    expect(ids.size).toBe(2);
  });

  it('reports an issue for each bare curly node fixed', () => {
    const code = 'graph TD\n  People --> {Personnes/Gens}';
    const issues = sanitizeMermaidCode(code).issues;
    expect(issues.some((i) => i.includes('bare curly-brace'))).toBe(true);
  });

  it('does NOT treat a valid rhombus `B{label}` as bare', () => {
    const code = 'graph TD\n  A --> B{decision}';
    const result = sanitized(code);
    // B{decision} is valid — must not be renamed to _bare_X
    expect(result).not.toMatch(/_bare_\d+/);
    expect(result).toContain('B{decision}');
  });
});

// ---------------------------------------------------------------------------
// Code-block marker stripping
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – code block stripping', () => {
  it('removes markdown fenced code block markers', () => {
    const code = '```mermaid\ngraph TD\n  A --> B\n```';
    const result = sanitized(code);
    expect(result).not.toContain('```');
    expect(result).toContain('graph TD');
  });

  it('reports a stripping issue', () => {
    const code = '```mermaid\ngraph TD\n  A --> B\n```';
    expect(hasIssue(code, 'Removed code block')).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Completeness check
// ---------------------------------------------------------------------------

describe('sanitizeMermaidCode – completeness detection', () => {
  it('flags a one-line diagram as incomplete', () => {
    const code = 'graph TD';
    const issues = sanitizeMermaidCode(code).issues;
    expect(issues.some((i) => i.includes('incomplete'))).toBe(true);
  });
});
