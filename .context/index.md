| Entry | Use when |
| --- | --- |
| [Start the demo](../README.md#start-the-demo) | Building and running the demo for the first time. |
| [Phone access](../README.md#use-your-phone) | Connecting a phone to the demo over the private LAN. |
| [Product scope](../docs/project-brief.md#scope-and-acceptance) | Determining included behavior, exclusions, or acceptance ownership. |
| [Architecture boundaries](../docs/architecture.md#boundaries) | Locating component responsibilities before changing a runtime boundary. |
| [HTTP server](../docs/server.md) | Changing routers, assets, health, SSE delivery, or connection shutdown. |
| [Source providers](../docs/sources/index.md) | Selecting or changing acquisition and normalization responsibilities. |
| [Demo scenarios](../docs/sources/demo.md#scenario-timing) | Checking deterministic source values, cycles, altitude phases, or fixture expectations. |
| [SimConnect integration](../docs/sources/simconnect.md) | Changing native acquisition, diagnostics, or assessing live evidence limits. |
| [PFD features](../docs/efi/index.md#features) | Finding requirements, presentation, implementation, and acceptance scenarios for a specific instrument. |
| [Shared PFD boundaries](../docs/efi/index.md#shared-responsibilities) | Changing composition, frames, telemetry, global status, or cross-module rendering. |
| [Validation scope](../docs/testing/index.md#validation-by-change-scope) | Selecting required checks for a change. |
| [Contribution workflow](../CONTRIBUTING.md#workflow) | Preparing an issue, branch, or PR. |
| [Acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) | Recording verified criteria, evidence, and pending checks. |
| [Repository rules](repository-instructions.md) | Finding contribution constraints, acceptance criteria, or follow-up rules. |
| [PR acceptance control](pr-acceptance.md) | Diagnosing acceptance status, changing the validator, or activating its required check. |
| [Task guidance selection](task-guidance.md) | The direct route is unclear, a task crosses several areas, or routing guidance itself needs maintenance. |
| [Implementation planning](implementation-planning.md) | Finding planning rationale and evidence limits, or the owning project-plan-change skill. |
| [Error diagnosis](error-diagnosis.md) | Selecting symptom-specific boundaries and restrictions, or the owning project-diagnose-failure skill. |
| [Security review](security-review.md) | Reviewing changes to network exposure, input handling, dependencies, secrets, permissions, or CI trust boundaries. |
| [Changes across components](component-changes.md) | Selecting contracts and checks for behavior changes across providers, telemetry, HTTP, and the browser. |
| [Architecture decisions](architecture-decisions.md) | Finding governing decisions, their scope and history, or the owning project-decide-architecture skill. |
| [Iterative development](iterative-development.md) | Choosing module boundaries or preserving accepted plans, handoffs, and amendments locally without publishing them. |
| [Windows validation](windows-validation.md) | Diagnosing SAC blocking, error 4551, or Rust E0463. |
| [Build validation](build-validation.md) | Changing Cargo features, local build commands, workflow checks, or required statuses. |
| [Local validation environment](local-validation-environment.md) | Diagnosing missing Playwright, blocked local test operations, executable replacement, or fixture reachability. |
| [Browser presentation](browser-presentation.md) | Changing shared display behavior, unavailable-state presentation, opacity, or preparing real-phone acceptance. |
| [PFD presentation](pfd-presentation.md) | Changing composition, frame scheduling, geometry, clipping, paint order, or visual validation. |
| [PFD demo scope](pfd-demo-scope.md) | Planning or implementing PFD demo features. |
| [Speed instruments](speed.md) | Extending or validating IAS, GS, TAS, speed ranges, references, or trend. |
| [Altitude instruments](altitude.md) | Extending or validating altitude, BARO, selected altitude, vertical speed, or altitude trend. |
| [Heading](heading.md) | Extending or validating magnetic headings, numeric readout formatting, north wrapping, selected-bug capture, or top-strip geometry. |
| [Writing style](writing-style.md) | Editing project prose or Markdown structure, or choosing which document owns a requirement. |
| [Context tooling maintenance](agent-maintenance.md) | Maintaining upstream skills, project procedure skills, procedure ownership, or CSV session hooks. |
| [Context authoring](context-authoring.md) | Preserving findings, reviewing knowledge, or changing entry metadata and navigation. |
