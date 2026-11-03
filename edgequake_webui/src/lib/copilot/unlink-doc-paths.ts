/**
 * Doc citations like `[docs/faq.md](docs/faq.md)` would resolve to dead pages of this app.
 * Keep web links (`https:`, `mailto:`) and in-app routes (`/graph`, `#x`); any other link
 * becomes its plain text.
 */
export function unlinkDocPaths(markdown: string): string {
  return markdown.replace(/\[([^\]\n]+)\]\(((?![a-z][a-z0-9+.-]*:|\/|#)[^)\s]+)\)/gi, "$1");
}
