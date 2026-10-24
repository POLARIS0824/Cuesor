# Terminal IME verification map

Read this index, then the matching feature file before driving Terminal IME. Each file lists the entry points and observable results required for that feature. Report coverage per sub-feature and entry point. A passing helper journey proves only the actions it actually drove.

## Baseline and ownership

- Run commands from the repository root on an interactive Windows desktop. PowerShell is used for the Win32/UI Automation helper; these external commands can be invoked from Nushell.
- Build the current checkout using the parent skill's Launch instructions. Keep `TerminalIME.exe` beside its matching `terminal-ime.exe`.
- Start a fresh run with `pwsh -NoProfile -File tools/verify.ps1 -Action Launch -RunId feature-check -Executable ./target/release/TerminalIME.exe`, then run `pwsh -NoProfile -File tools/verify.ps1 -Action Doctor -RunId feature-check`.
- Use a distinct RunId for each proof journey. Substitute that same ID in every command for that run. Launch may instead name an existing portable GUI executable; record which build was used.
- Require the helper's owned GUI, expected executable, accessible settings controls, and owned runtime status. Run Doctor again whenever the window, process, or state looks wrong.
- `TERMINAL_IME_DATA_DIR` isolates JSON files only. The desktop singleton and console workers share the Windows user/integrity scope. Refuse a conflicting desktop instance and arrange a dedicated Windows test session for terminal-color work. A second data directory does not create an independent desktop instance.
- Keep ordinary verification away from startup and UAC controls. Those manual journeys need their own preconditions and restoration, described in [Permissions, startup, and diagnostics](./permissions-startup-diagnostics.md).

## Driving and evidence

- Drive supported journeys using `pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId feature-check -Feature Appearance`. Other supported feature values are `Colors`, `Tray`, and `Pause`.
- Capture an additional owned-window observation with `pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId feature-check`.
- Evidence lives at `artifacts/verify/<RunId>/evidence/`. Keep `actions.jsonl`, snapshot UI Automation JSON, PNGs, settings/runtime snapshots, `verdict.json`, and `cleanup.json`. The adjacent `state.json` records ownership and is not proof of behavior by itself.
- For manual steps, add `manual-observations.md` to the same evidence directory. Record the sub-feature, entry point, exact action or command, time, observed result, and remaining failures. Retain command stdout/stderr and exit status beside relevant reports.
- A settings preview or runtime record alone does not prove visible terminal color. Capture the action and resulting cursor in a newly created physical Windows Terminal test session. Capture only that test session's content.
- Missing terminals, permissions, input methods, seed state, or automation capabilities leave the affected journey failed or unverified. Record the attempted action and unmet precondition; retain the original required journey.
- Finish every successful or failed attempt with `pwsh -NoProfile -File tools/verify.ps1 -Action Cleanup -RunId feature-check`. Confirm evidence still exists after cleanup. End manually created sessions through their own shell exit and exit only GUI instances whose identity was recorded for that run.

## Features

- [Cursor indication and automatic sessions](./cursor-indication-sessions.md) covers actual color following, existing/new sessions, focus, panes, and pause effects. `Pause` automates only the settings toggle and restoration.
- [Color schemes and customization](./color-schemes.md) covers all six schemes, both native color editors, cancel, current-scheme reset, and restart persistence. Use `Colors`.
- [Appearance, window, and keyboard](./appearance-window-keyboard.md) covers theme selection plus manual geometry and keyboard paths. Use `Appearance` for theme selection and restart persistence.
- [Tray and desktop lifecycle](./tray-lifecycle.md) covers close/hide, second-launch restore, tray actions, and exit. Use `Tray` for the documented window and native-handler subset.
- [Permissions, startup, and diagnostics](./permissions-startup-diagnostics.md) covers UAC, login startup, diagnostic reports, and the retained standalone CLI integration. These journeys use manual actions and existing repo tools.

## Observed save failure

Verification has reproduced `保存失败： Access is denied. (0x80070005)` during repeated settings changes, in both the existing portable package and a fresh source build. The intended settings remain in `settings.<GUI-PID>.tmp` while `settings.json` and the UI retain the previous values. This is failed persistence, even when the clicked control receives focus. The shared-reader replacement failure has since been reproduced through the backend and fixed at the file replacement boundary with a backup-backed ReplaceFileW fallback. The current source passes the shared-reader snapshot regression and the Appearance and Colors journeys. Do not attribute future save failures to missed clicks or retry until a journey appears green. Retain the failure screenshot, actions, and `final-data/` files, and report the failed entry point. Local reproductions include `artifacts/verify/colors-portable-02/evidence/` and `artifacts/verify/appearance-source-01/evidence/`; historical successful settings runs do not close these failures.


Current closure evidence is retained under artifacts/verify/ui-after-appearance-01/evidence/, ui-after-colors-01/evidence/, and .audit/ui-review-fixes/backend-tests.log. Historical failure artifacts remain failed baseline evidence.
