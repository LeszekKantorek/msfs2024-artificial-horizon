# Agent instructions

## Context selection

Use these maps whenever a task needs project knowledge. Select rows by the requested outcome, affected behavior, or unresolved question.

| Need | Map |
| --- | --- |
| Change behavior across providers, telemetry, HTTP, or the browser | [Changes across components](.context/component-changes.md) |
| Understand or reconsider an accepted architecture choice | [Governing decisions](.context/architecture-decisions.md#find-the-governing-decision) |
| Select procedures for implementation, diagnosis, validation, review, documentation, or delivery | [Task guidance](.context/task-guidance.md#practical-selection) |

1. Follow the applicable map rows and their conditions to the relevant sources.
2. Read only the source sections needed for the task.
3. Consult [.context/index.md](.context/index.md) as the final fallback when the maps leave a required source unidentified.
4. Stop discovery when the expected result, applicable constraints, and validation method are clear.
5. Return to the maps when a dependency, conflict, missing fact, or changed scope requires more context.

* A link is a reference, not an instruction to read every linked document recursively.
* Reuse relevant content already read in this conversation while it remains applicable and unchanged.
* Read README for startup or user-facing behavior, architecture for changed boundaries, and CONTRIBUTING for the applicable contribution stage.
* Apply every relevant route when a change crosses boundaries, including required validation, acceptance, and accepted-plan invariants.
* When validating a change, select required checks from the [validation scope matrix](docs/testing.md#validation-by-change-scope).

## Project constraints

* Write project text in English and preserve the MIT license.
* Keep credentials, SDK binaries, local SDK paths, and generated output out of commits. The only generated-output exception is the PFD skill tracked under its [source ownership rules](.context/agent-maintenance.md#project-procedure-skills).
* Target Windows x64 MSVC for Rust work and keep default checks independent of the simulator.
* Preserve accepted scope and user changes without an arbitrary file-count limit, and keep supplied implementation plans local and verbatim.
* Report only validation actually performed or applicable recorded evidence, with missing checks stated explicitly.

## Project knowledge

Select a project procedure directly when its outcome matches the task:

| Skill | Use |
| --- | --- |
| [project-plan-change](.agents/skills/project-plan-change/SKILL.md) | Prepare a detailed implementation plan with invariants and acceptance evidence |
| [project-diagnose-failure](.agents/skills/project-diagnose-failure/SKILL.md) | Diagnose an observed application, test, build, or environment failure |
| [pfd-validate-presentation](.agents/skills/pfd-validate-presentation/SKILL.md) | Validate PFD presentation and report the limits of desktop and device evidence |
| [project-prepare-pr](.agents/skills/project-prepare-pr/SKILL.md) | Prepare a local PR description or publish the requested PR with accurate acceptance evidence |
| [project-review-change](.agents/skills/project-review-change/SKILL.md) | Obtain and assess an independent diff review when delegation is authorized |
| [project-decide-architecture](.agents/skills/project-decide-architecture/SKILL.md) | Compare architecture options and record an ADR with its actual decision status |

Use the context skills to maintain project knowledge:

| Skill | Use |
| --- | --- |
| `context-preserve` | Preserve decisions and findings with rationale and evidence |
| `context-review-sessions` | Review pending sessions through `context-preserve` |
| `context-map` | Connect tasks and questions to relevant sources |
| `context-maintain` | Improve knowledge coherence and organization |
| `context-diagnose` | Investigate agent mistakes and check corrections |
| `context-compile` | Turn established project procedures into skills |
| `procedural-skill-creator` | Create or revise executable skill procedures |

* Follow each skill's instructions.
* Choose the skill that fits the task; no skill sequence is required.
* Exclude `.context/sessions.log`, `.context/.sessions.lock`, and legacy `.context/sessions/` from knowledge retrieval.
* Use `context-review-sessions` to review registered sessions.
