---
description: Use when choosing module boundaries or extending demo behavior across delivery iterations.
status: active
---

# Build foundations for the next iteration

The project owner established during the issue #1 review on 2026-10-06 that demo
will be built and extended in successive iterations. Each step should provide
foundations for the next one. Assess design choices against both the current
outcome and the next documented use in the [roadmap](../docs/roadmap.md).

A small structural change is justified when it simplifies a concrete next step.
Avoid postponing such a change solely because the current skeleton can work
without it. Future extensibility must still be grounded in agreed requirements;
this preference does not authorize implementing later issues or speculative
frameworks as part of the current task.

Use [architecture](../docs/architecture.md) for target boundaries, including
ownership of source types, and [CONTRIBUTING](../CONTRIBUTING.md) for document
responsibilities and the issue workflow. Those documents remain the canonical
sources; this entry records the owner's development preference and its rationale.

When applying this preference, identify the next use that motivates the choice,
explain its immediate cost, and verify that the current outcome still works.
