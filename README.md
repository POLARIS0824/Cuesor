# Cuesor

Cuesor 由 cue（提示）和 cursor（光标）组合而成，用光标颜色提示 Windows Terminal 当前的中文或英文输入状态。默认中文为橙色，英文为蓝色，支持六组可编辑配色、明暗外观、托盘运行和登录启动。

![Cuesor 设置界面参考，包含光标配色、外观和登录启动设置](docs/images/settings-light.png)

## 构建并启动

需要 Windows、Windows Terminal、Rust 和 PowerShell 7。MSVC 工具链需要 C++ Build Tools 与 Windows SDK，GNU 工具链需要完整 GCC。

在项目根目录运行以下 Nushell 命令。

```nu
pwsh -NoProfile -File tools/build-desktop.ps1
./target/release/Cuesor.exe
```

程序自动接入已有和新开的终端会话，无需修改 shell 配置。将 `Cuesor.exe` 与后台组件 `cuesor-engine.exe` 保留在同一目录。

运行 `pwsh -NoProfile -File tools/package-desktop.ps1` 可生成 `artifacts/Cuesor-portable/` 便携目录。

## 调整设置

在设置窗口和托盘中完成日常操作。

- 选择配色，用 **自定义颜色** 修改当前组的中英文颜色。 **重置当前配色** 只影响当前组。
- 在右上角选择 **系统**、**浅色** 或 **深色** 外观。
- 开启 **登录 Windows 时启动**，让程序在登录后驻留托盘。
- 关闭设置窗口后程序继续运行。双击托盘图标打开设置，右键菜单可暂停、恢复或退出。
- 若提示管理员终端需要授权，点击 **以管理员身份重新启动** 并确认 Windows 授权提示。

设置和窗口尺寸自动保存到 `%LOCALAPPDATA%\Cuesor\`。

## 从更名前的版本升级

1. 在旧版设置中关闭登录启动，再从托盘退出旧版。不要同时运行新旧版本。
2. 如需保留偏好，将原配置目录中的 `settings.json` 和 `window.json` 复制到 `%LOCALAPPDATA%\Cuesor\`。
3. 启动 Cuesor，按需重新开启登录启动。

Cuesor 使用新的配置目录和登录启动项，不会自动迁移旧设置。

## 使用范围

同一 Windows Terminal 窗口中的分屏共用当前输入状态。窗口失去前台焦点或无法识别输入状态时，光标恢复终端主题色。

历史验证覆盖微软拼音和美式英文键盘。其他输入法、WSL、SSH 和全部 TUI 尚未全面验证。主动修改光标颜色的程序可能覆盖指示色。

[CLI 接入与诊断](docs/CLI.md)提供独立使用方法。设计约定见 [界面参考](docs/GUI_REDESIGN.md)，本轮检查范围见 [桌面版验证](docs/DESKTOP_VALIDATION.md)。[CLI 历史性能数据](docs/VALIDATION.md)不代表当前版本的性能。
