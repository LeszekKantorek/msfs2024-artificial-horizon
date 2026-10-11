---
description: Select tests that detect relevant failures and keep evidence tied to the tested state. Prepare independent subagent review and report its limits separately.
status: active
---

## Validation scope

Use [validation by change scope](../docs/testing.md#validation-by-change-scope) for required checks and [evidence reuse](../docs/testing.md#evidence-reuse) for repeated runs.

* Preserve explicit requirements for fresh final runs and restrictions on execution.
* Report unperformed checks and evidence limits without claiming a pass.

## Select evidence by failure risk

1. Map each changed requirement and preserved invariant to a plausible failure.
2. Select a check that observes the required behavior independently of its implementation.
3. Define controlled inputs, expected results, relevant boundaries, and recovery scenarios.
4. Check whether the test can detect the failure it claims to cover.
5. Use a temporary negative control when a complex test could pass despite the relevant defect.
6. Restore temporary mutations reliably and keep them outside the production change.
7. Run applicable checks on the combined implementation and inspect the required visual evidence.
8. Record the tested revision, environment, inputs, artifact identity where relevant, results, and missing evidence.

Choose complementary evidence rather than treating one successful check as proof of every requirement.

| Evidence | Establishes | Limit |
| --- | --- | --- |
| Calculation and lifecycle tests | Values, boundaries, availability, expiry, and recovery | Do not establish visible paint order |
| Browser fixture with real modules | Controlled rendering, geometry, clipping, and object lifetime | Does not establish packaged application integration |
| Compiled application browser test | Embedded assets and the actual telemetry-to-display path | Does not establish actual mobile browser behavior |
| Matched screenshots and inspection | Readability and visual differences under recorded conditions | Cover only the inspected scenes |
| Actual phone checks | Device, browser bars, safe areas, rotation, and recovery | Cover only the recorded devices and versions |
| Independent code review | Assessed defects, unintended changes, and test gaps in a specified diff | Does not replace tests or establish absence of defects |

Use [PFD layout validation](pfd-layout-validation.md) for rendering checks and [screenshot comparisons](../docs/testing.md#screenshot-comparisons) for visual evidence.

## Independent subagent review

Include this stage in code-change plans before delivery completion.

1. Check whether the active instructions authorize subagent delegation.
2. When delegation is authorized and available, request a separate read-only review after implementation and applicable validation.
3. Supply the exact base and head revisions, checkout, and diff scope.
4. Supply the complete accepted plan and amendments through the local handoff index.
5. Supply applicable contracts, preserved invariants, validation evidence, and known limitations.
6. Ask the reviewer to inspect correctness, regressions, scope changes, and gaps in tests or acceptance evidence.
7. Require actionable findings with severity, file and line, failure scenario, and supporting reasoning.
8. Assess each finding against the requirements and current code before changing the implementation.
9. Resolve established defects and rerun affected checks.
10. Request review of material fixes and record the resulting reviewed revision.
11. Record the review outcome and any unreviewed scope separately from test results.

Use a handoff that permits an independent conclusion.

* Ask the reviewer to challenge the implementation without accepting the author's conclusion as evidence.
* Permit read-only source inspection and relevant checks without concurrent edits to the same files.
* Record unavailable delegation as an unperformed independent review.
* Do not describe self-review, passing tests, or a review request without a returned report as completed independent review.
* Keep unavailable device acceptance separate under [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification).

## Sources and limits

* Source: [#63 implementation in PR #70](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/pull/70), tested at `f7f0bc82889fbc46e1346c7c8430e8b840106bf2`.
* Independent subagent code review and actual iOS Safari/Android Chrome acceptance were not performed for that evidence record.
* Source: request to preserve testing lessons and a subagent review procedure, 2026-10-11.
* The review procedure is guidance for subsequent authorized work, not evidence that PR #70 received an independent review.
* Source: validation scope and rendering guidance consolidated during the full context review, 2026-10-11.
