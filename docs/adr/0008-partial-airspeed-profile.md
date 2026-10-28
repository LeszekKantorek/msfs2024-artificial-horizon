# Independently available airspeed data

Date: 2026-10-28
Status: Accepted through the supplied implementation plan.

## Context

A complete speed profile requirement prevents valid references and ranges from appearing when standard SimConnect data supplies only part of the profile. Dynamic redline also differs from a fixed VNE.

## Decision

Make thresholds and named references independently optional within the existing profile. Validate each number independently and validate ordering only for each rendered interval. Add optional redline separately from VNE. Each warning and marker requires only its own inputs. Preserve the existing endpoint, schema version, source lifecycle and application composition.

## Alternatives

Keeping the complete profile suppresses valid data. Filling missing values from Demo or similar aircraft misrepresents the source. Aircraft-specific configuration readers require a separate integration beyond standard SimVars.

## Consequences

Consumers must replace all profile members on each snapshot and remove lost indications independently. Full Demo profiles preserve existing presentation. Standard SimVars may still leave individual references unavailable. Project minimum rotation is not an FMS-calculated takeoff Vr. This decision does not establish simulator or device acceptance.

See the [wire contract](../telemetry-contract.md#optional-airspeed-ranges-and-trend) and [speed presentation](../efi/airspeed-indications.md#presentation).
