/**
 * Detect HTML-only inline codespans that should render as real sub/sup
 * instead of literal tags inside `<code>` (e.g. `c<sub>i</sub>(W)`).
 *
 * SSOT for "what counts as an HTML codespan"; sanitization stays in sanitizeHtml.
 */
import { sanitizeHtml } from "./sanitize-html";

/** Tags allowed inside an HTML codespan candidate (shape check only). */
const ALLOWED_TAG_RE =
  /<\/?(?:sub|sup|i|em|b|strong|var)\b[^>]*>/gi;

/**
 * True when text looks like safe phrasing wrapping at least one sub/sup.
 * Does not sanitize — call tryHtmlCodespan for that.
 */
export function isHtmlCodespanCandidate(text: string): boolean {
  const trimmed = text.trim();
  if (!trimmed) return false;

  // Must include sub or sup (the bug we fix).
  if (!/<\/?(?:sub|sup)\b/i.test(trimmed)) return false;

  // Reject obvious dangerous tags before sanitize.
  if (/<\/?(?:script|iframe|object|embed|link|style|svg|math)\b/i.test(trimmed)) {
    return false;
  }
  if (/\son[a-z]+\s*=/i.test(trimmed)) {
    return false;
  }

  // After stripping allowed tags, no raw `<` may remain.
  const withoutAllowed = trimmed.replace(ALLOWED_TAG_RE, "");
  if (withoutAllowed.includes("<") || withoutAllowed.includes(">")) {
    return false;
  }

  return true;
}

/**
 * Returns sanitized HTML if codespan body is HTML-only safe phrasing with
 * sub/sup; otherwise null (caller keeps literal `<code>` rendering).
 *
 * Client-only: without DOMPurify we return null so SSR does not strip tags
 * into plain text and lose the chance to hydrate correctly.
 */
export function tryHtmlCodespan(text: string): string | null {
  if (!isHtmlCodespanCandidate(text)) {
    return null;
  }

  // sanitizeHtml SSR fallback strips all tags — skip unwrap until browser.
  if (typeof window === "undefined") {
    return null;
  }

  const sanitized = sanitizeHtml(text);
  if (!sanitized.trim()) {
    return null;
  }
  // Sanitizer must have preserved at least one sub/sup.
  if (!/<su[bp]\b/i.test(sanitized)) {
    return null;
  }

  return sanitized;
}
