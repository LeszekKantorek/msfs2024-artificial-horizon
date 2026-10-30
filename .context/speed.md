---
description: Select speed instrument contracts, implementation, rationale, and checks.
status: active
---

## IAS, GS, and speed indications

Use the canonical sources for the contract being extended:

| Contract | Source |
| --- | --- |
| Speed fields, independent validity, attitude-only compatibility | [Speed contract](../docs/telemetry-contract.md#optional-speed-indications) |
| Optional profile and source acceleration | [Range/trend contract](../docs/telemetry-contract.md#optional-airspeed-ranges-and-trend) |
| Tape, rolling digits, GS placement, unavailable/overflow states | [Speed presentation](../docs/efi/airspeed.md#presentation) |
| Reference placement, fallback values, colors, prediction | [Ranges, references, and trend](../docs/efi/airspeed-indications.md#presentation) |
| IAS/GS implementation and acceptance | [IAS and GS](../docs/efi/airspeed.md#implementation), [tests](../docs/efi/airspeed.md#testing) |
| Range/reference implementation and acceptance | [Speed indications](../docs/efi/airspeed-indications.md#implementation), [tests](../docs/efi/airspeed-indications.md#testing) |
| Accepted choices, glyph measurements, and evidence limits | [IAS/GS lessons](../docs/efi/airspeed.md#rationale-and-regression-lessons), [range/trend lessons](../docs/efi/airspeed-indications.md#rationale-and-regression-lessons) |

IAS history expires independently of IAS availability. The [SimConnect policy](../docs/sources/simconnect.md#optional-instrument-observations) uses a 0.5-second measurement horizon and a one-second IAS freshness deadline. At exactly 0.5 seconds, test receipt before expiry and expiry before receipt. Also test equal timestamps. Otherwise, expired history can seed a new trend. [Issue #87](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/87) records the implementation and acceptance evidence.

## True airspeed

| Need | Source |
| --- | --- |
| Requirements and presentation | [Feature contract](../docs/efi/true-airspeed.md#requirements) |
| Object ownership and calculations | [Implementation](../docs/efi/true-airspeed.md#implementation) |
| Automated, visual, and device acceptance | [Testing](../docs/efi/true-airspeed.md#testing) |
| Decision rationale, regression lessons, and evidence limits | [Rationale](../docs/efi/true-airspeed.md#rationale-and-regression-lessons) |

Partial-profile decision: [ADR 0008](../docs/adr/0008-partial-airspeed-profile.md) governs independent thresholds, references and current redline.
