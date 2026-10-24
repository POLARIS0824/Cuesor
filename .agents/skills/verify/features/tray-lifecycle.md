# Tray and desktop lifecycle

Closing settings leaves Terminal IME in the tray. Users reopen settings, pause or resume indication, and exit the application without stranded discovery processes or workers.

## Sub-features

- `window-close` hides settings while the tray/controller remain alive.
- `singleton-reopen` restores the original settings window when the executable is opened again.
- `tray-reopen` opens settings from the real tray icon and menu.
- `tray-toggle` pauses/resumes through the real tray menu and saves the enabled state.
- `tray-exit` exits the GUI/controller/workers and restores terminal cursor state.

## How to get to it (user POV)

- Close the settings window using its title-bar close control.
- Open `TerminalIME.exe` again from the same installation.
- Find Terminal IME in the Windows tray or `显示隐藏图标` area. Double click the icon or choose `打开设置` from its context menu.
- Choose `暂停指示` or `恢复指示` or `退出` from the tray context menu.

## Driving it with tools/verify.ps1 and the Windows tray

Preconditions:

- Launch and Doctor with RunId `tray-check` using the index. Explorer and its notification area must be available for actual icon/menu journeys.

- **Window and singleton.** Run `pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId tray-check -Feature Tray`. Require an owned settings window that becomes hidden without GUI exit, a second executable launch that exits, and restoration of the original window/PID.
- **Native tray handler.** In that same journey, require native menu-handler pause/resume actions and both resulting saved enabled states. The helper's tray messages prove production handler behavior. Record the real Explorer interaction separately below.
- **Actual icon entry points.** Close settings manually. Locate only this run's Terminal IME tray icon, including the overflow area if necessary, then double click it. Require the recorded GUI's settings window to appear. Repeat closing, right click the icon, and choose `打开设置`. Run `pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId tray-check` and record each gesture/result in `manual-observations.md`.
- **Actual menu toggle.** Right click that icon and choose `暂停指示` or `恢复指示`, then repeat to restore the original state. Capture menu actions and settings state after each. Also observe a fresh physical test Terminal for visible pause effects using the [cursor journey](./cursor-indication-sessions.md).
- **Exit.** Record owned GUI/controller/worker PIDs before choosing the actual tray `退出` entry. Require each recorded process to terminate and observe theme cursor restoration in the test Terminal. Run `pwsh -NoProfile -File tools/verify.ps1 -Action Cleanup -RunId tray-check` to retain its cleanup record and complete any owned teardown. Confirm evidence survives.

## Gotchas

- `Tray` does not exercise Explorer's icon gestures, overflow discovery, context-menu rendering, or real menu selection. Its verdict must not close those entry points.
- Finding the hidden `TerminalIME.Tray` message window proves an owned handler window, not a visible Explorer icon.
- Multiple data directories share the user/integrity singleton. Validate the original PID before attributing a restored settings window to this run.
- Closing settings differs from the tray `退出` action. Check process and worker state for both.
- Cleanup may use the owned production exit handler. Its success proves teardown, not the actual Explorer exit-menu gesture.
