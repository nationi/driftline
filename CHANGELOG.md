# Changelog

Notable changes to Driftline are recorded here.

## 1.1.5 — 2026-10-24

- Let tiled drag hit-testing pass through the moving window's stale layout
  position, so overlapping drop targets remain reachable. Edge scrolling keeps
  working while the pointer is over a valid drop target.
- Restyle tiled drag, resize and mode cues as compact rounded badges, and make
  the drop-target outline thinner.

## 1.1.4 — 2026-10-24

- Relaunch eligible apps after login and restore their saved canvas positions
  and tiling slots when `restore_windows = true`.
- Keep a clipped neighboring tile reachable during hover focus and refine tiled
  resize and transition animations.

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
