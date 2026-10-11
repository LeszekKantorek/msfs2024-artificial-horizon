---
description: Preserve responsive PFD geometry and use the full arrangement fixture when adding the next instrument.
status: active
---

* Use [PFD desktop checks](../docs/testing.md#responsive-pfd-layout-and-desktop-checks) before changing geometry or adding an instrument.
* Validate all 17 intended elements through the development-only fixture before narrowing attitude space.
* The accepted minimum is 320 CSS px wide and 240 CSS px of usable height after browser bars and safe areas.
* Preserve the sample deadline during resize and redraw through the latest-sample frame scheduler.
* Test painted symbol extents, including stroke clearance, rather than only instrument-region bounds.
* Exclude hidden SVG labels and intersect visible bounds with their ancestor clips when checking content collisions.
* After changing a browser viewport, wait for SVG `viewBox` dimensions to match its CSS bounds before measuring glyphs. A fresh sample alone does not confirm completed resize geometry.
* For #37 and later, distinguish overlapping background coverage from non-overlapping indication content using the [module plan](pfd-module-plan.md).
* Keep desktop Edge evidence separate from required real iOS Safari and Android Chrome acceptance.

Sources: accepted #20 implementation plan, 2026-10-09; bank-zero clipping reproduced in 568x240 desktop viewport and covered by `tests/layout.test.mjs`.

Source: #66 browser check correction, 2026-10-11; retained hidden heading labels had nonzero bounds outside their painted clip.

Source: accepted #26 scope amendment, 2026-10-11. Scope excludes ground-track item 19. The full fixture contains 17 elements and uses production heading SVG.
