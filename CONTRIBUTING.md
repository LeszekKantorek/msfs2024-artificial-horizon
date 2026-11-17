# Contributing

## Environment setup

### 1. Install the tools

| Tool | Requirement / purpose |
| --- | --- |
| Windows x64 | Supported development, build, and runtime platform |
| Git | Clone the repository and manage changes |
| Visual Studio C++ Build Tools | MSVC linker and Windows SDK |
| Rust through rustup | Stable MSVC toolchain, rustfmt, and Clippy |
| Node.js 24 with npm/npx | Browser tests and skill installation |
| Python 3 on `PATH` | Session hooks use the `python` command |
| PowerShell | Run the commands below and Windows session hooks |
| Codex | Required only for the agent workflow and its skills/hooks |

> Demo development requires no simulator or SimConnect SDK. The web client requires no npm install or frontend build.

### 2. Prepare the checkout

Skip cloning if you already have a checkout.

```powershell
git clone https://github.com/LeszekKantorek/msfs2024-artificial-horizon.git
cd msfs2024-artificial-horizon
```

Prepare the toolchain under these requirements:

* Run subsequent commands from the repository root.
* Use the Windows x64 MSVC toolchain when installing rustup.

```powershell
rustup default stable
rustup component add rustfmt clippy --toolchain stable
```

The project does not pin a compiler version.

### 3. Prepare agent skills and hooks

