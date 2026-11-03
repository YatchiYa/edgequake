import { describe, expect, it } from "vitest";
import { searchProductDocs, tokenize } from "../product-docs";

const topPaths = (q: string) => searchProductDocs(q, 4).map((s) => s.path);

describe("copilot product docs search", () => {
  it("folds case and accents so French queries match", () => {
    expect(tokenize("Requête élégante, GRAPHE !")).toEqual(["requete", "elegante", "graphe"]);
  });

  it("answers the graph storage questions from the graph-at-scale page", () => {
    expect(topPaths("neo4j postgres graph storage limits scale")).toContain("docs/concepts/graph-storage-at-scale-fr.md");
    expect(topPaths("Cypher Apache AGE removed")).toContain("docs/concepts/graph-storage-at-scale-fr.md");
    expect(topPaths("1 million noeuds échelle")).toContain("docs/concepts/graph-storage-at-scale-fr.md");
  });

  it("finds operational topics and ignores unknown words", () => {
    expect(topPaths("upgrade migrate schema").some((p) => p.startsWith("docs/operations/"))).toBe(true);
    expect(searchProductDocs("zzqxv")).toEqual([]);
  });

  it("never ships internal or client documentation", () => {
    const all = searchProductDocs("altaroc client documentation interne", 50).map((s) => s.path);
    expect(all.some((p) => /documentation_interne|specs\//.test(p))).toBe(false);
  });
});
