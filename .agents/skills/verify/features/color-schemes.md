# Color schemes and customization

Users select one of six Chinese/English cursor-color pairs, customize either color for that scheme, and reset only the selected scheme. Their independent values and selected scheme survive restart.

## Sub-features

- `scheme-select` applies both colors from each of the six schemes.
- `scheme-customize-cn` confirms a Chinese color through the native picker.
- `scheme-customize-en` confirms an English color through the native picker.
- `scheme-cancel` cancels the native picker without changing saved values.
- `scheme-restart` restores edited independent schemes and selection after restart.
- `scheme-reset` restores the current recommendation while preserving other schemes.

## How to get to it (user POV)

- In settings, choose `配色1：经典`, `配色2：珊瑚`, `配色3：暮紫`, `配色4：极光`, `配色5：柔粉`, or `配色6：暖砂`.
- Choose `自定义中文输入光标颜色` or `自定义英文输入光标颜色` above the scheme row, then confirm or cancel the native color dialog.
- Choose `重置当前配色` beside the scheme heading.

## Driving it with tools/verify.ps1

Preconditions:

- Launch and Doctor with RunId `colors-check` using the index. Native color dialogs must be available on the interactive Windows desktop.

- **Full automated journey.** Run `pwsh -NoProfile -File tools/verify.ps1 -Action Drive -RunId colors-check -Feature Colors`. Review `actions.jsonl` and the journey verdict for all six named selectors, both color editors, picker confirmation/cancellation, reset, and restart. Require each scheme selection to apply its saved Chinese/English pair and each edited pair to survive independently.
- **Picker confirmation and cancellation.** Require evidence of opening the production dialog through its accessible editor button, applying its result, and observing saved settings and preview. Require the canceled dialog to preserve the previous value. The harness's production common-dialog messages cover confirm/cancel; arbitrary manual picker gestures remain separate if relevant to the change.
- **Reset and restart.** Require a changed active scheme before reset, recommended values afterward, and preservation of another edited scheme. Require post-restart selected scheme and colors, not only JSON written before exit.
- **Final observation.** Run `pwsh -NoProfile -File tools/verify.ps1 -Action Snapshot -RunId colors-check`. Keep the selected controls, preview PNG, and saved settings alongside the action log and verdict. For actual cursor application, also run the physical Terminal journey in [Cursor indication](./cursor-indication-sessions.md).

## Gotchas

- Each scheme contains two colors. A Chinese-only assertion does not cover selecting or restoring the English color.
- Picker dialogs temporarily disable other editing controls. Wait for the dialog to close and the intended control to become enabled.
- `重置当前配色` is disabled when the current recommendation is unchanged. The journey must first create a changed value.
- Save proof requires an independent settings read and restored UI after restart. A successful click alone is insufficient.
- A passing settings journey proves persisted choices and previews. Visible cursor behavior belongs to the linked Terminal journey.
