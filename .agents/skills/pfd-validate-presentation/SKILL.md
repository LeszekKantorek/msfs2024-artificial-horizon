---
name: pfd-validate-presentation
description: Validate PFD geometry, SVG layers, rendering lifecycle, or browser module integration. Use for presentation changes and visual regressions, not prose-only edits or certification of real-phone acceptance.
---

# Validate PFD Presentation

## Goal

Produce traceable presentation-validation results with unresolved failures and device-evidence limits stated explicitly.

## Steps

Use this procedure for geometry, layers, rendering lifecycle, and browser module integration changes.
Run commands from the repository root, using the Windows environment in [setup](../../../CONTRIBUTING.md#environment-setup).

1. Identify the revision, changed behavior, accepted invariants, and affected instruments. Use the [validation matrix](../../../docs/testing/index.md#validation-by-change-scope) to confirm scope. For prose-only changes, perform documentation checks and stop this presentation procedure.
2. Select applicable scenarios from the [feature procedures](../../../docs/testing/index.md#find-the-required-procedure) and [layout constraints](../../../docs/testing/pfd.md#geometry-and-paint-regression-checks). Define expected results for geometry, availability, expiry, and recovery where affected. Apply [evidence reuse](../../../docs/testing/index.md#evidence-reuse) before scheduling new runs.
3. For visual comparisons, preserve baseline and candidate inputs under [screenshot comparisons](../../../docs/testing/evidence.md#screenshot-comparisons). Before binary-dependent checks, establish [build provenance](../../../docs/testing/evidence.md#build-provenance). If provenance is uncertain, follow its fresh-build recovery. Keep source and artifact failures separate from rendering failures.
4. Run required [automated checks](../../../docs/testing/automated.md#automated-checks) and both [desktop scripts](../../../docs/testing/pfd.md#responsive-pfd-layout-and-desktop-checks) for the affected PFD scope, unless their recorded evidence remains reusable. Supply the explicit release executable to the runtime script. Apply [environment diagnosis](../../../.context/local-validation-environment.md) for launch or fixture failures, and the mandatory [SAC stop rule](../../../.context/windows-validation.md) if error 4551 occurs.
5. Inspect the relevant screenshots and painted bounds. For layer changes, apply [paint-order checks](../../../docs/testing/pfd.md#paint-order-checks), including test sensitivity and cleanup. Check visible content within clips after resize settles; geometry assertions alone do not prove readability.
6. Correct established defects within the authorized scope and repeat affected checks. Preserve ambiguous observations as unresolved; do not attribute pixel differences to browser variation without evidence. If a restriction blocks execution, retain completed evidence and identify the missing check.
7. When device behavior or feature acceptance requires it, apply the [real-phone procedure](../../../docs/testing/devices.md#real-phone-acceptance-procedure) and the feature's scenarios on actual iOS Safari and Android Chrome. If devices are unavailable, leave that acceptance unconfirmed. Desktop results cannot replace device evidence.
8. Record commands, native exit codes, revision and artifact identity, environment, screenshots, observations, and unresolved checks in the requested report. Use the owning issue or PR when publication is authorized. Distinguish fixture, compiled application, and real-device evidence. Finish when every selected check has a result or an explicit unresolved condition.
