# Verify appearance, window size, and keyboard operation

Use a fresh `appearance-check` run from the [verification skill](../SKILL.md). Record Windows display scaling before comparing physical bounds with saved logical dimensions.

## Select and restore appearance

Run the automated journey.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId appearance-check -Feature Appearance
```

Require actions for `外观：浅色`, `外观：深色`, and `外观：系统`. Check matching saved values and screenshots. After a real exit and restart, require the selected dark control and saved `dark` value.

To test system following, select system appearance and change Windows appearance in a disposable desktop session. Observe the app change with Windows. The automated journey selects system mode but does not change the operating-system theme.

## Resize and restore the window

Use the real border and title-bar controls. Resize toward the 760 × 520 logical-pixel minimum. Check both color editors, all schemes, appearance, startup, reset, and the bottom status. Scroll when the content needs more space.

Maximize, minimize, and restore. Check that `window.json` retains normal dimensions. Exit and reopen the app, then compare its normal bounds. The default normal size is 880 × 600 logical pixels.

For the existing geometry regression, first clean up the owned run. Then run the separate helper.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Cleanup -RunId appearance-check
pwsh -NoProfile -File tools/desktop-check.ps1 -Executable ./target/release/Cuesor.exe -KeepArtifacts
```

Retain `artifacts/desktop-check-verdict.json` and the unique directory named by its `DataDirectory`. That helper uses production window messages. Its result does not replace evidence of actual border and title-bar actions.

## Operate controls with the keyboard

Traverse controls with Tab and Shift+Tab. Use Space on `启用光标颜色指示` and Enter on a scheme button. Record focused accessible names, visible results, and saved settings. Restore the original values afterward.

Capture snapshots before and after manual actions. Keep geometry, keyboard, and system-theme observations separate from the automated Appearance verdict. Screenshots alone do not prove focus order or persistence.
