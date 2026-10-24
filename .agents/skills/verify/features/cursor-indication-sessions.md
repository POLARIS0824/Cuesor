# Cursor indication and automatic sessions

Terminal IME attaches Windows Terminal sessions automatically and changes their cursor color with the active Chinese or English input state. Pausing or leaving the terminal restores its theme cursor color.

## Sub-features

- `attach-existing` initializes sessions that were open before the desktop app.
- `attach-new` initializes sessions opened after the desktop app.
- `color-follow` applies the active scheme's Chinese and English colors while typing normally.
- `color-focus` restores theme color when the terminal loses foreground focus or input state cannot be confirmed.
- `color-panes` follows the window's active input mode across attached panes.
- `color-pause` pauses and resumes through settings and the tray, including visible cursor effects.
- `session-exit` removes a closed root shell's worker without stopping other sessions.

## How to get to it (user POV)

- Open a Nushell or PowerShell session inside Windows Terminal, then open Terminal IME.
- Open another Windows Terminal tab or pane while Terminal IME is already running.
- Type into an ordinary editable prompt, change the Microsoft Pinyin Chinese/English state, or select the US English keyboard.
- Use `启用光标颜色指示` in settings or `暂停指示` or `恢复指示` in the tray menu.

## Driving it with tools/verify.ps1 and Windows Terminal

Preconditions:

- Use a newly created physical Windows Terminal test session on a dedicated interactive Windows desktop. Record its shell PID and `WT_SESSION`; keep unrelated terminal content out of captured evidence.
- Microsoft Pinyin and US English keyboard are available. GUI and test Terminal have compatible privileges. Open one test shell before Launch, then Launch and Doctor with RunId `cursor-check` as described in the index.

- **Existing session.** In the fresh Nu test shell, record `$nu.pid` and `$env.WT_SESSION`. Run `pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId cursor-check`. Require that shell PID in the captured runtime sessions, a live corresponding worker, and the settings session list. Record any runtime errors.
- **New session.** Open a second Nu or PowerShell tab/pane after Launch. Record its PID, wait for initialization, then repeat Snapshot. Require the new session in runtime and the visible session count. An empty discovery result leaves this journey unverified.
- **Real cursor.** Read the active Chinese/English colors from the snapshot. Focus the test prompt, select Chinese mode, type a harmless unexecuted string, then change to English and continue editing. Capture both actions and cursor states, and compare with the selected colors. Move focus to settings and observe theme restoration in the Terminal. Snapshot captures settings only; capture the fresh test Terminal separately.
- **Panes and shell exit.** Switch focus between the two test panes, change input state, and observe shared window mode rather than independent pane preferences. Exit one root test shell. Repeat Snapshot and require its session/worker to disappear while the remaining shell stays attached.
- **Settings pause.** Run `pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId cursor-check -Feature Pause`. Its action log and settings snapshots prove UI toggle and restoration. Separately pause via `启用光标颜色指示` while watching the test Terminal, require theme color, then resume and require color following. Repeat through the real tray `暂停指示` or `恢复指示` entry. Record each entry point and result in `manual-observations.md`.

## Gotchas

- `Pause` does not drive or capture a physical Terminal cursor. Report visible color coverage separately from its automated verdict.
- GUI Snapshot does not include other windows. Settings previews, ready workers, and `runtime.json.sessions` are supporting evidence, not visible color proof.
- Automatic discovery can attach every accessible Windows Terminal console for this user. A scratch JSON directory does not limit that discovery.
- The product shares the active input mode within one Windows Terminal window. Independent per-pane language preference is outside this behavior.
- Programs that actively set cursor color may overwrite the indication. Keep that separate from ordinary prompt verification.
- An elevated Terminal with an ordinary GUI must be diagnosed through the permissions feature; a missing session must not be relabeled a passing color check.
