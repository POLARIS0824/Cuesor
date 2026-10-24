# Verify color schemes

Launch a fresh `colors-check` run using the [verification skill](../SKILL.md). Native color dialogs must be available on the interactive Windows desktop.

## Exercise selection and editing

Run the automated journey.

```nu
pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId colors-check -Feature Colors
```

Review all six selectors in `actions.jsonl` and the saved state.

- `配色1：经典`
- `配色2：珊瑚`
- `配色3：暮紫`
- `配色4：极光`
- `配色5：柔粉`
- `配色6：暖砂`

Each selection must apply its saved Chinese and English pair. The journey opens `自定义中文输入光标颜色` and `自定义英文输入光标颜色`, confirms colors through the production dialog, and checks independent persistence after restart.

## Check cancellation and reset

Require a canceled picker to retain its previous saved value and preview. Wait for the modal dialog to close before interacting with other editors.

Change the active scheme before testing `重置当前配色`. After reset, require its recommended pair and preservation of another edited scheme. Reset is disabled when the active pair already matches its recommendation.

## Check the final state

Read saved settings independently. Require the selected scheme and both colors after restart. Keep the action log, verdict, PNG, UI Automation state, and settings snapshots together.

The automated journey covers the common dialog's production confirmation and cancellation messages. It does not prove arbitrary manual picker gestures or actual terminal cursor colors. Use the [cursor journey](cursor-indication-sessions.md) for visible color application. Always finish with Cleanup.
