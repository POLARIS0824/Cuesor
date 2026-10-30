# Cuesor for Obsidian

Cuesor shows the current input method state in the Obsidian editor through the CodeMirror cursor color, a floating badge next to the caret, or both.

The plugin requires the Cuesor desktop app. IME state arrives over the local WebSocket bridge at `127.0.0.1:23333`. The browser DOM cannot expose the real IME mode, so the plugin does not guess from composition events.

## Features

- **Indicator Modes**:
  - **Cursor color only**: The CodeMirror cursor changes color based on the input mode (orange for Chinese, blue for English).
  - **Cursor badge only**: A minimal floating chip follows the text caret and displays `中` or `EN`.
  - **Both**: Color and badge active simultaneously.
  - **Disabled**: Standard Obsidian cursor appearance.
- **Colors**: Follow the active Cuesor color scheme, or set custom Chinese and English colors.
- **Tracking**: Handles multi-line editing, scrolling out of view, split editor leaves, and window blur.
- **Reconnect**: Retries the local bridge every 5 seconds when Cuesor is not running.

## Installation

1. Install and start the Cuesor desktop app. The plugin reads IME state from it.
2. Copy `main.js`, `manifest.json`, and `styles.css` to `<Vault>/.obsidian/plugins/cuesor-obsidian/`.
3. Open **Settings -> Community plugins** in Obsidian and enable **Cuesor**.
4. Adjust preferences in the **Cuesor** settings tab.

If Cuesor is installed somewhere unusual, set the executable path in the plugin settings.

## Building from source

```bash
cd obsidian-plugin
npm install
npm run build
```
