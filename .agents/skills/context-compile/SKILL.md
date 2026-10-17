---
name: context-compile
description: Use to turn established project procedures into directly selectable skills.
---

# Compile Project Procedures

## Goal

Make established project procedures directly selectable and executable as skills, preserving their meaning with one maintained source per procedure.

## Steps

1. **Understand the procedure and its intended use.**

* Establish what the procedure achieves, when it applies, and what guides its execution.
* Use the supplied documents and related project decisions and requirements.
* Use the questions below to examine the procedure.

| Question | Identify | Example |
| --- | --- | --- |
| What result does the procedure produce? | Its purpose and expected outcome | A release procedure produces a verified package ready for distribution. |
| When does it apply? | Recognizable situations, prerequisites, and exclusions | Packaging applies to a selected release version, not every development build. |
| What guides its execution? | Actions, choices, dependencies, and completion criteria | The target platform determines packaging steps and required checks. |
| What supports or limits it? | Rationale, sources, exceptions, and unresolved matters | A compatibility decision explains why one target needs a separate package. |

* Add questions when the table does not explain how the procedure achieves its result.
* Continue when the procedure, its scope, and necessary context are understood, with gaps identified.

2. **Determine the skill boundaries and source ownership.**

* Determine what belongs in the skill and where the procedure will be maintained.
* Use the applications and dependencies from Step 1, existing skills, and relevant project conventions.
* Use the table to make these decisions.

| Decision | Consider | Establish |
| --- | --- | --- |
| Skill boundary | Do the actions serve one outcome and share selection conditions? | A coherent scope that agents can recognize from their tasks. |
| Existing skill | Does the procedure extend an existing skill or need a separate one? | The skill to update or create and its relationship to related skills. |
| Procedure ownership | Will the skill own the procedure or be generated from a maintained document? | One source for editing the procedure and the role of each resulting file. |
| Supporting knowledge | What must accompany the instructions, and what can remain in linked sources? | Content needed during execution and conditions for reading additional material. |

* Adapt these choices when another arrangement better preserves the procedure and supports its use.
* For generated skills, determine how to reproduce the result and detect when it no longer matches the source.
* Continue when boundaries, content locations, and source ownership are clear.

3. **Translate the procedure into actionable skill instructions.**

* Express the actions and decisions from Step 1 as instructions using the boundaries and source arrangement from Step 2.
* Use the table to translate source contributions into skill content.

| Source contribution | Skill expression | Example |
| --- | --- | --- |
| Situations and conditions | A name and description that support selection | Select a release-packaging skill when preparing a distributable package. |
| Intended result | A Goal describing the outcome | Produce the selected release package with the required verification. |
| Actions and dependencies | Steps whose results support subsequent work | Identify the target before choosing its packaging procedure. |
| Conditional choices | Decision criteria, questions, or tables | Select signing instructions according to the package target. |
| Supporting knowledge | Context beside the instruction or a link with a reading condition | Read compatibility constraints when choosing supported targets. |
| Completion criteria | Checks of the result and actions for unresolved gaps | Verify package contents and identify checks that could not be completed. |

* Use another form when the listed expressions do not preserve a contribution needed for execution.
* Choose steps and tables to fit the procedure.
* Continue when the SKILL.md expresses the established procedure, with necessary context and unresolved gaps explicit.

4. **Integrate the skill with its sources and entry points.**

* Place the skill from Step 3 in the selected location and implement the source arrangement from Step 2.
* Use the table to integrate the skill with related sources.

| Arrangement | Action |
| --- | --- |
| Skill owns the procedure | Move the instructions into the skill and replace the previous procedure with an appropriate reference. |
| Source document owns the procedure | Generate the skill from the maintained source, preserving the instructions developed in Step 3, and provide a freshness check. |
| Supporting documents remain separate | Retain necessary rationale and knowledge, linking them to the skill where needed. |
| Existing routes reference the procedure | Update affected links and entry points to reach the skill or its supporting knowledge. |

* Address other affected relationships when the table does not cover the integration needs.
* Continue when the skill is accessible, procedure ownership is unambiguous, and related sources retain necessary context.

5. **Check selection, execution, and source consistency.**

* Check that the skill preserves the procedure, supports appropriate selection, and leads to the expected result.
* Compare the integrated skill with the procedure from Step 1 and the boundaries and source arrangement from Step 2.
* Check selection with matching and nonmatching tasks, and execution with a representative matching task.
* Use the table to assess the results and identify corrections.

| Check | Establish | Correction |
| --- | --- | --- |
| Meaning | Actions, conditions, exceptions, and completion criteria remain intact | Restore missing content or correct changes in meaning. |
| Selection | The name and description match intended tasks and distinguish unrelated work | Clarify the description or adjust the skill boundary. |
| Execution | A representative task can reach its expected result without reconstructing missing instructions | Restore necessary actions, decisions, or context from the established procedure. |
| Source consistency | The procedure has one editing location, and generated output can be checked against its source | Remove independent copies or repair generation and freshness checks. |
| References | Required knowledge remains reachable through valid links | Repair paths, anchors, or missing relationships. |

* Add a check when another consequence could affect selection, execution, or maintenance.
* Correct problems and repeat affected checks, keeping unresolved gaps explicit.
* Distinguish findings from text inspection from observed execution, and state what remains untested.
