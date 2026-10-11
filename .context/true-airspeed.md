---
description: Extend or validate optional TAS telemetry, the top-left TAS field, or its browser and device acceptance.
status: active
---

Use the [speed contract](../docs/telemetry-contract.md#optional-speed-indications), [TAS presentation](../docs/pfd-presentation.md#true-airspeed), and [TAS acceptance](../docs/testing.md#true-airspeed-acceptance).

* Use the existing top-left 32 CSS px allocation without moving heading or narrowing attitude.
* Keep the TAS frame white; GS retains its speed-trend magenta frame.
* TAS uses source-supplied data independently of IAS, GS, configuration, and acceleration.
* The 999 kt presentation limit applies before rounding; 999.01 kt is valid telemetry with an overflow indication.
* Exercise browser omission and malformed input after a valid TAS reading so an existing X cannot satisfy the transition check.
* Reconnect can finish after the demo's initial zero-speed segment; verify the recovered sample instead of assuming zero.

Source: accepted #66 implementation plan and formatting choices, 2026-10-11.
Source: release-runtime restart check on 2026-10-11 demonstrated a nonzero fresh TAS after reconnect.
Evidence limit: actual iOS Safari and Android Chrome acceptance remains separate from desktop checks.
