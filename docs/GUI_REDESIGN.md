# Cuesor 界面参考与历史检查

设置窗口使用 GPUI Kit 0.8.0，集中展示输入指示开关、中英文光标配色和终端连接状态。日常操作见 [README](../README.md)。本页描述界面约定，并保留更名前的检查结果。

## 控件与布局

| 职责 | 实现 |
| --- | --- |
| 普通操作和颜色编辑 | Kit `Button` |
| 即时生效设置 | Kit Base `Switch`，支持键盘操作与可访问名称 |
| 外观选择 | Kit `ButtonGroup`，显示当前选项 |
| 内容滚动 | Kit `Scrollable` |
| 明暗颜色 | Kit `Theme` 注册的 Radix 主题 |
| 图标 | Kit Lucide 资源 |
| 窗口与焦点 | Kit `application`、`init`、`open_window` 和 `Root` |

默认窗口为 880 × 600 逻辑像素，最小为 760 × 520。内容按需滚动，底栏固定。应用绘制的边框为 0.5 逻辑像素。Tab 和 Shift+Tab 移动焦点，Space 和 Enter 激活控件。标题栏、颜色选择器与 UAC 流程使用系统原生界面。

## 设置保存

`settings.json` 保存配色、启用状态、登录启动偏好和 `appearance`。外观取值为 `system`、`light` 或 `dark`，默认跟随系统。只有选择 `system` 时才同步系统主题变化。

`window.json` 单独保存普通窗口的逻辑尺寸。调整尺寸后延迟约 350 至 550 ms 保存，退出时再保存最后尺寸。最大化和最小化不覆盖普通尺寸，重启时按当前屏幕工作区域限制窗口大小。

同一配置目录内的旧格式设置会补齐默认外观，并将原有两种颜色保留到第一组。此格式兼容不迁移更名前的配置目录。Cuesor 默认使用 `%LOCALAPPDATA%\Cuesor\`，可通过 `CUESOR_DATA_DIR` 指定其他目录。

## 六组配色

每组独立保存中英文颜色。选中一组会同时应用两种颜色，自定义和重置均只影响当前组。

| 名称 | 中文光标 | 英文光标 |
| --- | --- | --- |
| 经典 | `#F76B15` | `#0090FF` |
| 珊瑚 | `#E54D2E` | `#12A594` |
| 暮紫 | `#6E56CF` | `#FFC53D` |
| 极光 | `#30A46C` | `#00A2C7` |
| 柔粉 | `#D6409F` | `#86EAD4` |
| 暖砂 | `#A18072` | `#3E63DD` |

默认值取自 Radix 色板，由项目配对。光标颜色不随应用明暗主题变化，已有自定义颜色保持原值。只有重置当前组才恢复该组默认值。

## 2026-10-22 历史检查

以下结果来自更名前的构建，不能视为 Cuesor 的复测。环境为 Rust 1.99.1 Windows GNU。

release 构建、12 项后端测试、workspace `--all-targets` Clippy 和 PowerShell 语法检查通过。测试覆盖旧格式配色加载、六组独立保存、当前组重置、窗口尺寸保存及显示范围约束。

`tools/desktop-check.ps1` 在独立设置目录启动真实 GUI，通过 UI Automation 覆盖十二个颜色的修改与重启保存、确认与取消、外观恢复、键盘暂停、托盘消息处理、关闭隐藏和单实例恢复。窗口重启后的外框尺寸与退出前相差不超过 2 个物理像素，最大化与最小化没有覆盖普通尺寸。

本轮没有 Windows Terminal 会话，报告为 `InitializedSessions=0` 和 `LegacyHandoffTested=false`。因此不证明真实终端颜色跟随、CLI 接管或管理员接入。未重新执行 UAC、注销登录、跨显示器 DPI 或资源采样。临时截图和详细日志随后清理。

## 2026-10-24 历史检查

本轮统一 Radix 明暗主题与六组出厂配色，并修复共享读取时文件替换失败和选中按钮遮盖键盘焦点边框的问题。release 构建、格式检查、Clippy 和 13 项后端测试通过，保存回归检查覆盖新旧快照与临时文件清理。

`tools/verify.ps1` 的 Appearance、Colors、Pause 和 Tray 流程通过。检查覆盖主题、十二个自定义颜色、取消、重置、持久化、重启及托盘消息处理入口。真实菜单截图显示了暂停和恢复条目，但菜单选择没有完成。UIA 未提供可调用项，键盘操作也未满足前台归属条件，不能由消息处理成功推定 Explorer 托盘手势通过。

`tools/desktop-check.ps1 -KeepArtifacts` 通过最小窗口、对齐、键盘、尺寸保存、最大化、最小化、重启恢复和退出检查。本次为 200% 缩放且没有 Windows Terminal 会话，未验证终端实际颜色、UAC 或登录启动。上述记录均早于更名。

## 复查入口

`tools/desktop-check.ps1` 默认清理本次测试目录和截图，`-KeepArtifacts` 保留证据。[项目验证技能](../.agents/skills/verify/SKILL.md)列出 `tools/verify.ps1` 的可运行流程。历史权限和资源结果见 [桌面版验证记录](DESKTOP_VALIDATION.md)。
