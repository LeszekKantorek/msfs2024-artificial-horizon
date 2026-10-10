---
description: Prepare PR checklists, change acceptance automation, or activate its required status.
status: active
---

* Apply [acceptance tracking](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) when preparing a PR.
* Read [PR acceptance control](../docs/pr-acceptance.md) for supported issue references, checkbox scope, refresh events, and activation order.
* Require all linked issue Acceptance criteria and all PR tasks to be checked independently. Do not compare wording or require copied text. Run the acceptance workflow on `ubuntu-latest`.
* Keep privileged acceptance validation on trusted main code; never execute the PR head with status-write permissions.
* Configure the required status only after the workflow reaches main to avoid blocking its bootstrap PR.

Source: accepted dynamic PR checklist control and its simplified scope, 2026-10-10, prepared for [#48](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/48). Live enforcement requires activation after merge.
