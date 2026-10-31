# Agent instructions

## Working with the user

* Treat analysis, discussion, and review as separate from authorization to implement or change GitHub. Complete explicitly requested work without confirming routine steps.
* Keep to the requested scope. Do not start unrelated work or add speculative abstractions or documentation.
* Present options and trade-offs for material project decisions; do not treat discussion as approval.
* Distinguish facts, hypotheses, proposals, and accepted decisions. Question assumptions when evidence warrants it, and surface unresolved conflicts between sources.
* Check the current repository and relevant remote state when they may have changed; do not rely on conversation memory as the current state.

## Context selection

Use these maps whenever a task needs project knowledge. Select rows by the requested outcome, affected behavior, or unresolved question.

| Need | Map |
| --- | --- |
| Change behavior across component boundaries, such as providers, telemetry, HTTP, or the browser | [Changes across components](.context/component-changes.md) |
| Understand or reconsider an accepted architecture choice | [Governing decisions](.context/architecture-decisions.md#find-the-governing-decision) |
| Find instrument sources or additional guidance for implementation, diagnosis, validation, review, documentation, or delivery | [Task guidance](.context/task-guidance.md#practical-selection) |

1. Follow the applicable map rows and their conditions to the relevant sources.
2. Read only the source sections needed for the task.
3. Consult [.context/index.md](.context/index.md) as the final fallback when the maps leave a required source unidentified.
4. Stop discovery when the expected result, applicable constraints, and validation method are clear.
5. Return to the maps when a dependency, conflict, missing fact, or changed scope requires more context.

* A link is a reference, not an instruction to read every linked document recursively.
* Reuse relevant content already read in this conversation while it remains applicable and unchanged.
* Read README for startup or user-facing behavior, architecture for changed boundaries, and CONTRIBUTING for the applicable contribution stage.
* Apply every relevant route when a change crosses boundaries, including required validation, acceptance, and accepted-plan invariants.
* When validating a change, select required checks from the [validation scope matrix](docs/testing/index.md#validation-by-change-scope).

## Documentation ownership

* [Roadmap issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) is the sole roadmap. Read delivery order and dependencies there. Do not create or maintain a separate roadmap document.

Agent-facing technical materials, including `.context/`, `AGENTS.md`, `SKILL.md`, and skill reference files, may link into `.context/`.
Project documentation, user instructions, and other non-agent documentation must not link into `.context/`.
Before creating documentation, check whether an existing document owns the subject. Avoid duplicating implementation details that are clear from code.
Keep agent procedures and supporting knowledge in their agent-facing sources. When project or user documentation needs that content, publish the relevant information in `docs/` or its appropriate documentation owner.
Keep feature requirements, presentation, implementation details, and acceptance scenarios on the owning feature page; shared pages contain only shared contracts and procedures.

## Project constraints

* Treat the server, telemetry contract, and embedded Web as one application. Rebuild and deliver the complete application after every change.
* Do not maintain backward compatibility with earlier application versions. Update producers, consumers, and fixtures together without legacy adapters or fallbacks to previous formats or values.
* Keep documentation links out of application source code and runtime output. Documentation may reference code.
* Write project text in English and preserve the MIT license.
* Keep credentials, SDK binaries, local SDK paths, and generated output out of commits.
* Target Windows x64 MSVC for Rust work and keep default checks independent of the simulator.
* Preserve accepted scope and user changes without an arbitrary file-count limit, and keep supplied implementation plans local and verbatim.
* Report only validation actually performed or applicable recorded evidence, with missing checks stated explicitly.

## Project knowledge

Select a project procedure directly when its outcome matches the task. Use [task guidance](.context/task-guidance.md#practical-selection) to select additional sources when their conditions apply.

### Project procedures

Maintain each procedure in its owning `SKILL.md` under the [source ownership rules](.context/agent-maintenance.md#project-procedure-skills):

| Skill | Use |
| --- | --- |
| [project-plan-change](.agents/skills/project-plan-change/SKILL.md) | Prepare a detailed implementation plan with invariants and acceptance evidence |
| [project-diagnose-failure](.agents/skills/project-diagnose-failure/SKILL.md) | Diagnose an observed application, test, build, or environment failure |
| [project-validate-presentation](.agents/skills/project-validate-presentation/SKILL.md) | Validate EFI presentation and report the limits of desktop and device evidence |
| [project-prepare-pr](.agents/skills/project-prepare-pr/SKILL.md) | Prepare a local PR description or publish the requested PR with accurate acceptance evidence |
| [project-review-change](.agents/skills/project-review-change/SKILL.md) | Obtain and assess an independent diff review when delegation is authorized |
| [project-decide-architecture](.agents/skills/project-decide-architecture/SKILL.md) | Compare architecture options and record an ADR with its actual decision status |
