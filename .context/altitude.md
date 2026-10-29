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

### G1000 STD source limitation

During one recorded [#89](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/89) session, G1000 STD was switched while `KOHLSMAN SETTING STD:1` remained false. Microsoft's [G1000 altimeter source](https://github.com/microsoft/msfs-avionics-mirror/blob/main/src/workingtitle-instruments-g1000/html_ui/PFD/Components/FlightInstruments/Altimeter.tsx) handles `std_baro_switch` as an avionics event and maintains its own standard-mode state. This observation does not establish the installed avionics version or prove that all aircraft behave the same way. Pressure near 29.92 inHg alone does not establish STD mode.

The accepted [BARO presentation](../docs/efi/altitude.md#presentation) uses a local inHg, hPa, and STD cycle. Local STD does not report or change cockpit mode. Reconsider simulator synchronization only with evidence for the selected aircraft and an accessible source of its mode.

For #89, controlled browser checks covered touch emulation and the text-selection style. BARO switching and absence of text selection were separately confirmed on Android. The confirmation does not establish behavior on other devices. Apply the shared [evidence limits](../docs/testing/evidence.md#select-evidence-by-failure-risk) when assessing desktop and actual-phone results.

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
