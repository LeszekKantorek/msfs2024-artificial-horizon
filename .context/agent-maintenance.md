---
description: Maintain project knowledge conventions, upstream and local skills, installation records, and session hooks.
status: active
---

## Knowledge maintenance

Maintain project knowledge under these requirements:

* Select the context skill matching retrieval, preservation, maintenance, mapping, diagnosis, compilation, or session review. Use [context-retrieve](../.agents/skills/context-retrieve/SKILL.md) to locate sources and [context-map](../.agents/skills/context-map/SKILL.md) to maintain situation-to-source routes. Reuse established findings and decisions rather than repeating completed analysis.
* Retain decisions, explanations, findings, and rules useful to future work, with their rationale and conditions of use.
* Follow [documentation maintenance](../CONTRIBUTING.md#documentation-maintenance) for migration completeness, current status, and historical requirements.
* Keep independent topics separate so agents can read only relevant entries. Link canonical sources instead of repeating their commands or rules.
* Remove session progress and expired task-specific permissions from reusable knowledge.
* Remove obsolete pending items for completed work while retaining useful constraints and lessons.
* Preserve existing accepted plans and amendments under the [handoff rules](iterative-development.md#accepted-plans-and-handoffs). Subtasks are the primary future handoff; this does not authorize deleting existing plans.
* For discrepancies outside the editing scope, follow [findings outside the current scope](iterative-development.md#findings-outside-the-current-scope).

### Local entry and index convention

These formats apply to `.context/` entries, their index, and navigation. They are local project conventions, not upstream skill requirements.

| Field / record | Project rule |
| --- | --- |
| `description` | State the entry's subject and when its knowledge is useful |
| `status` | Use `active`, `superseded`, or `deprecated` |
| `superseded_by` | Required for `superseded`; resolve the replacement path relative to the entry |
| Active entry | Apply within its stated scope, retaining evidence limits and uncertainty |
| Superseded entry | Follow the replacement; report broken paths or cycles instead of guessing |
| Deprecated entry | Treat as withdrawn; retain the reason and useful history |
| `.context/index.md` | Keep a table-only map with each active entry once and concrete selection conditions |
| Direct documentation routes | Include recurring tasks that can go directly to a canonical document or section |

Maintain navigation and retrieval under these requirements:

* Put direct routes in the [index](index.md) and conditional routes in [task guidance](task-guidance.md).
* Keep retrieval procedure steps in context-retrieve. [AGENTS](../AGENTS.md#context-selection) owns project selection constraints; [implementation context preparation](iterative-development.md#prepare-subtask-context) applies them to execution. Maps supply destinations and use conditions rather than duplicate the procedure.
* When moving knowledge, update incoming links, index rows, and replacement paths.
* Exclude session records from knowledge retrieval under [hook maintenance](#hook-maintenance).

## Reinstall context skills

Use [setup and skill sources](../CONTRIBUTING.md#3-prepare-agent-skills-and-hooks) to identify the installed revision and installation commands before reinstalling:

1. Inspect the repository diff and preserve intentional local changes before replacing installed files.
2. Install the context skills, including context-retrieve, from the selected upstream revision. Reconcile setup references with that revision when updating the installation.
3. Update `skills-lock.json` and `.agents/skills/` together.
4. Compare installed skill files with the selected upstream revision. Check hooks against their separately documented integration revision.

Preserve these constraints during reinstallation:

* Keep installer-generated `computedHash` values as source provenance. Do not substitute local hash calculations.
* Keep installed context skills byte-identical to the selected upstream revision while evaluating their behavior.
* Keep `procedural-skill-creator` installed from `LeszekKantorek/agent-skills`.
* Preserve the [exclusions](../.gitignore) that include the context skills, including context-retrieve, and `procedural-skill-creator` while excluding unrelated local skills.
* Preserve the [local entry and index convention](#local-entry-and-index-convention).

## Project procedure skills

The project skills below are versioned locally. They are separate from the pinned upstream skills and have no installer entries in `skills-lock.json`.

| Skill | Procedure owner | Supporting knowledge |
| --- | --- | --- |
| [project-write-content](../.agents/skills/project-write-content/SKILL.md) | Edit its `SKILL.md` and Markdown examples | Language remains in `technical-writing-rules`; project conventions remain in [writing guidance](../CONTRIBUTING.md#writing-guidance) |
| [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) | Edit its `SKILL.md` for Task planning and self-contained Subtasks | [Planning rationale](implementation-planning.md); [Subtask execution and handoffs](iterative-development.md#accepted-plans-and-handoffs) |
| [project-diagnose-failure](../.agents/skills/project-diagnose-failure/SKILL.md) | Edit its `SKILL.md` | [Symptom map and sources](error-diagnosis.md); environment restrictions remain in their entries |
| [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) | Edit its `SKILL.md` | Feature checks, commands, and evidence requirements remain in testing documentation |
| [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md) | Edit its `SKILL.md` | Acceptance criteria and CI interpretation retain their existing owners |
| [project-review-change](../.agents/skills/project-review-change/SKILL.md) | Edit its `SKILL.md` | [Evidence selection and limits](../docs/testing/evidence.md#select-evidence-by-failure-risk) |
| [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) | Edit its `SKILL.md` | [Governing-decision map and sources](architecture-decisions.md) |

Maintain project procedures under these ownership and validation requirements:

* Edit each skill's name, description, goal, and procedure in its owning `SKILL.md`.
* Keep one maintained copy of each procedure. Documentation and maps must link to that owner.
* After edits, check selection conditions, relative links, retained constraints, and evidence limits.
* Preserve the project skill inclusions in [.gitignore](../.gitignore). Unrelated local skills must remain ignored.
* Keep agent planning rationale, diagnostic maps, and evidence limits in `.context`, with references to the owning skills.
* Keep project and user requirements and evidence in their relevant `docs/` owners.
* Select skills by the requested outcome. No fixed sequence of project skills is required.
* Apply project-write-content to Markdown and technical-writing-rules to language within its scope.
* Use [procedural-skill-creator](../.agents/skills/procedural-skill-creator/SKILL.md) when designing or revising a skill's executable procedure.
* Keep publicly required rationale and evidence in project documentation; edit procedure steps only in their designated owner.

## Session tooling compatibility

Preserve these session-tooling contracts:

* The session registry is UTF-8 CSV at `.context/sessions.log`, with an OS-backed `.context/.sessions.lock`.
* The tools do not read or migrate `.context/sessions/`. Retain that metadata separately; absence from the CSV registry does not establish review.
* Keep session-tooling tests upstream in `context-skills`.

## Hook maintenance

Maintain the hook integration under these requirements:

* Read [hook setup](../CONTRIBUTING.md#session-hooks) for sources, events, dependencies, and trust steps.
* Preserve unrelated hooks when updating the integration.
* The startup reminder describes the six context skills without prescribing a sequence.
* Use `context-review-sessions` with `context-preserve` for session review. The installed skill reports review outcomes without updating the CSV registry; do not claim automatic completion bookkeeping.
* Subtask execution must leave a [complete main-conversation summary](iterative-development.md#final-subtask-summary) for later JSON session analysis, including resolved gaps and consultation results.
* Do not assume that local journals, detailed test reports, or separate subagent transcripts remain available after execution. No session-analysis tooling changes are required by this reporting rule.
* Exclude the CSV registry, its lock, and session metadata under `.context/sessions/` from knowledge retrieval and commits.
* Stop and PreCompact allow five seconds; Interrupt and SessionEnd retain the host's three-second maximum.
* An unavailable transcript prevents content review but does not prove that the session performed no work.

> Setup documentation and helper tests do not prove that local Codex hook events executed successfully.
