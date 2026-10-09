---
description: Preserve responsive PFD geometry and use the full arrangement fixture when adding the next instrument.
status: active
---

* Use [PFD desktop checks](../docs/testing.md#responsive-pfd-layout-and-desktop-checks) before changing geometry or adding an instrument.
* Validate all 18 intended elements through the development-only fixture before narrowing attitude space.
* The accepted minimum is 320 CSS px wide and 240 CSS px of usable height after browser bars and safe areas.
* Preserve the sample deadline during resize and redraw through the latest-sample frame scheduler.
* Test painted symbol extents, including stroke clearance, rather than only instrument-region bounds.
* Keep desktop Edge evidence separate from required real iOS Safari and Android Chrome acceptance.

Sources: accepted #20 implementation plan, 2026-10-09; bank-zero clipping reproduced in 568x240 desktop viewport and covered by `tests/layout.test.mjs`.
