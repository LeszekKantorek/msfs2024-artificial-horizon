---
description: Interpret the November 2026 review findings and select the project review procedure without treating historical findings as current defects.
status: active
---

# Review evidence and interpretation

The review procedure belongs to [project-review-change](../.agents/skills/project-review-change/SKILL.md).
It combines component inspection with semantic and lifecycle traces. This entry preserves the evidence behind that choice, not another procedure.

## Evidence basis

The supplied reports inspect revision `3e8fd243938e80083e6629fe53d72591b2f1bbba` through static reasoning.
Their findings are historical observations, not confirmation that a later revision retains each problem.
The original review pair records earlier test executions separately from its final static scope.

| Report | Contribution |
| --- | --- |
| [Original backend](../.local/codding-review-backend/report.md) | Instrument indices, misleading publication trigger, and hidden Demo timeline units. |
| [Original frontend](../.local/codding-review-frontend/report.md) | BARO fallback, cancellation during output waits, and overwritten cleanup diagnostics. |
| [Backend with skill 0.3](../.local/backend_v03/report.md) | Instrument indices and overlapping disconnect deadlines. |
| [Frontend with skill 0.3](../.local/frontend_v03/report.md) | BARO fallback and fixture startup outside cleanup. |
| [Contract review](../.local/contracts_review/report.md) | Producer-to-presentation BARO trace and native-gap versus synthetic-fixture evidence conflict. |
| [Lifecycle review](../.local/lifecycle_review/report.md) | Blocked output, partial startup, and primary native errors replaced by Close failures. |
| [Additional broad review](../.local/code-quality-guidelines/20261110-192804-sol-high/report.md) | BARO, positional identities, and obsolete module commentary about the native Demo boundary. |

These local reports are provenance, not required inputs to execute the skill. They can be absent in another checkout.

## Preserved lessons and limits

| Evidence pattern | Reusable interpretation | Limit |
| --- | --- | --- |
| BARO omission selected text fallback; assertions preserved that behavior | Compare producer, consumer, fixture, and requirement together. Tests can preserve a contradiction. | Healthy native production usually supplied numeric pressure too. |
| Positional identities repeated through conversion, history, and gates | A maintenance finding can establish synchronized-change cost without a current wrong value. | Reports differed between P2 and P3. Frequency does not settle severity. |
| `received_at` used only presence; Demo times hid units and inclusive progress | Inspect semantic meaning separately from runtime correctness. | Generic names are not automatically defects. |
| Shorter nested disconnect replaced a longer deadline | Examine repeated and overlapping temporal events, not only isolated transitions. | The reported sequence was reasoned statically, not reproduced. |
| Drain and flush delayed child shutdown; readiness preceded cleanup protection | Trace cancellation to each blocking dependency and ownership from first acquisition. | Actual orphan duration and storage-stall behavior were not measured. |
| JavaScript cleanup and native Close could replace primary errors | Inspect combined failures and diagnostic paths with logging disabled. | Enabled healthy logs can retain primary native evidence. |
| Synthetic helper omitted VSI immediately while native cache retained it | Distinguish presentation-model evidence from production acquisition evidence. | Native gap retention follows its contract. Final jump-availability promises remain unresolved. |
| Module comment described Demo as non-native | Compare explanatory comments with current architecture. | Historical wording alone does not prove executable malfunction. |

Accepted Demo argument coupling, native resource retention, and meaningful panel delegation were repeatedly excluded as defects.
Their governing decisions remain in the existing architecture and feature owners.

The review sets overlapped only partly. Nonrediscovery does not refute an earlier finding.
Different prompts, reviewers, and scopes prevent attributing improved discovery to a skill version or work division alone.
The project procedure therefore combines perspectives without claiming measured superiority or requiring a fixed number of reviewers or findings.

## Ownership decision

Extend the existing project review skill rather than duplicate review orchestration in a second skill.
Keep project-specific investigation decisions in its single `SKILL.md`, without a `references` directory.
Keep generic quality guidance and canonical product contracts with their existing owners.
Do not encode the historical defects as permanent findings or automatically publish corrections to GitHub.
