# Testing guide

Select shared checks by the changed boundary, then add the affected feature's acceptance scenarios.
Record results and acceptance progress in the PR, a related GitHub issue, or a durable linked validation record.

## Find the required procedure

| Task | Source | Use condition |
| --- | --- | --- |
| Select checks or reuse evidence | [Validation scope](#validation-by-change-scope) and [evidence reuse](#evidence-reuse) | Every change |
| Run application checks | [Automated checks](automated.md) | Application code, dependencies, or build changes |
| Check startup, telemetry, or lifecycle | [Runtime checks](runtime.md) | Affected server, source, or browser lifecycle |
| Validate presentation | [Shared EFI validation](pfd.md) | Rendering, geometry, lifecycle, or asset integration |
| Validate one instrument | [Feature directory](../efi/index.md#features) | Use that page's Testing section |
| Validate speed | [IAS/GS](../efi/airspeed.md#testing), [TAS](../efi/true-airspeed.md#testing), [ranges/trend](../efi/airspeed-indications.md#testing) | Changed speed indication |
| Validate altitude | [Altitude/BARO](../efi/altitude.md#testing), [alerts](../efi/selected-altitude.md#testing), [VSI](../efi/vertical-speed.md#testing), [trend](../efi/altitude-trend.md#testing) | Changed right-side indication |
| Validate heading | [Heading](../efi/heading.md#testing) | Heading or full-panel acceptance |
| Validate phones or supply controlled inputs | [Device procedure and fixtures](devices.md) | Device behavior or feature acceptance |
| Record test results or acceptance evidence | [Evidence records](evidence.md#test-evidence-records) | Automated tests, manual checks, reported acceptance, and reused results |
| Change acceptance control | [Validator checks](../pr-acceptance.md#validate-changes-to-the-control) | Acceptance parser or workflow changes |

## Validation by change scope

Use the changed behavior and acceptance criteria to select local checks:

| Change | Required checks | Scope |
| --- | --- | --- |
| Documentation or context only | Diff, links, anchors, terminology, preserved requirements, and claims against current sources | No local application build or browser run |
| Session hooks or other development tooling | Changed behavior and failure paths | Application checks only when application inputs change |
| PR acceptance control | [Validator suite](../pr-acceptance.md#validate-changes-to-the-control), permissions, and trusted-code boundary | Acceptance tooling |
| Rust or browser application code, dependencies, or build configuration | [Automated checks](automated.md#automated-checks), affected [runtime checks](runtime.md), and [feature scenarios](../efi/index.md#features) | Affected application behavior |
| Browser JavaScript, EFI display, geometry, lifecycle, or module/asset integration | Both [desktop browser scripts](pfd.md#responsive-efi-layout-and-desktop-checks), screenshot tests, and inspection of the affected captures | Additional EFI evidence |
| Mobile behavior or feature acceptance | Applicable real iOS Safari and Android Chrome checks | Device evidence separate from desktop results |

Apply these scope rules:

* Use every applicable row when a change crosses boundaries.
* Rebuild the complete application after changes to application code or build inputs, including embedded Web assets.
* Documentation-only changes require no application build or application screenshot tests.
* Local validation remains mandatory. Application builds and test suites do not run in GitHub Actions.
* Preserve the acceptance workflow's triggers, required `Acceptance criteria` status, and unrelated repository protections. The `Build & Test` status must not be required.
* Report checks prevented by explicit task restrictions.
* Leave the corresponding acceptance criteria unconfirmed when required evidence is missing.

### Evidence reuse

Check existing evidence before repeating a validation run:

1. Identify the tested revision and environment.
2. Compare relevant source, fixtures, dependencies, commands, configuration, and the tested artifact.
3. Explain why later changes do or do not affect the recorded result.
4. Select the action below.

| Evidence condition | Action |
| --- | --- |
| Relevant inputs and environment unchanged | Reuse the recorded result |
| Prose-only correction outside application inputs | Retain applicable application evidence |
| New summary of the same unchanged result | No additional run |
| Relevant changes, failures, incomplete logs, or unresolved concerns | Run affected checks again |
| Staged refactor without final integration evidence | Run required final checks on the combined outcome |

> Stage results alone do not establish final integration. A recorded final run remains reusable under the same conditions.
