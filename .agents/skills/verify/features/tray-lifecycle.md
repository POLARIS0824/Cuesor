# Verify tray and desktop lifecycle

Use a fresh `tray-check` run from the [verification skill](../SKILL.md). Explorer and its notification area must be available for actual icon and menu gestures.

## Check hiding and reopening

Run the automated journey.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId tray-check -Feature Tray
```

Require closing settings to hide the window while the owned GUI and controller remain alive. A second `Cuesor.exe` launch must exit and restore the original GUI window and PID.

The journey also sends pause and resume through the owned native tray handler and checks saved enabled state. Those production messages do not prove Explorer icon gestures or menu selection.

## Use the real tray icon

Close settings manually. Locate this run's Cuesor icon, including the hidden-icon area if needed. Double-click it and require the recorded settings window to reopen. Repeat with the real context-menu `打开设置` entry.

Choose `暂停指示` and `恢复指示` from the real menu. Record each action and saved result, then restore the original state. Use the [cursor journey](cursor-indication-sessions.md) to observe visible terminal pause effects.

Keep menu captures and manual observations with a GUI snapshot. Finding the hidden `Cuesor.Tray` message window proves handler ownership, not an Explorer icon or its gestures.

## Exit and retain evidence

Record GUI, controller, and worker PIDs before choosing the actual tray exit entry. Require those processes to end. Observe cursor restoration in the fresh test terminal when that route is in scope.

Run Cleanup and inspect its ownership and preservation report. Closing settings and exiting from the tray are different lifecycle actions. Cleanup may call the production exit handler, but that does not prove the Explorer menu gesture.
