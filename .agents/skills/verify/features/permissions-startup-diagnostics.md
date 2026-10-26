# Verify permissions, startup, and diagnostics

These journeys use real Windows UAC, login, registry state, and terminal sessions. They are outside the four automated settings journeys. Use a disposable Windows user or VM for elevation and login tests. Record process identity, privilege, executable path, and data location before each action.

## Diagnose blocked terminals

Start a fresh `diagnostics-check` run using the [verification skill](../SKILL.md). Open an elevated test Windows Terminal while the GUI is ordinary. Export the report from Nushell.

```nu
./target/release/cuesor-engine.exe diagnose ./artifacts/verify/diagnostics-check/evidence/diagnose.json
pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId diagnostics-check
```

Require the elevated terminal PID in `elevated_terminal_pids` and the visible `管理员终端需要授权` state. Keep command output and exit status.

## Cancel and approve elevation

Clean up the helper's run before the manual UAC journey. Launch Cuesor normally in the disposable session. Choose `以管理员身份重新启动`, cancel, and require the original instance to remain usable with an actionable error. Repeat and approve.

Record the replacement GUI PID, elevated token, initialized shell and worker, and actual terminal colors. For the normal data directory, inspect the current GUI from Nushell.

```nu
let gui_pid = (open ($env.LOCALAPPDATA | path join Cuesor runtime.json) | get controller_pid)
pwsh -NoProfile -File tools/check-terminal-access.ps1 -ProcessId $gui_pid
```

That helper reads `%LOCALAPPDATA%\Cuesor\runtime.json`. It cannot inspect a scratch-data run unchanged. Establish ownership again after elevation before interacting or cleaning up.

## Verify login startup

Record the exact prior existence, type, and contents of `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Cuesor`. `CUESOR_DATA_DIR` does not isolate that value.

Enable `登录 Windows 时启动`. Require saved `auto_start=true` and a Run command containing the quoted current GUI path plus `--background`. Disable it and require value removal and `auto_start=false`. Restore the original registry and settings state after testing.

For actual login coverage, enable startup, sign out, and sign in. Require a live owned GUI and controller, a tray icon, hidden settings, and attachment to a new test terminal. A manual `--background` launch does not prove login startup. Record results before restoring state.

## Check diagnostic reports

Retain both report types in the run's evidence directory.

```nu
./target/release/cuesor-engine.exe discover ./artifacts/verify/diagnostics-check/evidence/discover.json
./target/release/cuesor-engine.exe diagnose ./artifacts/verify/diagnostics-check/evidence/diagnose.json
```

`discover` returns a session array containing root client PIDs. `diagnose` returns sessions plus `elevated_terminal_pids`. Compare those PIDs with actual processes. These commands inspect terminals without starting indication workers. The GUI owns worker startup and shutdown. Use the cursor journey for editing, input switching, focus, and visible color checks.

Keep observations and reports after teardown. Exit only recorded test processes. Restore startup state and close the owned elevated app before leaving the disposable session. Missing UAC, login, or terminal observations remain unverified.
