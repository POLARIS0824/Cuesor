# Cuesor for Obsidian

Cuesor Obsidian Plugin enhances the Obsidian editor by showing real-time input method (IME) status via CodeMirror cursor color, a floating badge next to the text caret, or both.

## Features

- **Indicator Modes**:
  - **Cursor color only**: The CodeMirror cursor changes color dynamically based on input mode (e.g. orange for Chinese, blue for English).
  - **Cursor badge only**: A minimal floating chip follows the text caret and displays `中` or `EN`.
  - **Both**: Color and badge active simultaneously.
  - **Disabled**: Standard Obsidian cursor appearance.
- **Theme Support**: Customizable colors for both Light and Dark Obsidian themes.
- **Precision Tracking**: Automatically handles multi-line editing, scrolling out of view, split editor leaves, and window blur events.
- **Cuesor Desktop Sync**: Syncs with the system-wide Cuesor engine via a local WebSocket bridge, with graceful fallback to DOM composition detection when running standalone.

## Installation

1. Copy `main.js`, `manifest.json`, and `styles.css` to your vault directory:
   `<Vault>/.obsidian/plugins/cuesor-obsidian/`
2. Open **Settings -> Community plugins** in Obsidian and enable **Cuesor**.
3. Adjust preferences in the **Cuesor** settings tab.

## Building from source

```bash
cd obsidian-plugin
npm install
npm run build
```
