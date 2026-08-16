/** Filename / stem helpers for SPEC-128 overlay ↔ markdown click-through. */

export function layoutAssetBasename(assetPath: string): string {
  const trimmed = assetPath.replace(/\\/g, '/');
  const noQuery = trimmed.split('?')[0] ?? trimmed;
  const parts = noQuery.split('/');
  return parts[parts.length - 1] || noQuery;
}

/** Stem without image extension so overlay `assets/foo.png` matches rewritten `/assets/foo`. */
export function layoutAssetStem(assetPath: string): string {
  return layoutAssetBasename(assetPath).replace(/\.(png|jpe?g|webp|gif)$/i, '');
}
