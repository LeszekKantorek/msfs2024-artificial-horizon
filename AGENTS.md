# Agent instructions

## Working with the user

Apply these scope, authorization, and communication requirements:

* Treat analysis, discussion, and review as separate from authorization to implement or change GitHub. Complete explicitly requested work without confirming routine steps.
* Reassess pending actions when the user changes the instruction. Tool approval does not expand the user's authorization.
* Keep to the requested scope. Do not start unrelated work or add speculative abstractions or documentation.
* Present options and trade-offs for material project decisions. Sol may resolve technical choices within accepted scope; changes to approved requirements or direction require a human decision.
* Distinguish facts, hypotheses, proposals, and accepted decisions. Question assumptions when evidence warrants it, and surface unresolved conflicts between sources.
* Check the current repository and relevant remote state when they may have changed; do not rely on conversation memory as the current state.
* Create additional working trees under `.worktrees/` in the main checkout, following the [contribution workflow](CONTRIBUTING.md#workflow).
* Do not automatically edit historical or completed GitHub issues. Change those records only at the user's explicit request.
* Maintain Epics, Tasks, and Subtasks only within authorized work. Planning alone does not authorize publication or reorganization, and an open issue does not authorize editorial changes.
* Preserve useful issue requirements in their documentation owner without rewriting the source issue. Follow the [issue-record rules](.context/repository-instructions.md#github-issue-records).

## Context selection

Follow [context selection and retrieval delegation](.context/context-retrival-delegation.md) before reasoning or planning that depends on project knowledge and when preparing implementation context.

Select the context skill that matches the requested outcome:

| Skill | When | Additional resources |
| --- | --- | --- |
| [context-retrieve](.agents/skills/context-retrieve/SKILL.md) | Locate sources for a concrete question. | [Selection and delegation](.context/context-retrival-delegation.md) |
| [context-preserve](.agents/skills/context-preserve/SKILL.md) | Preserve decisions and findings with their scope and evidence. | [Knowledge maintenance](.context/agent-maintenance.md#knowledge-maintenance) |
| [context-review-sessions](.agents/skills/context-review-sessions/SKILL.md) | Review pending sessions through context-preserve. | [Knowledge maintenance](.context/agent-maintenance.md#knowledge-maintenance) |
| [context-map](.agents/skills/context-map/SKILL.md) | Connect situations to sources for selective reading. | [Context index](.context/index.md) and [task guidance](.context/task-guidance.md) |
| [context-maintain](.agents/skills/context-maintain/SKILL.md) | Improve coherence and organization of existing project knowledge. | [Knowledge maintenance](.context/agent-maintenance.md#knowledge-maintenance) |
| [context-diagnose](.agents/skills/context-diagnose/SKILL.md) | Diagnose agent mistakes and check targeted improvements. | [Knowledge maintenance](.context/agent-maintenance.md#knowledge-maintenance) |
| [context-compile](.agents/skills/context-compile/SKILL.md) | Turn established procedures into selectable skills. | [Project procedure skills](.context/agent-maintenance.md#project-procedure-skills) |

## Documentation ownership

Use the owning writing and knowledge procedures:

* Apply [technical-writing-rules](.agents/skills/technical-writing-rules/SKILL.md) for language and accuracy within its scope, including non-Markdown technical text.
* Apply [project-write-content](.agents/skills/project-write-content/SKILL.md) whenever creating, revising, moving, or reviewing Markdown, including within implementation tasks. This includes documentation, `.context/`, agent instructions, and Markdown issue or PR text. Ordinary chat is outside its scope.
* Apply [project writing conventions](CONTRIBUTING.md#writing-guidance) to all project text, including code comments and commit messages.
* For knowledge work, select the matching context skill and follow [knowledge maintenance](.context/agent-maintenance.md#knowledge-maintenance).
* For skill design, use procedural-skill-creator under the [source ownership rules](.context/agent-maintenance.md#project-procedure-skills).

Organize issue-based work under these requirements:

* Follow the [work organization model](docs/work-organization.md) for Epic → Task → Subtask.
* Treat [MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) as the target MVP Epic. Its migration is a separate authorized change.
* Keep delivery order and dependencies in GitHub without maintaining a competing roadmap document.

Keep documentation in its subject's owner:

* Agent-facing technical materials, including `.context/`, `AGENTS.md`, `SKILL.md`, and skill reference files, may link into `.context/`.
* Project documentation, user instructions, and other non-agent documentation must not link into `.context/`.
* Before creating documentation, check whether an existing document owns the subject. Select the owner by subject, not by the task that revealed the rule.
* Use the existing context maps. When no route fits, inspect directly related sources before proposing a new location.
* Avoid duplicating implementation details that are clear from code.
* Keep agent procedures and supporting knowledge in their agent-facing sources. When project or user documentation needs that content, publish it in the appropriate documentation owner.
* Keep feature requirements, presentation, implementation details, and acceptance scenarios on the owning feature page; shared pages contain only shared contracts and procedures.

## Project constraints

Apply these delivery, platform, and evidence requirements:

* Treat the server, telemetry contract, and embedded Web as one application. Rebuild and deliver the complete application after changes to application code or build inputs. Documentation-only changes require no application build.
* Run screenshot tests and inspect the captures when browser JavaScript or display behavior changes. Select checks through the validation scope matrix.
* Do not maintain backward compatibility with earlier application versions. Update producers, consumers, and fixtures together without legacy adapters or fallbacks to previous formats or values.
* Keep documentation links out of application source code and runtime output. Documentation may reference code.
* Write project text in English and preserve the MIT license.
* Keep credentials, SDK binaries, local SDK paths, and generated output out of commits.
* Target Windows x64 MSVC for Rust work and keep default checks independent of the simulator.
* Preserve accepted scope and user changes without an arbitrary file-count limit. Keep supplied accepted plan originals local and verbatim; future Subtasks are the primary execution handoff.
* Report only validation actually performed or applicable recorded evidence, with missing checks stated explicitly.

## Project knowledge

Select and execute project procedures under these requirements:

* Select a procedure whose outcome matches the requested work. Use [task guidance](.context/task-guidance.md#practical-selection) for additional sources when their conditions apply.
* Sol Medium plans and consults; Terra Medium executes under the [Subtask execution rules](.context/iterative-development.md#accepted-plans-and-handoffs).
* During Subtask execution, Terra may delegate bounded technical consultations to Sol when supported. This permission does not authorize parallel implementation or GitHub changes.
* Report execution results, gaps, consultations, decisions, and their effectiveness in the main conversation under the [final summary rules](.context/iterative-development.md#final-subtask-summary).

### Project procedures

Maintain each procedure in its owning `SKILL.md` under the [source ownership rules](.context/agent-maintenance.md#project-procedure-skills):

| Skill | Use |
| --- | --- |
| [project-write-content](.agents/skills/project-write-content/SKILL.md) | Construct and review Markdown using decision rules and examples while preserving meaning |
| [project-plan-change](.agents/skills/project-plan-change/SKILL.md) | Plan a Task as self-contained executable Subtasks with decisions, context, and acceptance evidence |
| [project-diagnose-failure](.agents/skills/project-diagnose-failure/SKILL.md) | Diagnose an observed application, test, build, or environment failure |
| [project-validate-presentation](.agents/skills/project-validate-presentation/SKILL.md) | Validate EFI presentation and report the limits of desktop and device evidence |
| [project-prepare-pr](.agents/skills/project-prepare-pr/SKILL.md) | Prepare a local PR description or publish the requested PR with accurate acceptance evidence |
| [project-review-change](.agents/skills/project-review-change/SKILL.md) | Obtain and assess an independent diff review when delegation is authorized |
| [project-decide-architecture](.agents/skills/project-decide-architecture/SKILL.md) | Compare architecture options and record an ADR with its actual decision status |
