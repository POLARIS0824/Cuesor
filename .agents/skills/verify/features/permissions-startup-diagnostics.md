# Permissions, startup, and diagnostics

Users diagnose missing terminal connections, request administrator mode when needed, enable tray-only login startup, and use the retained CLI for standalone session diagnostics.

## Sub-features

- `permission-diagnosis` identifies elevated Windows Terminal instances that an ordinary GUI cannot attach.
- `permission-approve` restarts through UAC and initializes compatible terminal sessions.
- `permission-cancel` leaves the original ordinary instance usable after UAC cancellation.
- `startup-toggle` saves the preference and creates/removes the current user's TerminalIME Run value.
- `startup-login` starts in the tray after an actual Windows login.
- `diagnostic-report` exports discovery and permission state without exposing terminal input.
- `cli-session` starts/stops the legacy session worker, restores cursor color, and leaves Nu/Atuin editing intact.

## How to get to it (user POV)

- With an elevated Windows Terminal open, read settings connection status and choose `以管理员身份重新启动`.
- Use `登录 Windows 时启动` in settings, then log out/in to verify actual login behavior.
- In a test Terminal, run `terminal-ime.exe diagnose REPORT.json`, `discover REPORT.json`, or `probe`.
- For standalone CLI use, source `integrations/nushell.nu` in a fresh Windows Terminal Nu session, then use `status`, `stop`, and `reset`.

## Driving it with tools/verify.ps1, existing CLI tools, and Windows

Preconditions:

- These journeys are outside the automated `Appearance`, `Colors`, `Tray`, and `Pause` drivers. Use a disposable Windows user/VM for UAC and actual login tests. Record app/Terminal privileges, executable path, data directory, and process ownership.
- For diagnosis only, Launch and Doctor with RunId `diagnostics-check` using the index. Create a fresh physical test Terminal, with Microsoft Pinyin available for color observations.
- Startup verification needs the original existence, type, and complete contents of `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TerminalIME` recorded for exact restoration. The JSON data-directory override does not isolate this registry value.

- **Permission diagnosis.** Open an elevated test Terminal while the GUI is ordinary. Run `./target/release/terminal-ime.exe diagnose ./artifacts/verify/diagnostics-check/evidence/diagnose.json` from Nushell, then `pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId diagnostics-check`. Require the elevated Terminal PID in `elevated_terminal_pids` and the visible `管理员终端需要授权` state. Record command output and exit code.
- **Approve/cancel UAC.** Cleanup `diagnostics-check` before beginning the disposable session's manual UAC journey. Open the GUI normally there, choose `以管理员身份重新启动`, first cancel, and confirm an actionable error plus continued ordinary app operation. Repeat and approve. Record the replacement GUI PID and actual elevated token, initialized test shell/worker, and visible cursor behavior. In that session's Nushell, run `let gui_pid = (open ($env.LOCALAPPDATA | path join TerminalIME runtime.json) | get controller_pid)` and `pwsh -NoProfile -File tools/check-terminal-access.ps1 -ProcessId $gui_pid`. Require that PID to be the observed replacement GUI; retain the check's JSON and exit code. This tool hardcodes the normal runtime path and cannot doctor a scratch-data instance unchanged.
- **Startup setting.** In that disposable session, read the exact prior Run value, enable `登录 Windows 时启动`, and compare saved `auto_start=true` with a Run command containing the current quoted GUI path plus `--background`. Disable it and require the value's removal plus `auto_start=false`. Record each UI action, saved settings, and independent registry reads. Restore the exact prior value/settings afterward.
- **Actual login.** Enable startup in the disposable session, then sign out and sign in. Require a live owned GUI/controller and tray icon, hidden settings, and automatic attachment of a new test Terminal. Record the fresh login and all results before restoring startup. Starting the app manually with `--background` is supporting evidence only.
- **Read-only CLI diagnostics.** In the fresh test Terminal, run `./target/release/terminal-ime.exe discover ./artifacts/verify/diagnostics-check/evidence/discover.json`, then `./target/release/terminal-ime.exe probe`. Keep each command's output and exit code. Compare discovery shell PIDs and permission PIDs with the actual processes. Plain `probe` samples input/window state; it does not write a cursor color.
- **Standalone CLI integration.** Cleanup the helper's desktop run and exit every manually launched desktop instance through its own tray, verifying the recorded GUI/controller PIDs have ended. In a newly created Nu test Terminal, source `tools/session-check.nu`, then inspect its session JSON and worker log under `artifacts/`. Require `status` to become `{"running":true}`. Exercise normal editing, completion, Ctrl+C, Chinese/English switching, and the actual Atuin Ctrl+R/Up cancel/accept paths where installed. Run `./target/release/terminal-ime.exe stop`, require `status` to become `{"running":false}`, and observe theme cursor restoration. Use `reset` if interrupted color-writing diagnostics leave a color behind. Copy session reports, relevant metadata logs, and observed actions into this run's evidence directory.

## Gotchas

- UAC approval, cancellation, elevation handoff, login, and startup registry changes have no automated proof from the new helper. Missing operator interaction or disposable-session prerequisites leave those paths unverified.
- UAC creates a new process with separate privilege/ownership and may use the normal data directory. Establish its actual identity and data location before driving or cleaning it up.
- `tools/check-terminal-access.ps1` reads `%LOCALAPPDATA%/TerminalIME/runtime.json`; its result must match the recorded GUI PID.
- `status` and `stop` address the current `WT_SESSION` legacy worker. They do not control desktop auto workers. A running desktop causes legacy workers to yield.
- `tools/session-check.nu` and `tools/atuin-check.nu` establish readiness/configuration facts, then request editing checks. Their JSON alone does not prove real editing or Atuin interaction.
- `probe --follow` and `probe --color cn|en|reset` write cursor state. Stop competing workers first and use only the newly created test Terminal. A foreground probe does not prove background Nu editing compatibility.
- Restore startup state and exit the manually recorded elevated instance before leaving the disposable session. Preserve reports and observations after teardown.
