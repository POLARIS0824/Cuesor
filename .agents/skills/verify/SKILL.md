---
name: verify
description: Drive Cuesor settings and tray on Windows, inspect saved state, and verify Windows Terminal cursor indication. Use after changing desktop behavior or reproducing a connection or diagnostic issue.
---

# Verify Cuesor

Cuesor has a GPUI Kit settings window and a native tray. Keep `Cuesor.exe` beside its required backend, `cuesor-engine.exe`. Read the [feature index](features/README.md) and the relevant journey before testing.

## Prepare an isolated run

Use an interactive Windows desktop with PowerShell 7 and the build prerequisites in the project README. Run these commands from the repository root in Nushell.

The driver needs access to Explorer's notification area. A restricted desktop sandbox can fail at tray creation before the controller starts. If Launch times out waiting for runtime status, inspect its failure PNG and UI Automation capture for a startup error. After Cleanup, retry with permitted access to the interactive desktop when the failure is caused by sandbox isolation.

```nu
pwsh -NoProfile -File tools/build-desktop.ps1
pwsh -NoProfile -File tools/verify.ps1 -Action Launch -RunId appearance-check
```

Choose a new RunId for each run. `Launch` defaults to `target/release/Cuesor.exe`. To verify a portable package, pass `-Executable ./artifacts/Cuesor-portable/Cuesor.exe`.

`Launch` gives the child process an isolated `CUESOR_DATA_DIR`, waits for its visible settings controls and controller, and records a baseline screenshot and state. Binary hashes identify what ran. The checkout HEAD identifies the source tree, so retain a successful build log separately when claiming a source build.

Exit every daily instance, including an older release, through its own tray before launching. Never kill a daily instance to make verification pass. The JSON override does not isolate Windows singletons, the login-startup registry value, or terminal discovery. Run one verifier at a time. Workers can attach to any accessible Windows Terminal console for the current user.

## Check the owned instance

Run Doctor before interaction and whenever the window or process state is uncertain.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Doctor -RunId appearance-check
```

Doctor checks executable hashes, GUI identity, visible controls, tray ownership, saved schemes, the controller, and reported workers. It does not change settings. Zero sessions can be valid for a settings test. Proving attachment requires the exact test console root client PID and a ready worker. See the cursor journey for shell-launcher PID handling. Proving cursor color requires observing the physical terminal.

## Run a journey

Run one mapped feature per RunId so the verdict has one scope.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId appearance-check -Feature Appearance
```

Choose the feature that matches the change.

- `Appearance` selects light, dark, and system appearance, then checks persistence after restart.
- `Colors` edits both colors in all six schemes, cancels a picker, resets the active scheme, and checks restart persistence.
- `Pause` toggles the setting and restores its previous value.
- `Tray` checks close-to-hide, second-launch restore, and native pause and resume handlers.

Use the feature guides for keyboard traversal, geometry, actual Explorer gestures, terminal colors, UAC, and login. The four automated journeys do not prove those routes.

Capture another observation of the owned visible window when needed.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId appearance-check
```

## Inspect evidence and clean up

Read `artifacts/verify/<RunId>/evidence/actions.jsonl` alongside the PNG, UI Automation, settings, runtime, and window snapshots. Inspect images yourself. A successful verdict or ready worker does not prove rendered terminal colors.

Record manual actions in `manual-observations.md` with the entry point, time, observed result, and remaining limits. Missing input methods, terminals, privileges, or desktop access leave the affected route unverified. Keep failed captures and reports.

Always finish with Cleanup, including after a failed attempt.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Cleanup -RunId appearance-check
```

Cleanup targets only processes whose recorded PID, creation time, and executable still match. It archives scratch data and preserves evidence. Check `cleanup.json` for `owned_processes_stopped`, `data_removed`, and `evidence_preserved`. Normal-exit proof also requires an empty `forced_pids` list.

The driver attempts cleanup after Launch or Drive failures. Doctor leaves the instance available for diagnosis. Retain `verdict.json`, `cleanup.json`, screenshots, logs, and archived `final-data/`. Do not delete the evidence directory during teardown.
