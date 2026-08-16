# 12 — Coordinate Systems

LAW-128-4: **PDF user space is SSOT**. One transform in Rust produces `bbox_norm`. The frontend positions with CSS percentages of the **measured** page box.

## Three spaces

```ascii
  PDF user space (SSOT)
    origin: bottom-left of MediaBox (or CropBox if that is displayed)
    units:  points (1/72 in)
    y:      up
    stored: bbox_pdf { x0, y0, x1, y1 }   // min/max, not ordered pair guess
    + page width_pt, height_pt, rotation ∈ {0,90,180,270}

  Raster (L2 input)
    origin: top-left of rendered PNG
    units:  pixels
    y:      down
    size:   raster_width_px × raster_height_px  (vision DPI / layout render)

  Overlay CSS (FE)
    origin: top-left of measured react-pdf page box
    units:  CSS px (or %)
    y:      down
    size:   onRenderSuccess width/height — NOT scale * pageWidthPt alone
```

## PDF → bbox_norm (unrotated page)

Let `W, H` = `width_pt`, `height_pt`. Box `x0,y0,x1,y1` in PDF space (y-up):

```
  nx = min(x0,x1) / W
  ny = 1 - max(y0,y1) / H          // flip y
  nw = abs(x1-x0) / W
  nh = abs(y1-y0) / H
```

Clamp to `[0,1]`. This is `bbox_norm { x: nx, y: ny, w: nw, h: nh }`.

## Rotation

pdf.js applies page rotation in the viewport. Persist ISO `/Rotate`. Transform PDF coords into **unrotated MediaBox** before the flip above, **or** persist boxes already in the same space pdf.js uses for the rendered page. Pick one and lock it in G-layout-coord fixtures for 0/90/180/270.

Recommended: store boxes in **unrotated user space**; API applies rotation when emitting `bbox_norm` to match pdf.js default page render. Golden tests per rotation.

## CropBox

If viewer shows CropBox, `width_pt`/`height_pt` on `document_pages` must be the **displayed** box, and `bbox_pdf` must live in that space (subtract CropBox origin). Store `cropbox_pdf` when it differs from MediaBox.

## Raster → PDF (L2)

```ascii
  model xyxy (input tensor space)
    → undo letterbox/stretch (MUST match preprocess)
    → raster pixel xyxy (top-left)
    → x_pt = x_px * (W / raster_w)
       y_pt_up = H - y_px_bottom * (H / raster_h)
```

Ink residual in `chart_crop.rs` is raster-space — convert with the **same** page PNG dimensions used for that crop, or do not persist ink boxes as PDF-space without conversion.

## Frontend (no second transform)

```ascii
  overlay style:
    left:   bbox_norm.x * 100%
    top:    bbox_norm.y * 100%
    width:  bbox_norm.w * 100%
    height: bbox_norm.h * 100%
  parent:  wrapper sized to canvas CSS box from onRenderSuccess
```

Do **not** use `scale` and optional `width` to invent CSS pixels. Width-fit changes the mapping.

## Test

G-layout-coord: known PDF rect → persist → GET layout → `bbox_norm` within 1e-3 of formula. Playwright: CSS box IoU ≥ 0.8 vs fixture at scale 1.0 and 1.5.

## Cross-refs

- Front: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Vision: [05-lenses/007-vision-expert.md](05-lenses/007-vision-expert.md)
- DB: store pdf only — [05-lenses/003-database.md](05-lenses/003-database.md)
