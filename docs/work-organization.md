# Work organization

GitHub issues describe project outcomes and how users can accept them. [CONTRIBUTING](../CONTRIBUTING.md#workflow) defines the local contribution workflow.

## Issue types

| Type | Purpose |
| --- | --- |
| Epic | Describe a large change, its goal, expected result, and delivery dependencies |
| Story | Describe a feature or other result to deliver and how an end user can check it |
| Bug | Describe observed and expected behavior, reproduction, and how an end user can verify the correction |

Stories and bugs describe the work and observable user acceptance. Keep AI agent context, implementation handoffs, plans, and analyses in the local working tree, not in issues. Agents do not create GitHub subtasks for execution. An Epic may group related stories and bugs when useful; verify any relationship that is created.

## Completion and evidence

Confirm each issue's own expected result and acceptance criteria before closure. Record applicable verification and its limits in the PR, related issue, or a durable linked record. Leave unconfirmed criteria open and identify the remaining verification. Keep durable requirements and contracts in their documentation owners.

## Adoption boundaries

[MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) remains the target MVP Epic. Migration of existing issues, labels, and relationships requires separate authorization. Do not rewrite historical or completed issues without an explicit request. Maintain delivery order and dependencies in GitHub without a competing roadmap document.

Check label availability before authorized publication. Planning alone does not authorize publishing or reorganizing issues.
