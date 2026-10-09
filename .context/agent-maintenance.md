---
description: Maintain installed context skills and session hooks.
status: active
---

## Reinstall skills

1. Read [setup and skill sources](../CONTRIBUTING.md#3-prepare-agent-skills-and-hooks).
2. Remove existing copies for a clean reinstall:

   ```powershell
   npx --yes skills remove context-apply context-gather context-import-sessions -y
   ```

3. Run the installation command from setup.

* Update `skills-lock.json` and `.agents/skills/` together.
* Remove retired skill copies during migrations.
* Keep `.context/index.md` as the table-only index.

## Hook maintenance

* Read [hook setup](../CONTRIBUTING.md#session-hooks) for sources, files, events, dependencies, and trust steps.
* Import selected checkpoints with `context-import-sessions`.
* Mark checkpoints reviewed only after reading their contents and successful gathering.
* Preserve unrelated hooks.
* Align the startup reminder with `.context/index.md`.

> Setup documentation does not prove that local hooks executed successfully.

Sources: [AGENTS](../AGENTS.md) defines roles, each skill defines its procedure, and moving maintenance rules here was approved on 2026-10-09.
