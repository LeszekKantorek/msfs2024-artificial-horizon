| Entry | Use when |
| --- | --- |
| [Start the demo](../README.md#start-the-demo) | Building and running the demo for the first time. |
| [Phone access](../README.md#use-your-phone) | Connecting a phone to the demo over the private LAN. |
| [Product scope](../docs/project-brief.md#scope-and-acceptance) | Determining included behavior, exclusions, or acceptance ownership. |
| [Architecture boundaries](../docs/architecture.md#boundaries) | Locating component responsibilities before changing a runtime boundary. |
| [PFD features](../docs/efi/index.md#features) | Finding requirements, presentation, implementation, and acceptance scenarios for a specific instrument. |
| [Shared PFD boundaries](../docs/efi/index.md#shared-responsibilities) | Changing composition, frames, telemetry, global status, or cross-module rendering. |
| [Validation scope](../docs/testing/index.md#validation-by-change-scope) | Selecting required checks for a change. |
| [Contribution workflow](../CONTRIBUTING.md#workflow) | Preparing an issue, branch, or PR. |
| [Acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) | Recording verified criteria, evidence, and pending checks. |
| [Repository rule history](repository-instructions.md) | Tracing the origins of contribution constraints, completion criteria, or follow-up rules. |
| [PR acceptance control](pr-acceptance.md) | Diagnosing acceptance status, changing the validator, or activating its required check. |
| [Task guidance selection](task-guidance.md) | The direct route is unclear, a task crosses several areas, or routing guidance itself needs maintenance. |
| [Implementation planning](implementation-planning.md) | Finding planning rationale and evidence limits, or the owning project-plan-change skill. |
| [Validation and review](validation-and-review.md) | Designing tests, resolving evidence limits, or finding the owning project-review-change skill; use the testing scope matrix for routine check selection. |
| [Error diagnosis](error-diagnosis.md) | Selecting symptom-specific boundaries and restrictions, or the owning project-diagnose-failure skill. |
| [Security review](security-review.md) | Reviewing changes to network exposure, input handling, dependencies, secrets, permissions, or CI trust boundaries. |
| [Changes across components](component-changes.md) | Selecting contracts and checks for behavior changes across providers, telemetry, HTTP, and the browser. |
| [Architecture decisions](architecture-decisions.md) | Finding governing decisions, their scope and history, or the owning project-decide-architecture skill. |
| [Iterative development](iterative-development.md) | Choosing module boundaries or preserving accepted plans, handoffs, and amendments locally without publishing them. |
| [Windows validation](windows-validation.md) | Diagnosing SAC blocking, error 4551, or Rust E0463. |
| [Build validation](build-validation.md) | Changing Cargo features, build commands, CI caching, or the build status name. |
| [Local validation environment](local-validation-environment.md) | Diagnosing error 5 when replacing a running executable, browser launch failures, or local fixture reachability. |
| [Browser presentation](browser-presentation.md) | Changing shared display behavior, unavailable-state presentation, opacity, or preparing real-phone acceptance. |
| [PFD layout validation](pfd-layout-validation.md) | Changing PFD geometry, clipping, or paint order, or adding the next instrument. |
| [PFD demo scope](pfd-demo-scope.md) | Planning or implementing PFD demo features. |
| [PFD composition](pfd-module-plan.md) | Changing visual-object ownership, frame scheduling, shared layers or clips, or calculation boundaries. |
| [Airspeed and ground speed](airspeed-ground-speed.md) | Extending or validating speed telemetry, tape-unit margins, GS frame, ranges, references, trend, rolling digits, or the demo profile. |
| [True airspeed](true-airspeed.md) | Extending or validating TAS telemetry, its top-left white frame, or TAS acceptance. |
| [Altitude and pressure](altitude-baro.md) | Extending or validating altitude, source-formatted BARO text, signed drums, or the right-side PFD layout. |
| [Altitude trend](altitude-trend.md) | Extending or validating six-second altitude prediction, left-side tick overlap, or notched-bug geometry. |
| [Selected altitude](selected-altitude.md) | Extending or validating targets, alert history, timed panel updates, or the longer altitude demo. |
| [Vertical speed](vertical-speed.md) | Extending or validating source-supplied VSI, the compact altitude/VSI composition, or the shared altitude scenario. |
| [Heading](heading.md) | Extending or validating magnetic headings, numeric readout formatting, north wrapping, selected-bug capture, or top-strip geometry. |
| [Writing style](writing-style.md) | Editing project prose or Markdown structure, or choosing which document owns a requirement. |
| [Context tooling maintenance](agent-maintenance.md) | Maintaining upstream skills, project procedure skills, procedure ownership, or CSV session hooks. |
| [Context authoring](context-authoring.md) | Preserving findings, reviewing knowledge, or changing entry metadata and navigation. |
