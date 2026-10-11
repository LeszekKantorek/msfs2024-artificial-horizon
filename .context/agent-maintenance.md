---
description: Maintain the three context skills, their installation records, and session hooks.
status: active
---

## Reinstall context skills

Use this procedure when maintaining the context tools named in [AGENTS](../AGENTS.md).

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
* Keep `.context/index.md` as the table-only index.
* Preserve the [exclusions](../.gitignore) that include only the three context skill directories by default.

## Project adaptations

Use [task guidance](task-guidance.md#working-approach) for the project workflow and its policy on retired expert skills.

* The versioned session importer lists the queue without writing and distinguishes missing paths from missing files.
* Both queue writers retry transient Windows replacement failures for up to one second while preserving the queue lock.
* Keep the five-second hook timeout sufficient for Git discovery, lock acquisition, replacement retries, and process startup.
* Compare importer behavior after reinstalling because installation can replace the adaptation.
* Keep session-tooling tests in `context-skills`; the proposed changes and test source are recorded in [context-skills #4](https://github.com/LeszekKantorek/context-skills/issues/4).

The upstream lock records the installed source, not a claim that adapted copies remain byte-identical to it.

## Hook maintenance

* Read [hook setup](../CONTRIBUTING.md#session-hooks) for sources, files, events, dependencies, and trust steps.
* Follow `context-import-sessions` for checkpoint selection, reading, and completion bookkeeping.
* Repeat the unlocked read-only queue snapshot if concurrent hook updates make the result incomplete.
* An unavailable transcript prevents content review but does not prove that the session performed no work.
* Preserve unrelated hooks.
* Align the startup reminder with `.context/index.md`.

> Setup documentation does not prove that local hooks executed successfully.

Sources: [AGENTS](../AGENTS.md) defines roles, each skill defines its procedure, and moving maintenance rules here was approved on 2026-10-09.

Source: accepted project-maintenance review, 2026-10-11, covering local skill adaptations and read-only session inspection.

Source: explicit migration to project context and existing Markdown guidance, 2026-10-11.
