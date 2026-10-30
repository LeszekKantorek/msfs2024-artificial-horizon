# Independently available airspeed data

Status: Accepted through the supplied implementation plan.

## Context

Requiring a complete speed profile hides valid references and ranges when standard SimConnect data supplies only part of the profile.
Dynamic redline also differs from a fixed VNE.

## Decision

* Make thresholds and named references independently optional within the existing profile.
* Validate each number independently and validate ordering only for each rendered interval.
* Add optional redline separately from VNE.
* Each warning and marker requires only its own inputs.
* Preserve the existing endpoint, schema version, source lifecycle, and application composition.

## Alternatives

| Alternative | Assessment |
| --- | --- |
| Keep the complete profile | Suppresses valid data |
| Fill missing values from Demo or similar aircraft | Misrepresents the source |
| Add aircraft-specific configuration readers | Requires a separate integration beyond standard SimVars |

## Consequences

* Consumers must replace all profile members on each snapshot and remove lost indications independently.
* Full Demo profiles preserve existing presentation.
* Standard SimVars may still leave individual references unavailable.
* Project minimum rotation is not an FMS-calculated takeoff Vr.

> This decision does not establish simulator or device acceptance.

| Contract | Reference |
| --- | --- |
| Telemetry fields and validation | [Wire contract](../telemetry-contract.md#optional-airspeed-ranges-and-trend) |
| Instrument presentation | [Speed presentation](../efi/airspeed-indications.md#presentation) |
