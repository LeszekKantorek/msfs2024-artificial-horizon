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

Use [context-retrieve](.agents/skills/context-retrieve/SKILL.md) to locate project sources before reasoning or planning that depends on project knowledge. An implementation agent must also use it when preparing execution context, including work without an issue.
State the concrete information needed, with known identifiers and constraints. Use the skill's verified source references and explicit gaps to select the passages to read; retrieval does not replace reading the governing requirements or the complete Subtask instructions.

Delegate context-retrieve to one Luna High subagent (`gpt-6-luna`, reasoning effort `high`) for a bounded source-location question:

* Give the subagent the question, known references, scope, and reading constraints. It performs read-only retrieval and returns verified paths, line ranges, brief source descriptions, and missing or inaccessible information.
* The retrieval subagent executes the skill directly without further delegation. It does not edit files, run tests, implement changes, or select the project's technical direction.
* The requesting agent reads the relevant returned passages and retains responsibility for interpretation, planning, and implementation. Reuse applicable verified results rather than repeating unchanged retrieval.
* Check model availability before delegation. Report an unavailable Luna High configuration instead of silently substituting another model.

Select a map by the requested outcome, affected behavior, or unresolved question:

| Need | Map |
| --- | --- |
| Change behavior across component boundaries, such as providers, telemetry, HTTP, or the browser | [Changes across components](.context/component-changes.md) |
| Understand or reconsider an accepted architecture choice | [Governing decisions](.context/architecture-decisions.md#find-the-governing-decision) |
| Find instrument sources or additional guidance for implementation, diagnosis, validation, review, documentation, or delivery | [Task guidance](.context/task-guidance.md#practical-selection) |

Apply these project navigation rules while using context-retrieve:

1. Follow the applicable map rows and their conditions to the relevant sources.
2. Read only the source sections needed for the task.
3. Consult [.context/index.md](.context/index.md) as the final fallback when the maps leave a required source unidentified.
4. Stop discovery when the expected result, applicable constraints, and validation method are clear.
5. Return to the maps when a dependency, conflict, missing fact, or changed scope requires more context.

Apply these reading and validation constraints:

* Before planning or executing issue-based work, read the [GitHub issue context, including artifacts](.context/repository-instructions.md#read-github-issue-context).
* A link is a reference, not an instruction to read every linked document recursively.
* Reuse relevant content already read in this conversation while it remains applicable and unchanged.
* Exclude `.context/sessions.log` and `.context/.sessions.lock` from knowledge retrieval.
* Read README for startup or user-facing behavior, architecture for changed boundaries, and CONTRIBUTING for the applicable contribution stage.
* Apply every relevant route when a change crosses boundaries, including required validation, acceptance, and accepted-plan invariants.
* When validating a change, select required checks from the [validation scope matrix](docs/testing/index.md#validation-by-change-scope).

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
