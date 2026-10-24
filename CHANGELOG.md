# Changelog

Notable changes to Driftline are recorded here.

## 1.1.3 — 2026-10-24

- Save tiling columns, stacks, widths, selected windows, and output ownership
  with restorable session entries, then restore them when tiling is entered
  after a compositor restart. The compositor still starts in canvas mode.
- Keep Alt-Tab in the active tiling layout instead of switching back to canvas.

## 1.1.2 — 2026-10-24

- Keep scrolling tiling active through fullscreen exit and the client's restored
  buffer commit.

## 1.1.1 — 2026-10-24

- Keep scrolling tiling active after a fullscreen window acknowledges its
  fullscreen configure; the fullscreen output's layout resumes when fullscreen
  ends.
- Restore Super+left-drag horizontal scrolling in tiling mode.
