---
description: Preserve responsive PFD geometry and use the full arrangement fixture when adding the next instrument.
status: active
---

* Use [pfd-validate-presentation](../.agents/skills/pfd-validate-presentation/SKILL.md) for the validation procedure before changing geometry or adding an instrument. This entry retains layout constraints and regression lessons.
* Validate all 17 intended elements through the development-only fixture before narrowing attitude space.
* The accepted minimum is 320 CSS px wide and 240 CSS px of usable height after browser bars and safe areas.
* Preserve the sample deadline during resize and redraw through the latest-sample frame scheduler.
* Test painted symbol extents, including stroke clearance, rather than only instrument-region bounds.
* Exclude hidden SVG labels and intersect visible bounds with their ancestor clips when checking content collisions.
* After changing a browser viewport, wait for SVG `viewBox` dimensions to match its CSS bounds before measuring glyphs. A fresh sample alone does not confirm completed resize geometry.
* For #37 and later, distinguish overlapping background coverage from non-overlapping indication content using the [module plan](pfd-module-plan.md).
* Keep desktop Edge evidence separate from required real iOS Safari and Android Chrome acceptance.

## Paint order checks

Apply these checks when changing SVG layers.

* Check points inside intersecting painted strokes or fills, including transforms and clipping.
* Use visible hit order at those points to check which object covers the other.
* Do not infer painted overlap solely from bounding boxes or render-call order.
* Temporarily reverse the relevant SVG node order to check whether the test detects the wrong layer.
* Apply [test sensitivity and cleanup rules](validation-and-review.md#select-evidence-by-failure-risk) when using temporary mutations.
* Preserve controlled inputs and baseline provenance under the [screenshot comparison procedure](../docs/testing.md#screenshot-comparisons).
* Include capture, pinned edges, both directions, signed values, missing data, expiry, and recovery when applicable.

The [altitude trend acceptance procedure](../docs/testing.md#altitude-trend-placement-acceptance) gives the concrete #63 scenarios.

## Sources

Sources: accepted #20 implementation plan, 2026-10-09; bank-zero clipping reproduced in 568x240 desktop viewport and covered by `tests/layout.test.mjs`.

Source: #66 browser check correction, 2026-10-11; retained hidden heading labels had nonzero bounds outside their painted clip.

Source: accepted #26 scope amendment, 2026-10-11. Scope excludes ground-track item 19. The full fixture contains 17 elements and uses production heading SVG.

Source: #63 painted-overlap and reversed-layer checks consolidated from validation guidance, 2026-10-11.
