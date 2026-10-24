---
name: verify
description: Drive Terminal IME's Windows desktop settings and tray, capture UI and persisted-state evidence, and verify its Windows Terminal integration. Use after changing user-facing behavior or when reproducing a desktop, cursor, or diagnostic journey.
---

# Verify Terminal IME

Read [features/README.md](features/README.md), then the feature being changed. This Rust workspace's primary surface is `TerminalIME.exe`, a GPUI Kit settings window and native tray. `terminal-ime.exe` is its required sibling backend and a separate diagnostic/legacy CLI surface. There is no HTTP port, database, seed account, or authentication.

## Launch

Run from the repository root in an interactive Windows desktop. Commands below are Nushell commands calling the existing PowerShell 7 Windows harness. Ensure `pwsh`, `cargo`, `rustc`, and the GNU or MSVC build prerequisites described in the repository README are available.

```nu
pwsh -NoProfile -File tools/build-desktop.ps1
pwsh -NoProfile -File tools/verify.ps1 -Action Launch -RunId appearance-check
```

`Launch` defaults to `target/release/TerminalIME.exe`. It creates an empty `artifacts/verify/appearance-check/data/`, sets `TERMINAL_IME_DATA_DIR` only for the child process, and lets the real application create default settings and start its controller. Ready requires an owned visible settings window, expected UI Automation controls, six persisted schemes, matching runtime ownership, and a live controller. It emits a doctor report and baseline screenshot/UIA/settings/runtime evidence.

Use a fresh RunId for every run; existing runs are retained. If deliberately verifying an existing portable package, select it explicitly:

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Launch -RunId portable-check -Executable ./artifacts/TerminalIME-portable/TerminalIME.exe
```

The harness records both binary hashes, backend version, and checkout HEAD. HEAD identifies the checkout; hashes identify what actually ran. A prebuilt executable does not establish that current source built successfully. The driver does not build binaries itself.

**Isolation:** `TERMINAL_IME_DATA_DIR` isolates JSON files only. The desktop singleton is scoped by Windows user and integrity level, independently of that directory. The controller also discovers all accessible Windows Terminal consoles. Run one verifier at a time and allow for temporary cursor changes in those terminals. Launch refuses any existing `TerminalIME.exe` or `terminal-ime.exe`; use the daily app's tray Exit before trying again. Never kill an existing instance to make a test pass. Registry login startup and UAC remain real Windows effects and are outside the automated journeys.

Teardown, including after a failed command:

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Cleanup -RunId appearance-check
```

## Doctor

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Doctor -RunId appearance-check
```

This read-only command verifies the recorded GUI PID, start time and executable path; both executable hashes; owned tray HWND; visible expected UIA controls; six schemes; matching `runtime.json.controller_pid`; live controller and reported workers. It reports sessions, permission blocks, and errors. It writes no evidence and changes no settings. Run it first whenever the instance looks wrong. A hidden window is not ready for settings interaction; use the mapped tray journey or clean up.

Zero sessions can be healthy for a settings-only test. Automatic attachment needs the exact test shell PID in runtime, a ready worker, and empty errors. Visible cursor behavior also needs actual terminal observation; Doctor explicitly does not prove that.

## Drive

The deterministic helper reuses the repo's UI Automation and screenshot approach, rather than coordinates. Each command records its action before invoking a production UI control and captures the resulting UI plus saved JSON. Run one mapped journey per run so `verdict.json` represents one feature.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId appearance-check -Feature Appearance
```

Supported journeys:

| Feature | Actual entry points and assertions |
| --- | --- |
| `Appearance` | UIA `外观：浅色`, `外观：深色`, `外观：系统`; selected controls, saved values, light/dark screenshots, and dark selection restored after a real exit/restart. |
| `Colors` | All six `配色N：名称` controls; both custom-color buttons; native color-dialog confirmation/cancel; independent persistence across restart; reset affects only the active scheme. |
| `Pause` | UIA `启用光标颜色指示`; saved toggle and restoration, with screenshots. Cursor restoration is a separate terminal journey. |
| `Tray` | Settings WM_CLOSE, second executable launch, and owned tray WM_COMMAND pause/resume handler. Proves hide/reopen and handler behavior; Explorer tray icon gestures need separate evidence. |

