---
description: Review changes against the project's network, data, dependency, and CI trust boundaries. Use when a change affects exposure, permissions, inputs, or executable content.
status: active
---

## Review procedure

Review the boundaries touched by the change, with depth proportional to its exposure.

1. Identify the exact revision, diff, affected assets, and entry points.
2. Read the accepted deployment, telemetry, repository, and CI contracts for those boundaries.
3. Identify who can supply inputs and what privileges consume them.
4. State relevant failure or abuse scenarios and their possible impact.
5. Compare the implementation with the required boundaries in the table below.
6. Inspect changed dependencies, installation behavior, permissions, and executable inputs where applicable.
7. Select focused checks that demonstrate preservation of the affected boundary.
8. Record findings with the affected file, trigger, impact, evidence, and proposed correction.
9. Check confirmed corrections and report remaining risks or unreviewed scope separately.

## Project boundaries

Apply these existing constraints rather than assuming broader protection.

| Area | Required boundary | Canonical source |
| --- | --- | --- |
| Local and LAN access | Loopback default, explicit LAN binding, private-network firewall scope, and no LAN authentication | [Deployment](../docs/architecture.md#deployment), [phone access](../README.md#use-your-phone) |
| Remote exposure | Public hosting and port forwarding are outside MVP scope, with remote access requiring a separate access-control/TLS decision | [Deployment](../docs/architecture.md#deployment) |
| Telemetry and resources | Validated snapshots, bounded latest-state delivery, freshness checks, and stalled-client deadlines | [Architecture](../docs/architecture.md#boundaries), [telemetry contract](../docs/telemetry-contract.md), [ADR 0002](../docs/adr/0002-sse-connection-lifecycle.md) |
| Displayed input | Preserve input validation and safe text rendering without changing the declared telemetry contract | [Browser responsibilities](../docs/architecture.md#boundaries), [telemetry contract](../docs/telemetry-contract.md) |
| Privileged acceptance CI | Execute trusted main code only, with limited permissions and no persisted checkout credentials | [CI trust boundary](../docs/pr-acceptance.md#trust-boundary) |
| Issue and PR content | Treat metadata as data and never interpolate it into shell commands | [CI trust boundary](../docs/pr-acceptance.md#trust-boundary) |
| Dependencies | Preserve locked dependencies and the acceptance tooling's installation without lifecycle scripts | [Repository rules](../CONTRIBUTING.md#change-requirements), [CI trust boundary](../docs/pr-acceptance.md#trust-boundary) |
| Secrets and distribution | Exclude credentials, local SDK paths, SDK binaries, and dependencies without distribution rights from commits | [Repository rules](../CONTRIBUTING.md#change-requirements) |

## Focused checks

Select checks for actual changes rather than adding unrelated security work.

* For network changes, inspect binding defaults, explicit LAN selection, and documented firewall scope.
* For input changes, trace malformed or unavailable data through normalization and display.
* Check whether source-formatted text can become executable HTML or script in the changed rendering path.
* For resource changes, inspect slow-client behavior, bounded state, expiry, cancellation, and shutdown.
* For workflow changes, inspect checkout revision, token permissions, credential persistence, and handling of external text.
* For dependency changes, inspect manifests, lockfiles, installation scripts, distribution rights, and relevant upstream security evidence.
* For changed logs and artifacts, check whether credentials or local sensitive paths can enter committed or public output.
* Use [validation by change scope](../docs/testing/index.md#validation-by-change-scope) and the [acceptance validator checks](../docs/pr-acceptance.md#validate-changes-to-the-control) where applicable.
* Request [project-review-change](../.agents/skills/project-review-change/SKILL.md) under the active delegation rules when the task requires it.

## Sources and limits

* Implementation example: [BARO rendering](../web/altitude/baro.js) assigns source-formatted text through `textContent` rather than HTML parsing.
* This entry defines a review procedure rather than a completed security audit or a vulnerability assessment.
* A private-network restriction does not add authentication or encryption.
* Report current dependency security claims only with evidence for the actual versions and review date.
