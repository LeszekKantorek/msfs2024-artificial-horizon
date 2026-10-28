# Measurements and validation evidence

Measurement limits, build provenance, and matched screenshot comparisons.

## Measurements and evidence

| Owner | Scope |
| --- | --- |
| #7 | Original simulator reliability and performance |
| #20-#26 | Planned EFI demo acceptance, without expanding #7 or #8 |

Record the measurement environment and observations:

* OS, CPU, phone models, and browser versions.
* Simulator, SDK, and aircraft versions.
* Build commit, source rate, network conditions, duration, and failures.
* Memory and subscriber trends with two phones for at least 30 minutes.

| Measurement | Method |
| --- | --- |
| Receipt-to-render | Browser monotonic clock and frame callbacks |
| Total latency | Instrumented source with measured clock offset and uncertainty, or synchronized or high-frame-rate recording of source and phone |
| Report | Method uncertainty, p50, and p95. Never subtract unrelated device timestamps |
| Proposed target | p95 <= 150 ms from source acquisition through visible rendering on healthy LAN |

* Attach results to the validation issue.
* Keep hardware-dependent tasks open until observations support acceptance. Apply the [manual acceptance rules](#sources-and-limits).
* Unit tests cannot certify simulator or phone behavior.

### Build provenance

Distinguish build execution from artifact creation:

* Cargo can reuse an existing artifact after a successful command.
* A changed executable hash establishes different bytes, not their source inputs.

Record these fields with binary-dependent validation:

| Field | Required evidence |
| --- | --- |
| Source | Commit, relevant uncommitted changes, and source checkout path |
| Build | Exact command, target, profile, toolchain, target directory, complete output, and native exit code |
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

Record comparison conditions before inspecting differences:

| Area | Required metadata |
| --- | --- |
| Display | Viewport, scale, browser, and fonts |
| Scene | Input state and capture method |
| Build | Source and artifact identity |

Compare the recorded captures:

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

Use complementary evidence for different requirements:

| Evidence | Establishes | Limit |
| --- | --- | --- |
| Calculation and lifecycle tests | Values, boundaries, availability, expiry, and recovery | Do not establish visible paint order |
| Browser fixture with real modules | Controlled rendering, geometry, clipping, and object lifetime | Does not establish packaged application integration |
| Compiled application browser test | Embedded assets and the actual telemetry-to-display path | Does not establish actual mobile browser behavior |
| Matched screenshots and inspection | Readability and visual differences under recorded conditions | Cover only the inspected scenes |
| Actual phone checks | Device, browser bars, safe areas, rotation, and recovery | Cover only the recorded devices and versions |
| Independent code review | Assessed defects, unintended changes, and test gaps in a specified diff | Does not replace tests or establish absence of defects |

* Use [EFI layout validation](pfd.md#geometry-and-paint-regression-checks) for rendering checks.
* Use [screenshot comparisons](#screenshot-comparisons) for visual evidence.

## Validation scope

* Select checks from the [validation matrix](index.md#validation-by-change-scope).
* Apply the [evidence reuse rules](index.md#evidence-reuse) to earlier results.
* Preserve explicit requirements for fresh final runs and restrictions on execution.
* Report unperformed checks and evidence limits without claiming a pass.

## Independent subagent review

* Include independent review in code-change plans before delivery completion.
* Follow [project-review-change](../../.agents/skills/project-review-change/SKILL.md) for the procedure, delegation conditions, and completion checks.
* Record the independent review result separately from manual acceptance.

## Sources and limits

### Manual acceptance

* Explicit operator confirmation is sufficient within its stated scope.
* Record it as reported acceptance and continue delivery.
* Missing logs, screenshots, device metadata, or direct phone access alone do not block completion or require another test.
* Use available logs and automated results as supporting evidence.
* Do not invent unreported details.

### Repeat tests and follow-up tasks

Request another manual test only for one of these reasons:

| Reason | Explain before requesting the test |
| --- | --- |
| Concrete unresolved failure | Which failure the test would resolve |
| Relevant change after acceptance | Which changed behavior needs verification |
| Explicitly required criterion outside the confirmed scope | Which criterion remains unconfirmed |

* Later usability findings can become separate follow-up tasks.
* Reopen accepted work only when findings establish a failure of its requirements.