The installed context skills match [context-skills commit `73e282d`](https://github.com/LeszekKantorek/context-skills/tree/73e282d8dc244337462c7f649db56f097f749fe6), version `1.1.0`. This identifies their source; it does not establish runtime validation of the skills.

| Skill | Purpose |
| --- | --- |
| `context-retrieve` | Locate verified source passages for a concrete project question |
| `context-preserve` | Preserve decisions and findings with rationale and evidence |
| `context-review-sessions` | Review pending sessions through `context-preserve` |
| `context-map` | Connect tasks and questions to relevant sources |
| `context-maintain` | Improve knowledge coherence and organization |
| `context-diagnose` | Investigate agent mistakes and check corrections |
| `context-compile` | Turn established project procedures into skills |

If the installed copies are missing, install them with:

```powershell
npx --yes skills add https://github.com/LeszekKantorek/context-skills/tree/73e282d8dc244337462c7f649db56f097f749fe6 --agent codex --skill context-retrieve context-preserve context-review-sessions context-map context-maintain context-diagnose context-compile --copy -y
```

The project also includes [procedural-skill-creator](https://github.com/LeszekKantorek/agent-skills/blob/main/skills/procedural-skill-creator/SKILL.md) for creating or revising executable skill procedures:

```powershell
npx --yes skills add LeszekKantorek/agent-skills --agent codex --skill procedural-skill-creator --copy -y
```

Select agent guidance under these requirements:

* Choose the skill that fits the work. No fixed skill sequence is required.
* Follow [AGENTS.md](AGENTS.md#context-selection) for agent work and procedure selection.

#### Session hooks

The checkout retains the [Codex integration from commit `2598a62`](https://github.com/LeszekKantorek/context-skills/tree/2598a6246b8649f7eaea68720201d9dbff1ddd84/integrations/codex), independently of the installed skill revision.
These files are separate from the skill installation command above:

| File | Purpose |
| --- | --- |
| [`.codex/hooks.json`](.codex/hooks.json) | Hook events and commands |
| [`.codex/hooks/context_register_session.py`](.codex/hooks/context_register_session.py) | Register session metadata without reading transcripts |

| Event | Action |
| --- | --- |
| `SessionStart` | Describe the six context skills so the agent can select one for the task |
| `Stop`, `Interrupt`, `PreCompact`, `SessionEnd` | Create or update one metadata row per session in Git-ignored `.context/sessions.log` |

To prepare the hooks:

1. Check the dependencies in PowerShell:

   ```powershell
   git --version
   python --version
   ```

2. Open the repository in the Codex CLI.
3. Open `/hooks` after installation or hook changes.
4. Review and trust the project definitions as described in the [hook documentation](https://learn.chatgpt.com/docs/hooks).

Apply these hook and session-review constraints:

* Treat hook output as session metadata. Hooks do not read transcripts or run reviews automatically.
* Use `context-review-sessions` with `context-preserve` to review session history. The installed skill no longer includes the registry-reading and review-marking helper; it does not update the registry's `reviewed_date` field.

Maintain the session registry and installed integration under these requirements:

* Preserve the UTF-8 CSV columns `session_date`, `session_id`, `path`, and `reviewed_date` in the registry.
* Exclude `.context/sessions.log` and `.context/.sessions.lock` from commits and knowledge retrieval.
* Retain `.context/sessions/` metadata separately without treating it as reviewed. The session tools do not migrate or read that directory.
* For reinstalls, preserve intentional local changes. Use the skill revision in [agent setup](#3-prepare-agent-skills-and-hooks) and the separate integration revision in [session hooks](#session-hooks).
* Update installed skills and `skills-lock.json` together. Preserve installer-generated hashes as source provenance and keep unrelated hooks.

### 4. Check the environment

Check the environment before starting development:

1. Run the [automated checks](docs/testing/automated.md#automated-checks), including the build and tests.
2. Start the demo:

   ```powershell
   cargo run --locked --bin msfs2024-artificial-horizon -- --scenario demo-scenarios/demo-scenario.jsonl
   ```

3. Open [http://127.0.0.1:8080](http://127.0.0.1:8080).
4. Check that the attitude display appears.
5. Stop the server with `Ctrl+C`.

## Change requirements

Apply the [project constraints](AGENTS.md#project-constraints) and the requirements for the changed boundary:

| Changed boundary | Requirement / source |
| --- | --- |
| Dependencies | Commit `Cargo.lock` and exclude dependencies without distribution rights |
| Material architecture choice | Record the decision in `docs/adr/` under the [decision procedure](.agents/skills/project-decide-architecture/SKILL.md) |
| Tests | Check behavior independently of implementation details under [test design](docs/testing/evidence.md#select-evidence-by-failure-risk) |
| Runtime lifecycle | Preserve errors, source-loss handling, reconnects, and cancellation cleanup under [HTTP startup checks](docs/testing/runtime.md#http-startup-checks) |
| Telemetry | Update compatible fixtures and consumers with the [wire contract](docs/telemetry-contract.md) |
| Documentation | Distinguish planned behavior, implementation, and confirmed device acceptance under the [writing guidance](#writing-guidance) |

Before delivery, satisfy these validation and evidence requirements:

* Apply the [validation scope matrix](docs/testing/index.md#validation-by-change-scope) and [evidence reuse rules](docs/testing/index.md#evidence-reuse).
* Run required local validation. GitHub Actions does not compile the application or run its test suites.
* Record the tested revision, environment, observations, results, and unavailable checks in the PR, related issue, or durable linked validation record.
* Track completion through the [acceptance procedure](#acceptance-criteria-and-additional-verification).
* Confirm that required checks pass and acceptance criteria have supporting evidence before marking work complete.

## Workflow

Follow the [work organization model](docs/work-organization.md) for issue types and acceptance:

1. Read the issue when one exists. Establish the requested outcome and user acceptance expectations.
2. Check current sources, decisions, prerequisites, and code. Report consequential gaps before dependent work.
3. Create a branch and working tree from current `main` as described in [additional working trees](#additional-working-trees).
4. Deliver the requested outcome, update affected documentation and contracts, and select checks through the [validation scope matrix](docs/testing/index.md#validation-by-change-scope).
5. Follow [PR preparation](#prepare-a-pull-request) for the requested delivery.

### Additional working trees

For an issue, use `<number>-<description>` for the branch and working tree. From the main checkout, replace the placeholder with the issue number and a short description:

```powershell
git worktree add -b <number>-<description> .worktrees/<number>-<description> main
```

Work in the new tree. Store plans and analyses for the work in `.plans/<number>-<description>/` within that tree. For work without an issue, use a descriptive name in place of the number and description.

Planning files may be committed on a working branch. Remove all tracked entries under `.plans/`, including placeholders, from the final tree of every PR targeting `main`. An absent directory satisfies this rule; the [acceptance control](docs/pr-acceptance.md#planning-files) checks it. Preserve durable requirements in their documentation owners.

Git ignores `.worktrees/` in the main checkout, but `git clean -fdx` can delete ignored working trees and local changes. After completing work, remove the working tree from the main checkout:

```powershell
git worktree remove .worktrees/<number>-<description>
```

Removing the working tree does not remove its branch.

### Prepare a pull request

Use [project-prepare-pr](.agents/skills/project-prepare-pr/SKILL.md) for local descriptions or authorized publication under these contribution requirements:

* Describe what changed, why, and important design choices or consequences.
* Link the relevant issue when one exists. Use a closing keyword only when the PR completes that issue's criteria.
* Issue linkage and acceptance checklists remain optional for work without an issue.
* Record applicable validation evidence in the PR, a related issue, or a durable linked record. No fixed test-status section or command list is required.

### Acceptance criteria and additional verification

Verify and record acceptance in this order:

1. Select required validation from the change scope.
2. Record confirming results in the PR, a related issue, or a durable linked record, including the tested commit and relevant environment.
3. If an issue uses an acceptance checklist, place its task items under `Acceptance criteria`. Mark them checked only after required validation confirms them.
4. Leave supplied unconfirmed criteria unchecked. Record remaining verification and expected results in the PR, related issue, or linked record.
5. Keep supplied checklists accurate. Unchecked criteria in a closing-linked issue and any visible unchecked PR task block the acceptance status.

Confirm each issue's own result and acceptance criteria before closure. Record remaining verification when evidence is incomplete. Change issue status or publish evidence only within authorized GitHub work.

Apply these merge and status constraints:

* Draft PRs can retain incomplete acceptance. Complete required acceptance before merge.
* Perform required local validation even when the acceptance status succeeds.
* Use the [acceptance control reference](docs/pr-acceptance.md#checklist-scope) for parser rules and [status meanings](docs/pr-acceptance.md#understand-the-result) for blocking conditions.

### Promote agreed findings

Use confirmed findings to define an agreed follow-up. Preserve durable requirements in their documentation owners and distinguish confirmed findings from hypotheses. Draft or publish an issue only within authorized scope, using the [work organization model](docs/work-organization.md).

## Project guidance

Apply these contribution and documentation requirements:

* Apply the [change requirements](#change-requirements) and [acceptance procedure](#acceptance-criteria-and-additional-verification) before submitting a change.
* Follow the [writing guidance](#writing-guidance) for documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Follow [AGENTS.md](AGENTS.md) for agent work and the [setup instructions](#3-prepare-agent-skills-and-hooks) for skills and hooks.

## Writing guidance

Write project text under these requirements:

* Apply [technical-writing-rules](.agents/skills/technical-writing-rules/SKILL.md) for English technical language and accuracy within its scope.
* Write concise, impersonal English with consistent terms across project text, including agent instructions and context entries.
* Describe results, decisions, and procedures without personal narration such as "the owner", "owner-assisted", "the user reported", or "the assistant verified". Use wording such as "The configuration selects..." or "The decision requires...".
* Retain technical ownership terms when they describe resource management or responsibilities.
* Use the [project terminology](docs/efi/index.md#terminology): EFI for the complete instrument, artificial horizon for attitude indication. Keep technical identifiers consistent with their consumers.
* Preserve requirements, units, limits, and uncertainty. Distinguish planned behavior, implementation, and confirmed acceptance.
* Keep supplied accepted plan originals local and verbatim, including their original language. Preserve accepted requirements and decisions in local plans.
* Put feature requirements, presentation, implementation details, and tests on the owning feature page. Keep shared procedures in the testing guide.
* Link canonical public documentation rather than agent context. Check moved content, links, anchors, and Markdown paragraph boundaries.
* Use [project-write-content](.agents/skills/project-write-content/SKILL.md) for Markdown structure and examples, including when writing is part of implementation. Explicit project conventions take precedence over conflicting general defaults.

### Documentation maintenance

Maintain documentation under these requirements:

* Organize content for retrieval and understanding. Shortening text is not an independent goal.
* Describe current requirements and behavior directly. Omit implementation chronicles and accounts of document edits from current reference documentation.
* Retain the rationale for current decisions. Mark accepted planned behavior separately from existing implementation; unresolved claims remain unresolved.
* Before moving or shortening content, compare individual conditions, exceptions, values, and obligation levels. A requirement must not become a description or optional example.
* Before removing source text, verify where every requirement, constraint, and caveat remains in canonical documentation.
* Before restoring a historical requirement, check later accepted decisions and its current owner. Use Git history to resolve discrepancies.
* Identify incoming links before moving content and repair affected paths and anchors. Required link changes do not justify retaining content under the wrong owner.
* For test or acceptance claims, follow [historical evidence](docs/testing/evidence.md#historical-evidence-and-current-claims) and the [accepted-value policy](docs/testing/evidence.md#accepted-values-and-test-evidence). Preserve their conditions and scoped limits; do not copy these policies into writing guidance.
