# Measurements and validation evidence

Measurement limits, build provenance, and matched screenshot comparisons.

## Measurements and evidence

| Owner | Scope |
| --- | --- |
| #7 | Original simulator reliability/performance |
| #20-#26 | Planned PFD demo acceptance, without expanding #7 or #8 |

Record:

* OS, CPU, phone models, and browser versions.
* Simulator, SDK, and aircraft versions.
* Build commit, source rate, network conditions, duration, and failures.
* Memory/subscriber trends with two phones for at least 30 minutes.

| Measurement | Method |
| --- | --- |
| Receipt-to-render | Browser monotonic clock and frame callbacks |
| Total latency | Instrumented source with measured clock offset/uncertainty, or synchronized/high-frame-rate recording of source and phone |
| Report | Method uncertainty and p50/p95. Never subtract unrelated device timestamps |
| Proposed target | p95 <= 150 ms from source acquisition through visible rendering on healthy LAN |

Attach results to the validation issue.
Keep hardware-dependent tasks open until observations support acceptance.
Unit tests cannot certify simulator or phone behavior.

### Build provenance

Distinguish build execution from artifact creation:

* Cargo can reuse an existing artifact after a successful command.
* A changed executable hash establishes different bytes, not their source inputs.

Record these fields with binary-dependent validation:

| Field | Required evidence |
| --- | --- |
| Source | Commit, relevant uncommitted changes, and source checkout path |
| Build | Exact command, target/profile, toolchain, target directory, complete output, and native exit code |
| Artifact | Absolute executable path, size, timestamp, and structured SHA-256 value |
| Runtime check | Exact command, artifact path, environment, output, and native exit code |

Preserve usable artifact records:

* Store hash data as structured JSON or an object with path and hash fields.
* Avoid display tables that truncate paths or hash values.
* Link existing logs and artifacts from context entries instead of copying their full contents.

If a copied checkout and build cache leave artifact provenance uncertain:

1. Select a fresh target directory.
2. Rebuild from the source checkout into that target directory.
3. Record the new artifact before making claims about that binary.

> Command success or a runtime screenshot alone does not establish recompilation.

### Screenshot comparisons

Compare captures only after recording viewport, scale, browser, fonts, input state, capture method, and source/artifact identity.

1. Preserve baseline and candidate captures with their metadata.
2. Record the difference extent and location, including raw comparison output.
3. Inspect the changed area visually.
4. Assess whether the difference violates the applicable geometry, readability, color, or availability requirement.
5. Record a cause only when evidence establishes it.

Report uncertainty explicitly:

* Use "cause not established" when the mechanism behind a pixel difference remains unknown.
* Separate the observed difference, its impact assessment, and any possible cause.
* Do not present browser noise or rendering variation as an established cause without evidence.
* Leave affected acceptance unresolved when the evidence cannot determine the impact.

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

Use [PFD layout validation](pfd.md#geometry-and-paint-regression-checks) for rendering checks and [screenshot comparisons](#screenshot-comparisons) for visual evidence.

## Validation scope

Use the [validation matrix](index.md#validation-by-change-scope) and [evidence reuse](index.md#evidence-reuse).
Preserve explicit requirements for fresh final runs and restrictions on execution.
Report unperformed checks and evidence limits without claiming a pass.

## Independent subagent review

Include this stage in code-change plans before delivery completion.
The [project-review-change skill](../../.agents/skills/project-review-change/SKILL.md) owns the procedure, delegation conditions, and completion checks.

## Sources and limits

* Source: [#63 implementation in PR #70](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/pull/70), tested at `f7f0bc82889fbc46e1346c7c8430e8b840106bf2`.
* Independent subagent code review and actual iOS Safari/Android Chrome acceptance were not performed for that evidence record.
* Source: request to preserve testing lessons and a subagent review procedure, 2026-10-11.
* The review procedure guides subsequent authorized work; it does not establish that PR #70 received an independent review.
* Source: validation scope and rendering guidance consolidated during the full context review, 2026-10-11.
