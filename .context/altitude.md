---
description: Select altitude, BARO, selected altitude, vertical speed, and trend sources.
status: active
---

## Altitude and BARO

Select the source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Fields, independent validity, pressure meaning, and demo sweeps | [Altitude telemetry](../docs/telemetry-contract.md#optional-altitude-and-pressure-setting) |
| Tape spacing, drums, display limits, BARO formatting, and availability | [Altitude presentation](../docs/efi/altitude.md#presentation) |
| Automated and device checks | [Altitude acceptance](../docs/efi/altitude.md#altitude-and-pressure-acceptance), [footer and BARO checks](../docs/efi/altitude.md#tape-footers-and-source-formatted-baro) |
| Public evidence and accepted scope | [Issue #23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23), [PR #59](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/pull/59), [footer revision #65](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/65) |
| Implementation and accepted tape/BARO refinements | [Implementation](../docs/efi/altitude.md#implementation), [rationale and regression lessons](../docs/efi/altitude.md#rationale-and-regression-lessons) |

## Altitude trend

| Need | Source |
| --- | --- |
| Requirements and presentation | [Feature contract](../docs/efi/altitude-trend.md#requirements) |
| Object ownership and calculations | [Implementation](../docs/efi/altitude-trend.md#implementation) |
| Automated, visual, and device acceptance | [Testing](../docs/efi/altitude-trend.md#testing) |
| Decision rationale, regression lessons, and evidence limits | [Rationale](../docs/efi/altitude-trend.md#rationale-and-regression-lessons) |

## Selected altitude

Select the source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Optional target, freshness, and shared altitude demo | [Selected-altitude telemetry](../docs/telemetry-contract.md#optional-selected-altitude) |
| Capture, bug geometry, thresholds, flashing, and deviation | [Selected-altitude presentation](../docs/efi/selected-altitude.md#presentation-and-alert-behavior) |
| Per-sample acceptance and timed rendering | [Panel lifecycle](../docs/efi/lifecycle.md#panel-interface-and-frame-lifecycle) |
| Automated and actual-device checks | [Selected-altitude acceptance](../docs/efi/selected-altitude.md#selected-altitude-and-alert-acceptance) |
| Alert behavior, regression lessons, and decisions | [Implementation](../docs/efi/selected-altitude.md#implementation), [rationale](../docs/efi/selected-altitude.md#rationale-and-regression-lessons) |

## Vertical speed

Select the source for the affected boundary:

| Boundary | Source |
| --- | --- |
| Signed ft/min, optional fields, freshness, and analytic scenario | [VSI telemetry](../docs/telemetry-contract.md#optional-vertical-speed) |
| Window geometry, styles, limits, rounding, and local availability | [VSI presentation](../docs/efi/vertical-speed.md#presentation) |
| Automated coverage and real-phone procedures | [VSI acceptance](../docs/efi/vertical-speed.md#vertical-speed-acceptance) |
| Prediction and overlap with the selected bug | [Altitude trend](#altitude-trend) |
| Object ownership, alignment, and scenario lessons | [Implementation](../docs/efi/vertical-speed.md#implementation), [rationale](../docs/efi/vertical-speed.md#rationale-and-regression-lessons) |
