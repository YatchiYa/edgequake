# 00 — Five WHYs

## Problem statement

Users cannot trust Query Mode answer links: page numbers and document names in
the reply may be invented by the LLM, and clicking does not reliably open the
correct PDF page.

---

### WHY 1 — Why can't users trust answer links / page numbers?

**Answer:** The answer markdown channel is not bound to retrieval locators.
Structured `sources[]` cards can deeplink correctly, but inline `[N]`, invented
"page 47", and markdown links in the answer are unverified display text.

---

### WHY 2 — Why is answer markdown unbound?

**Answer:** The UI only deeplinks from the `SourceCitations` panel. Inline
`[N]` is never parsed into a Next.js navigation. `StreamingMarkdownRenderer`
treats generic links as `target=_blank` and never receives `onCitationClick`.

---

### WHY 3 — Why does the pipeline treat citations as LLM style?

**Answer:** Prompt SSOT (`grounding.rs`) asks the model to emit `[N]` matching
chunk headers. There is no typed `CitationCatalog` and no post-generation
verifier. Document title is absent from chunk headers (or is a UUID behind a
flag). Locators live only on the API `sources[]` side-channel.

---

### WHY 4 — Why was post-gen verification missing?

**Answer:** 2026 literature (DeepCitation, LLM Best Practices RAG citations,
Azure RAG prompt engineering, TREC RAG) treats verification as non-optional,
but EdgeQuake deferred it. Acc gold (SPEC-082) treated `[N]` as a **scoring
contaminant** to strip, which froze product citation work rather than defining
a verified product contract.

---

### WHY 5 — Why did SPEC-033 / L-B1 not close this?

**Answer:** SPEC-033 grouped the citations **panel** and fixed controlled PDF
navigation. SPEC-047 L-B1 (entity→chunk→page in **answer** sources / inline)
remained deferred. The gap is specifically **answer-inline verified links**,
not page storage (already on chunks).

---

## Root cause (one line)

```text
LLM cites a free-form handle; system never attaches storage locators to the
answer text; UI never turns that text into a verified deeplink.
```

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
