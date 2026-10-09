---
description: Maintain installed context skills and session hooks. Use when reinstalling skills, migrating their files, or changing the session registration integration.
status: active
---

## Skills and hooks

The project uses [context-skills](https://github.com/LeszekKantorek/context-skills).
[AGENTS.md](../AGENTS.md) defines skill roles. Each skill defines its procedure.

### Reinstall skills

Follow [environment setup](../CONTRIBUTING.md#environment-setup) for prerequisites, skill sources, and first installation.

For a clean reinstall, remove the existing copies:

```powershell
npx --yes skills remove context-apply context-gather context-import-sessions -y
```

Then run the [skill installation command](../CONTRIBUTING.md#3-prepare-agent-skills-and-hooks).

* Update `skills-lock.json` and installed `.agents/skills/` files together.
* Remove retired skill copies during migrations.
* Keep `.context/index.md` as the table-only navigation index.

### Session hooks

Use [session hook setup](../CONTRIBUTING.md#session-hooks) for integration sources, files, events, dependencies, and trust steps.

* Process selected checkpoints with `context-import-sessions`.
* Mark checkpoints reviewed only after reading their contents and successful gathering.
* Preserve unrelated hooks when updating the integration.
* Keep the startup reminder aligned with `.context/index.md`.

## Source

* Maintenance instructions moved from CONTRIBUTING at the owner's request in the documentation prototype session on 2026-10-09.
* Setup documentation identifies integration sources, not evidence that local hooks executed successfully.
