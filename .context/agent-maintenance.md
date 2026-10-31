---
description: Maintain project knowledge conventions, upstream and local skills, installation records, and session hooks.
status: active
---

## Knowledge maintenance

* Select the context skill matching preservation, maintenance, mapping, diagnosis, compilation, or session review. Reuse established findings and decisions rather than repeating completed analysis.
* Retain decisions, explanations, findings, and rules useful to future work, with their rationale and conditions of use.
* Follow [documentation maintenance](../CONTRIBUTING.md#documentation-maintenance) for migration completeness, current status, and historical requirements.
* Keep independent topics separate so agents can read only relevant entries. Link canonical sources instead of repeating their commands or rules.
* Remove session progress and expired task-specific permissions from reusable knowledge.
* Remove obsolete pending items for completed work while retaining useful constraints and lessons.
* Preserve active plans under the [plan-preservation rules](iterative-development.md#accepted-plans-and-handoffs). Removing obsolete expectations does not authorize deleting those plans.
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

* Put direct routes in the [index](index.md) and conditional routes in [task guidance](task-guidance.md).
* When moving knowledge, update incoming links, index rows, and replacement paths.
* Exclude session records from knowledge retrieval under [hook maintenance](#hook-maintenance).

## Reinstall context skills

Use [setup and skill sources](../CONTRIBUTING.md#3-prepare-agent-skills-and-hooks) for the tested revision and installation commands.

1. Inspect the repository diff and preserve intentional local changes before replacing installed files.
2. Install the six context skills from the pinned revision in setup.
3. Update `skills-lock.json` and `.agents/skills/` together.
4. Compare the installed files and hooks with the selected upstream revision.

* Keep installer-generated `computedHash` values as source provenance; do not substitute local hash calculations.
* Keep the six context skills byte-identical to the tested upstream revision while evaluating their behavior.
* Keep `procedural-skill-creator` installed from `LeszekKantorek/agent-skills`.
* Preserve the [exclusions](../.gitignore) that include the six context skills and `procedural-skill-creator` while excluding unrelated local skills.
* Preserve the [local entry and index convention](#local-entry-and-index-convention).

## Project procedure skills

The project skills below are versioned locally. They are separate from the pinned upstream skills and have no installer entries in `skills-lock.json`.

| Skill | Procedure owner | Supporting knowledge |
| --- | --- | --- |
| [project-write-content](../.agents/skills/project-write-content/SKILL.md) | Edit its `SKILL.md` and Markdown examples | Language remains in `technical-writing-rules`; project conventions remain in [writing guidance](../CONTRIBUTING.md#writing-guidance) |
| [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) | Edit its `SKILL.md` | [Planning rationale and sources](implementation-planning.md); [handoff requirements](iterative-development.md#accepted-plans-and-handoffs) |
| [project-diagnose-failure](../.agents/skills/project-diagnose-failure/SKILL.md) | Edit its `SKILL.md` | [Symptom map and sources](error-diagnosis.md); environment restrictions remain in their entries |
| [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) | Edit its `SKILL.md` | Feature checks, commands, and evidence requirements remain in testing documentation |
| [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md) | Edit its `SKILL.md` | Acceptance criteria and CI interpretation retain their existing owners |
| [project-review-change](../.agents/skills/project-review-change/SKILL.md) | Edit its `SKILL.md` | [Evidence selection and limits](../docs/testing/evidence.md#select-evidence-by-failure-risk) |
| [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) | Edit its `SKILL.md` | [Governing-decision map and sources](architecture-decisions.md) |

Each project skill owns its name, description, goal, and procedure. Edit its `SKILL.md` directly.
Keep one maintained copy of each procedure; documentation and maps link to the owning skill.
After edits, check selection conditions, relative links, and preservation of constraints and evidence limits.
The project skill directories are explicitly included by [.gitignore](../.gitignore); unrelated local skills remain ignored.
Keep agent planning rationale, diagnostic maps, and evidence limits in `.context`, with references to the owning skills.
Project and user documentation retain their relevant requirements and evidence in `docs/`.
Select skills by task; no fixed sequence of project skills is required. Apply project-write-content to Markdown and technical-writing-rules to language within its scope.

* Use [procedural-skill-creator](../.agents/skills/procedural-skill-creator/SKILL.md) when designing or revising a skill's executable procedure.
* Keep publicly required rationale and evidence in project documentation; edit procedure steps only in their designated owner.

## Session tooling compatibility

* The session registry is UTF-8 CSV at `.context/sessions.log`, with an OS-backed `.context/.sessions.lock`.
* The tools do not read or migrate `.context/sessions/`. Retain that metadata separately; absence from the CSV registry does not establish review.
* Keep session-tooling tests upstream in `context-skills`.

## Hook maintenance

* Read [hook setup](../CONTRIBUTING.md#session-hooks) for sources, events, dependencies, and trust steps.
* Preserve unrelated hooks when updating the integration.
* The startup reminder describes the six context skills without prescribing a sequence.
* Use `context-review-sessions` with `context-preserve` for pending-session review and completion bookkeeping.
* Exclude the CSV registry, its lock, and session metadata under `.context/sessions/` from knowledge retrieval and commits.
* Stop and PreCompact allow five seconds; Interrupt and SessionEnd retain the host's three-second maximum.
* An unavailable transcript prevents content review but does not prove that the session performed no work.

> Setup documentation and helper tests do not prove that local Codex hook events executed successfully.
