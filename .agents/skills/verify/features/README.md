# Cuesor verification journeys

Use the [verification skill](../SKILL.md) to build, launch, inspect, and clean up the owned instance. Pick the guide that covers the entry point being changed.

## Choose a guide

- [Cursor indication and sessions](cursor-indication-sessions.md) covers attachment, visible colors, focus, panes, pause, and shell exit.
- [Color schemes](color-schemes.md) covers six independent color pairs, picker confirmation and cancellation, reset, and persistence.
- [Appearance, window, and keyboard](appearance-window-keyboard.md) covers theme selection, size restoration, window controls, and keyboard operation.
- [Tray lifecycle](tray-lifecycle.md) covers hide, reopen, single-instance behavior, pause, resume, and exit.
- [Permissions, startup, and diagnostics](permissions-startup-diagnostics.md) covers elevation, login startup, and diagnostic reports.

## Establish ownership and coverage

Exit daily instances before testing. Use one fresh RunId and retain both executable hashes, the checkout revision, and the successful build log. Keep `Cuesor.exe` beside `cuesor-engine.exe`.

`CUESOR_DATA_DIR` isolates JSON files. It does not isolate terminal consoles, Windows named objects, or login startup. Use a dedicated test session for terminal and registry journeys. Capture only the newly created test terminal's contents.

Pair each action with its observable result. Read saved state independently and check the UI after restart when persistence matters. Settings previews and runtime records support cursor tests but do not replace physical terminal observations.

Keep failed results as failed evidence. End every attempt with the mapped Cleanup command and inspect its report. A verdict covers only the routes the driver actually exercised.

## Interpret historical evidence

Earlier runs reproduced `0x80070005` while replacing a settings file held by a shared reader. The source now uses a backup-backed `ReplaceFileW` fallback. The backend snapshot regression and later appearance and color journeys cover that fix.

The prior failure captures remain in `artifacts/verify/colors-portable-02/evidence/` and `artifacts/verify/appearance-source-01/evidence/`. Prior closure captures are in `artifacts/verify/ui-after-appearance-01/evidence/` and `artifacts/verify/ui-after-colors-01/evidence/`. These paths identify historical runs, not proof of a later build. Retain a new failure's temporary settings and reports instead of retrying until a verdict passes.
