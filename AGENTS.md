# Repository instructions

- Write code, comments, documentation, issues, commits, and PRs in English.
- Before working on a task, read and follow
  [.context/repository-instructions.md](.context/repository-instructions.md).
- Follow [CONTRIBUTING.md](CONTRIBUTING.md) for the issue/branch/PR workflow,
  document responsibilities, and required checks.
- Use [.context/index.md](.context/index.md) to find relevant project knowledge.

## Project context

- Use `context-apply` to read the index and relevant entries before planning,
  implementing, validating, or resuming work; exclude `.context/sessions/`.
- Inspect relevant project sources when knowledge is missing, and state unresolved gaps.
- Check sources and entry status, follow superseded entries to their replacements,
  and do not apply deprecated entries or treat `active` as proof of a claim.
- Apply constraints to the change and its validation, distinguish intent from
  observed behavior, and surface unresolved conflicts.
- Use `context-gather` to save durable session learning and review or repair stored
  knowledge, updating its existing home or linking to canonical sources.
- Keep entry descriptions within three short sentences and bodies in concise,
  single-sentence bullets; require `description` and `status` frontmatter.
- Require `superseded_by`, relative to the entry, for superseded knowledge.
- Keep `.context/index.md` as a table with only `Entry` and `Use when` columns,
  linking each active entry once with paths relative to the index.
- Update the existing index when adding, renaming, replacing, or withdrawing entries;
  exclude session records, withdrawn entries, and the index itself.
- Use `context-import-sessions` for selected saved transcripts or session checkpoints,
  passing loaded material to `context-gather` before marking a checkpoint reviewed.
- Leave missing, incomplete, changed, or report-only checkpoints pending.
- Do not require a full skill sequence or a memory update for every task.
- Keep task progress and execution results in Issues/PRs or the task plan, and
  keep raw chats and secrets out of `.context/`.
