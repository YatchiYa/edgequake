import { describe, expect, it } from "bun:test";
import {
  bindFigureImagesToPageAssets,
  getDocumentMmAssetUrl,
  isDurableMmAssetHref,
  rewriteMarkdownMmAssetUrls,
} from "@/lib/api/edgequake/documents";

describe("mm-asset markdown rewrite (MV-28)", () => {
  it("does not invent fig paths for figure headings without durable assets", () => {
    const md = `<!-- edgequake-page:1 -->
## Figure 1: Autodata pipeline

The framework employs an agent.`;
    const bound = bindFigureImagesToPageAssets(md);
    expect(bound).not.toContain("![Figure 1");
    expect(bound).not.toContain("assets/page-0001");
  });

  it("keeps existing fig crop hrefs", () => {
    const md = `<!-- edgequake-page:1 -->
![Figure 1: Autodata pipeline](assets/page-0001-fig-01.png)

The framework employs an agent.`;
    const bound = bindFigureImagesToPageAssets(md);
    expect(bound).toContain("![Figure 1: Autodata pipeline](assets/page-0001-fig-01.png)");
  });

  it("binds hallucinated figure href only when a durable fig/chart/table exists", () => {
    const md = `<!-- edgequake-page:1 -->
![Figure 1. Overview](media/fig1.png)
![x](assets/page-0001-fig-01.png)

Abstract`;
    const bound = bindFigureImagesToPageAssets(md);
    expect(bound).toContain("![Figure 1. Overview](assets/page-0001-fig-01.png)");
    expect(bound).not.toContain("media/fig1.png");
  });

  it("rewrites assets/ links to document asset-id API URL", () => {
    const md = `![Page 1](assets/page-0001-fig-01.png)`;
    const out = rewriteMarkdownMmAssetUrls(md, "doc-abc");
    expect(out).toContain("/documents/doc-abc/assets/page-0001-fig-01");
    expect(
      isDurableMmAssetHref(getDocumentMmAssetUrl("doc-abc", "assets/page-0001-fig-01.png")),
    ).toBe(true);
  });

  it("skips text-only pages (no full-page invent)", () => {
    const md = `<!-- edgequake-page:1 -->
# Learning the ARTS

15.3% relative improvement.`;
    const bound = bindFigureImagesToPageAssets(md);
    expect(bound).not.toContain("![");
    expect(bound).not.toContain("assets/page-0001.png");
  });

  it("rewrites table crop links to asset-id API URL", () => {
    const md = `![Table 1](assets/page-0006-table-01.png)`;
    const out = rewriteMarkdownMmAssetUrls(md, "doc-abc");
    expect(out).toContain("/documents/doc-abc/assets/page-0006-table-01");
  });

  it("does not invent table paths when none exist", () => {
    const md = `<!-- edgequake-page:6 -->
## Table 1: Pass rates

Discussion.`;
    const bound = bindFigureImagesToPageAssets(md);
    expect(bound).not.toContain("assets/page-0006");
  });

  it("keeps durable table crop when present", () => {
    const md = `<!-- edgequake-page:6 -->
![Table 1](assets/page-0006-table-01.png)

## Table 1: Pass rates`;
    const bound = bindFigureImagesToPageAssets(md);
    expect(bound).toContain("assets/page-0006-table-01.png");
  });

  it("strips drawing tags in rewrite", () => {
    const md = `<!-- edgequake-page:1 -->
![Figure 1](assets/page-0001-fig-01.png)
<drawing id="im-x" format="png" path="assets/page-0001-fig-01.png" />

Caption`;
    const out = rewriteMarkdownMmAssetUrls(md, "doc-abc");
    expect(out).not.toContain("<drawing");
    expect(out).toContain("/documents/doc-abc/assets/page-0001-fig-01");
  });
});
