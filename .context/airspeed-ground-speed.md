---
description: Extend or validate IAS/GS, ranges, references, trend, rolling digits, or the synthetic profile.
status: active
---

Use the canonical sources for the contract being extended:

| Contract | Source |
| --- | --- |
| Speed fields, independent validity, attitude-only compatibility | [Speed contract](../docs/telemetry-contract.md#optional-speed-indications) |
| Optional profile and source acceleration | [Range/trend contract](../docs/telemetry-contract.md#optional-airspeed-ranges-and-trend) |
| Tape, rolling digits, GS placement, unavailable/overflow states | [Speed presentation](../docs/pfd/airspeed.md#presentation) |
| Reference placement, fallback values, colors, prediction | [Ranges, references, and trend](../docs/pfd/airspeed-indications.md#presentation) |
| IAS/GS implementation and acceptance | [IAS and GS](../docs/pfd/airspeed.md#implementation), [tests](../docs/pfd/airspeed.md#testing) |
| Range/reference implementation and acceptance | [Speed indications](../docs/pfd/airspeed-indications.md#implementation), [tests](../docs/pfd/airspeed-indications.md#testing) |
| Accepted choices, glyph measurements, and evidence limits | [IAS/GS lessons](../docs/pfd/airspeed.md#rationale-and-regression-lessons), [range/trend lessons](../docs/pfd/airspeed-indications.md#rationale-and-regression-lessons) |
