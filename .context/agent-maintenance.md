---
description: Maintain the six context skills, procedural-skill-creator, installation records, and session hooks.
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

## Test revision and compatibility

* The test installation uses [context-skills PR #7](https://github.com/LeszekKantorek/context-skills/pull/7), branch `codex/context-skills-refresh`, commit `2598a6246b8649f7eaea68720201d9dbff1ddd84`.
* The six skills replace the retired Apply, Gather, and Import Sessions skills; do not reinstall the retired set for testing.
* The session registry is UTF-8 CSV at `.context/sessions.log`, with an OS-backed `.context/.sessions.lock`.
* The new tools do not migrate or read the legacy `.context/sessions/` registry.
* Retain legacy session metadata separately; absence from the new registry does not establish completed review.
* Previous local importer adaptations apply to the retired implementation; do not copy them into the new workflow without evidence of a current defect.
* Keep session-tooling tests upstream in `context-skills`.

## Hook maintenance

* Read [hook setup](../CONTRIBUTING.md#session-hooks) for sources, events, dependencies, and trust steps.
* Preserve unrelated hooks when updating the integration.
* The startup reminder describes the six context skills without prescribing a sequence.
* Use `context-review-sessions` with `context-preserve` for pending-session review and completion bookkeeping.
* Exclude the CSV registry, its lock, and legacy session metadata from knowledge retrieval and commits.
* Stop and PreCompact allow five seconds; Interrupt and SessionEnd retain the host's three-second maximum.
* An unavailable transcript prevents content review but does not prove that the session performed no work.

> Setup documentation and helper tests do not prove that local Codex hook events executed successfully.

## Sources

* Source: requested replacement with PR #7 and installation of `procedural-skill-creator`, 2026-10-17.
* Source: installation instructions and Codex integration at the pinned upstream revision.
* Source: earlier project maintenance rules, 2026-10-09 through 2026-10-11; retired implementation details are superseded by the test revision above.
