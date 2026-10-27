---
description: Locate author responsibilities, acceptance-control rules, and checks for workflow changes.
status: active
---

| Situation | Source |
| --- | --- |
| Prepare a PR and maintain issue acceptance evidence | [project-prepare-pr](../.agents/skills/project-prepare-pr/SKILL.md); [author responsibilities](../CONTRIBUTING.md#acceptance-criteria-and-additional-verification) define acceptance tracking |
| Diagnose rejected references or unchecked tasks | [Issue references](../docs/pr-acceptance.md#issue-references), [checklist scope](../docs/pr-acceptance.md#checklist-scope) |
| Diagnose status refresh or API failures | [Status meanings](../docs/pr-acceptance.md#understand-the-result), [refresh events](../docs/pr-acceptance.md#validation-triggers), [API recovery](../docs/pr-acceptance.md#api-failures-and-recovery) |
| Activate the required check | [Activation order](../docs/pr-acceptance.md#activate-merge-blocking) |
| Change the acceptance workflow | [Validator checks](../docs/pr-acceptance.md#validate-changes-to-the-control), [trust boundary](../docs/pr-acceptance.md#trust-boundary) |

Run test suites and builds locally. Evidence belongs to the issue or a linked record; PR checklists are optional.
