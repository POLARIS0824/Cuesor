# Appearance, window, and keyboard

Users select system, light, or dark appearance, retain a usable normal window size across launches, and operate settings from the keyboard.

## Sub-features

- `appearance-select` saves system, light, and dark appearance through the visible selector.
- `appearance-restart` restores the selected appearance after restart.
- `window-size` restores the manually resized normal window.
- `window-extremes` preserves normal dimensions through maximize/minimize and keeps controls reachable at minimum size.
- `keyboard-navigation` moves focus with Tab/Shift+Tab and activates controls with Space/Enter.

## How to get to it (user POV)

- Use the settings window's upper-right `外观：系统`, `外观：浅色`, and `外观：深色` controls.
- Drag the window border, maximize/minimize from the title bar, close settings, or restart the desktop app.
- Press Tab/Shift+Tab through settings and use Space/Enter on focused buttons and switches.

## Driving it with tools/verify.ps1 and the Windows desktop

Preconditions:

- Launch and Doctor with RunId `appearance-check` using the index. Record the Windows display scaling for geometry proof.

- **Appearance.** Run `pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId appearance-check -Feature Appearance`. Require UI Automation actions for all three named controls, settings values `light`, `dark`, and `system`, corresponding screenshots, and a final dark selection restored after restart. Use `actions.jsonl`, settings/UIA snapshots, PNGs, and `verdict.json` together.
- **System behavior.** Select `外观：系统`, then change Windows appearance in a disposable desktop session and observe the app follow. Record both system action and resulting app appearance in `manual-observations.md`; the helper only selects and persists system mode.
- **Manual window entry points.** While the owned run is active, drag its border to resize toward the 760 × 520 logical-pixel minimum. Check both color editor buttons, all schemes, appearance controls, startup switch, and reset remain reachable, scrolling where needed. Maximize and minimize through the title bar, restore normal mode, and require the captured `window.json` to retain normal dimensions. Run `pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId appearance-check` before and after, and record the actual gestures.
- **Keyboard.** In the owned settings window, use Tab and Shift+Tab to traverse the visible controls. Use Space on `启用光标颜色指示` and Enter on a color-scheme button, verify visible state plus saved settings, and restore the original values. Record focused accessible names in order and each activation result. Run Snapshot after restoration.
- **Resize and restart regression.** First run `pwsh -NoProfile -File tools/verify.ps1 -Action Cleanup -RunId appearance-check`. With that desktop instance fully stopped, run `pwsh -NoProfile -File tools/desktop-check.ps1 -Executable ./target/release/TerminalIME.exe -KeepArtifacts` as a separate run. Require its window resize/restart and maximize/minimize preservation assertions. Copy `artifacts/desktop-check-verdict.json` to `artifacts/verify/appearance-check/evidence/window-verdict.json` and retain the unique `artifacts/desktop-check-*` directory named by its `DataDirectory` result. That retained directory contains the actual screenshots and saved window state; link it in `manual-observations.md`.

## Gotchas

- `Appearance` automates theme selection and restart persistence only. Resize, maximize/minimize, system-theme changes, and the full keyboard journey need separate observations.
- The helper has no general restart or ownership-refresh action. The existing desktop regression script owns and cleans up its separate processes; run it only after the new helper's Cleanup.
- `desktop-check.ps1` uses production Win32 resize/maximize/minimize messages. Its regression verdict does not replace the manual border/title-bar entry points. It also discovers accessible test consoles, so apply the same dedicated-session boundary.
- Window bounds are physical pixels in some Win32 observations and logical pixels in settings. Record DPI before comparing them.
- The current default normal size is 880 × 600 logical pixels. Maximized or minimized dimensions are not the saved normal size.
- Screenshots alone cannot prove keyboard focus order or restart persistence. Retain focused names, saved values, and post-restart observations.
