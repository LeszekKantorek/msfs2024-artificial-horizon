---
description: Select implementation sources, validate changes, and record follow-up findings. Use before planning or implementing a repository change.
status: active
---

## Steps

1. Read README, architecture, and CONTRIBUTING before implementation, then select the other sources below for the affected scope.
2. Apply their constraints without treating target designs or SDK compatibility as verified behavior.
3. Add focused checks for changed behavior and run the required validation for the affected scope.
4. Report observed results and unavailable hardware checks separately.
5. Collect implementation findings in MVP issue #9 with a reference to the originating task.
6. Review those findings after the originating task closes before deciding on follow-up scope.

## Source

| Source | Use when |
| --- | --- |
| [README](../README.md) | Selecting runnable commands, supported targets, and user-facing limitations. |
| [Architecture](../docs/architecture.md) | Applying component boundaries, library/CLI ownership, SDK isolation, and runtime constraints. |
| [CONTRIBUTING](../CONTRIBUTING.md) | Following the issue workflow, document responsibilities, definition of done, and required checks. |
| [Telemetry contract](../docs/telemetry-contract.md) | Changing telemetry or browser behavior, including read-only delivery and unavailable data. |
| [Testing](../docs/testing.md) | Choosing behavior checks and simulator or mobile acceptance procedures. |
| [Iterative development](iterative-development.md) | Choosing module boundaries or extending demo behavior across iterations. |
| [MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) | Collecting findings and deciding on follow-up scope after the originating task closes. |
