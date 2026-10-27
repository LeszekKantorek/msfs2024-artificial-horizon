---
description: Maintain upstream context skills, locally owned project skills, installation records, and session hooks.
status: active
---

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
* Keep `.context/index.md` as this project's table-only navigation map; the upstream skills impose no index format.

## Project procedure skills

These six project skills are versioned locally. They are separate from the pinned upstream skills and have no installer entries in `skills-lock.json`.

| Skill | Procedure owner | Supporting knowledge |
| --- | --- | --- |
| [project-plan-change](../.agents/skills/project-plan-change/SKILL.md) | Edit its `SKILL.md` | [Planning rationale and sources](implementation-planning.md); [handoff requirements](iterative-development.md#accepted-plans-and-handoffs) |
| [project-diagnose-failure](../.agents/skills/project-diagnose-failure/SKILL.md) | Edit its `SKILL.md` | [Symptom map and sources](error-diagnosis.md); environment restrictions remain in their entries |
| [project-validate-presentation](../.agents/skills/project-validate-presentation/SKILL.md) | Edit its `SKILL.md` | Feature checks, commands, and evidence requirements remain in testing documentation |
| [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md) | Edit its `SKILL.md` | Acceptance criteria and CI interpretation retain their existing owners |
| [project-review-change](../.agents/skills/project-review-change/SKILL.md) | Edit its `SKILL.md` | [Evidence selection and limits](../docs/testing/evidence.md#select-evidence-by-failure-risk) |
| [project-decide-architecture](../.agents/skills/project-decide-architecture/SKILL.md) | Edit its `SKILL.md` | [Governing-decision map and sources](architecture-decisions.md) |

Each project skill owns its name, description, goal, and procedure. Edit its `SKILL.md` directly.
Keep one maintained copy of each procedure; documentation and maps link to the owning skill.
After edits, check selection conditions, relative links, and preservation of constraints and evidence limits.
The six skill directories are explicitly included by [.gitignore](../.gitignore); unrelated local skills remain ignored.
Keep agent planning rationale, diagnostic maps, and evidence limits in `.context`, with references to the owning skills.
Project and user documentation retain their relevant requirements and evidence in `docs/`.
Select skills by task; no sequence of the six skills is required.

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
