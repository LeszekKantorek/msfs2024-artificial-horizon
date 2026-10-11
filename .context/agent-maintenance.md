---
description: Maintain installed context skills, local skill exclusions, and session hooks.
status: active
---

## Reinstall skills

1. Read [setup and skill sources](../CONTRIBUTING.md#3-prepare-agent-skills-and-hooks).
2. Inspect the repository diff and preserve intentional local adaptations before replacing installed files.
3. Remove existing copies for a clean reinstall:

   ```powershell
   npx --yes skills remove context-apply context-gather context-import-sessions -y
   ```

4. Run the installation command from setup.
5. Compare the installed files with the project adaptations below before accepting the reinstall.

* Update `skills-lock.json` and `.agents/skills/` together when changing their upstream installation.
* Keep upstream `computedHash` values as source provenance without replacing them with undocumented local hash calculations.
* Review intentional repository adaptations separately from the upstream lock record.
* Remove retired skill copies during migrations.
* Keep `.context/index.md` as the table-only index.
* Follow the [skill exclusions](../.gitignore) when adding skills; only the three listed context skill directories are included by default.

## Project adaptations

The [skill selection rules](skill-selection.md#planning-and-verification-scope) own the planning and verification adaptations.

* Local `brainstorming` respects accepted plans and permits direct planning when `writing-plans` is unavailable.
* Local `verification-before-completion` checks whether evidence still applies before requiring another execution.
* These two skill copies are Git-ignored and do not arrive with a fresh checkout.
* Apply the versioned project rules if those local skills are unavailable or replaced during installation.
* The versioned session importer lists the queue without writing and distinguishes missing paths from missing files.
* Both queue writers retry transient Windows replacement failures for up to one second while preserving the queue lock.
* Keep the five-second hook timeout sufficient for Git discovery, lock acquisition, replacement retries, and process startup.
* Compare importer behavior after reinstalling because installation can replace the adaptation.
* Keep session-tooling tests in `context-skills`; the proposed changes and test source are recorded in [context-skills #4](https://github.com/LeszekKantorek/context-skills/issues/4).

The upstream lock records the installed source, not a claim that adapted copies remain byte-identical to it.

## Hook maintenance

* Read [hook setup](../CONTRIBUTING.md#session-hooks) for sources, files, events, dependencies, and trust steps.
* Import selected checkpoints with `context-import-sessions`.
* Mark checkpoints reviewed only after reading their contents and successful gathering.
* Keep report-only, unavailable, and incomplete imports pending.
* Repeat the unlocked read-only queue snapshot if concurrent hook updates make the result incomplete.
* An unavailable transcript prevents content review but does not prove that the session performed no work.
* Preserve unrelated hooks.
* Align the startup reminder with `.context/index.md`.

> Setup documentation does not prove that local hooks executed successfully.

Sources: [AGENTS](../AGENTS.md) defines roles, each skill defines its procedure, and moving maintenance rules here was approved on 2026-10-09.

Source: accepted project-maintenance review, 2026-10-11, covering local skill adaptations and read-only session inspection.