For `Colors`, `Pause`, or `Tray`, start a fresh run with another RunId, then use that ID in the Drive/Cleanup commands. Read its feature map for additional entry points and preconditions. Window resizing, full keyboard traversal, Explorer gestures, actual cursor colors, UAC, and login startup have recipes in the feature maps; the four automated journeys do not cover them by implication.

Capture additional observations of an owned visible instance with:

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId appearance-check
```

## Evidence

Proof survives in `artifacts/verify/<RunId>/evidence/` (Git-ignored):

- `actions.jsonl`: timestamped actions, doctor observations, snapshots, errors, and verdicts.
- Timestamped `.png`, `.uia.json`, `.settings.json`, `.runtime.json`, and optional `.window.json`: owned window screenshots, accessible control names/selection, and independently read saved state.
- `verdict.json`: feature, tested entry point, executable hashes, checkout HEAD, pass result, and explicit coverage limits. It is written only after a successful Drive.
- `final-data/` and `cleanup.json`: retained final JSON/log files, owned-process teardown result, forced PID list, and surviving evidence inventory.
- `stderr-*.log`: GUI startup stderr. Ownership metadata lives in the sibling `state.json`.

Exercise the real user path. Record the action and result together, and verify persistence/other side effects independently. UI setters, unit tests, and an isolated successful entry point do not prove other mapped routes. Use mocks only across an existing production boundary. These journeys have no mock, test-only endpoint, dry-run, or network stub: they start the real app, controller, and workers. They avoid login-startup and elevation controls; when testing those manually, observe and restore actual Windows state. Verify a new dry-run's files/network/process effects before trusting its name.

Inspect screenshots yourself and correlate them with UIA and JSON. A JSON `passed` alone is insufficient evidence of rendering or terminal cursor color. Report unmet terminal/IME/elevation preconditions as failed or unverified, with the attempt and reason; keep the intended journey intact.

The feature index retains the historical settings-save access-denied failure and its current-source closure evidence. Treat it as failed persistence and retain the failure capture and temporary JSON; a passing portable run does not establish that every current-source journey passes.

## Cleanup

Use the exact Cleanup command above. It sends Exit to the tray HWND only after verifying the owning PID, waits for recorded GUI/controller/workers, and uses force only for recorded processes whose PID, creation time and path still match. It archives scratch data before deleting only this run's `data/`. It preserves screenshots, logs, verdicts, ownership metadata, and cleanup report. Launch/Drive failures attempt this cleanup automatically; inspect cleanup errors and rerun it if needed. Doctor failures leave the instance available for diagnosis.

After cleanup, inspect `cleanup.json`: require `owned_processes_stopped`, `data_removed`, and `evidence_preserved` to be true. Normal-exit coverage additionally requires an empty `forced_pids`; forced cleanup is not proof of normal exit. Confirm the captured PNGs, actions, and successful feature verdict still exist. Never delete `evidence/` as part of teardown.

## Helpers

`tools/verify.ps1` is executable through `pwsh -NoProfile -File`; Launch, Doctor, Drive, Snapshot, and Cleanup invocations are shown above. It uses the existing `tools/capture-window.ps1`. The wider existing `tools/desktop-check.ps1 -KeepArtifacts` covers resize/keyboard and more settings assertions in a separate run; see the map for its shared-session and artifact-retention caveats. `tools/check-terminal-access.ps1` reads the daily data directory and is not a doctor for these isolated runs.

Keep the maintained feature map aligned with new user entry points using `/maintain-verification-skill`.
