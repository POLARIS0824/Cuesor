# Verify cursor indication and terminal sessions

Cuesor discovers accessible Windows Terminal consoles and applies the selected Chinese or English cursor color. Use a dedicated Windows test session with Microsoft Pinyin and a US English keyboard. GUI and terminal privileges must match.

## Attach existing and new sessions

Create a physical Windows Terminal test session before launching the app. Record `$nu.pid` and `$env.WT_SESSION` in Nushell. Launch a fresh `cursor-check` run using the [verification skill](../SKILL.md).

Capture runtime state with the existing helper.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId cursor-check
```

Require the exact test shell PID in `runtime.json`, a live corresponding worker, and the visible settings session list. Record errors. Open a second Nu or PowerShell tab after launch and repeat those checks for its PID.

An empty discovery result does not pass attachment. The JSON data override does not restrict discovery to the test terminals.

## Observe the real cursor

Read the active colors from saved settings. Focus the test prompt, select Chinese mode, and type harmless unexecuted text. Switch to English and continue editing. Capture the actions and actual cursor colors in that test terminal.

Move focus to settings and require the terminal's theme color to return. Compare attached panes with the window's active input mode. Cuesor does not keep independent language preferences for each pane.

Exit one root test shell. Require its session and worker to disappear while the other test shell remains attached. Keep unrelated terminal contents out of captures.

## Pause and resume

Use `启用光标颜色指示` while watching the test terminal. Require theme color when paused and indication after resume. Repeat through the real tray `暂停指示` and `恢复指示` entries. Record each route separately.

The automated `Pause` journey checks the setting and restoration only. It does not capture a terminal cursor. A ready worker, settings preview, or GUI snapshot also does not prove visible colors.

If a program actively overwrites cursor color, record that case separately. Diagnose elevated terminals through the [permissions guide](permissions-startup-diagnostics.md). Missing preconditions leave the affected route unverified. Exit owned test shells and clean up the app run after observations.
