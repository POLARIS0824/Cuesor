# GPUI Kit 界面重设计

2026-10-22，Asia/Hong_Kong。设计依据为 [GPUI Kit Design Guides](https://gpui-kit.com/docs/design-guides/)，组件来自 [GPUI Kit](https://gpui-kit.com/) 0.8.0。

## 界面与组件

设置窗口围绕“启用指示 → 选择中英文光标颜色 → 确认终端连接”组织。配色预览使用并排、等宽的编辑区域；连接信息、登录启动和托盘说明使用分隔线组织，取消原来的营销标题、彩色卡片和手绘开关。

| 界面职责 | 实际使用 |
| --- | --- |
| 普通操作、自定义颜色、恢复默认 | Kit `Button`，包含禁用、悬停、按下和键盘焦点状态 |
| 即时生效设置 | Kit `Switch`，带可访问名称 |
| 外观选择 | Kit `ButtonGroup`，持续显示当前选项 |
| 内容滚动 | Kit `Scrollable`，滚动条位于内容区域外缘 |
| 颜色、字体、圆角 | Kit `Theme` 和 `ActiveTheme` 的语义令牌 |
| 图标 | Kit 自带的 Lucide 图标资源 |
| 窗口与焦点层 | Kit `application`、`init`、`open_window` 和 `Root` |

布局沿用 GPUI 的 rem 间距与字号辅助方法。仅平台窗口大小使用逻辑像素；色板与预览里的 RGB 值属于实际光标颜色数据。预览背景、文本、分隔线、错误和权限状态均读取主题色。保留系统窗口标题栏、原生颜色选择器和原生 UAC 重启流程。

默认窗口为 880 × 600 逻辑像素，最小为 760 × 520。通过收紧间距、压缩预览和连接信息，减少默认高度。窗口缩小时保留顶部开关、配色操作和固定底栏，内容区域按需滚动。Tab／Shift+Tab 移动焦点，Space／Enter 激活库控件。

## 设置与兼容性

`settings.json` 新增 `appearance`，接受 `system`、`light`、`dark`，默认跟随系统；外观改变自动保存。旧设置文件自动补上默认外观，保留原来的颜色、启用状态与登录启动偏好。系统主题变化仅在选择跟随系统时同步。

托盘、自动发现、IME 检测和颜色写入流程保持原有架构。后台停止时清除过期连接展示，明确提示重新打开程序；权限不足、部分会话失败、暂停、等待会话和已连接分别显示对应状态。

`window.json` 单独保存普通窗口的逻辑尺寸。拖动后延迟约 350–550 ms 保存，退出时再写入最后尺寸；最大化、最小化不覆盖普通尺寸。重新启动时按当前屏幕工作区域限制尺寸，避免旧尺寸超出当前显示范围。

## 六组可编辑配色

点击任一组，同时应用它的中文和英文颜色；上方两侧的自定义按钮编辑当前组，各组独立保存。重置只还原当前组。升级时保留原来的两种颜色，并放入第一组。

| 名称 | 中文光标 | 英文光标 | 色板参考 |
| --- | --- | --- | --- |
| 经典 | `#FF7800` | `#0077FF` | 原有橙蓝配色 |
| 珊瑚 | `#E54D2E` | `#12A594` | Radix Tomato / Teal |
| 暮紫 | `#6E56CF` | `#FFC53D` | Radix Violet / Amber |
| 极光 | `#D08770` | `#88C0D0` | Nord Aurora / Frost |
| 柔粉 | `#F38BA8` | `#94E2D5` | Catppuccin Mocha Red / Teal |
| 暖砂 | `#A18072` | `#3E63DD` | Radix Bronze / Indigo |

颜色取自 [Radix Colors 官方色板](https://www.radix-ui.com/colors)、[Nord 官方色板](https://www.nordtheme.com/docs/colors-and-palettes/) 和 [Catppuccin 官方色板](https://catppuccin.com/palette/)。两两组合由本项目选择，目的是区分两种输入状态；极光和柔粉较适合深色终端。它们不会随应用明暗主题改变，用户可按自己的终端背景调整。

## 本轮验证

工具链为 Rust 1.99.1 Windows GNU。以下自动检查已通过：

- `cargo test -p terminal-ime --release --offline --locked -j 1`：12 项测试，包括旧配色迁移、六组颜色独立保存、重置不影响其他组、窗口尺寸保存与显示范围约束。
- `cargo clippy --workspace --release --offline --locked --all-targets -j 1 -- -D warnings`。
- 修改的 PowerShell 工具通过语法解析。

release 构建通过，最终链接没有重复清单警告。GPUI Kit 的平台层已提供 Common Controls v6、asInvoker 和 PerMonitorV2 清单，因此应用只另外嵌入自身图标；界面用到的额外 Lucide 图标通过 `icon_assets!` 选择性打包。

`tools/desktop-check.ps1` 在独立设置目录中启动真实 GUI，使用 Windows UI Automation 的控件名称操作，已通过：

- 自定义颜色确认保存、取消保持原值；逐组编辑全部十二个颜色，完全退出后再切换六组，确认各自的颜色仍然保留。
- 点击配色同时应用两种颜色；重置只影响当前组，推荐颜色未修改时重置按钮禁用。
- 浅色／深色切换、外观自动保存，以及退出后重新启动恢复深色选中状态。
- Space 暂停、Tab 从启用开关移动到中文颜色编辑、托盘恢复。
- 默认窗口下两侧自定义按钮的顶部和尺寸严格相同；默认尺寸下登录启动开关可见。
- 拖动到不同宽高后完全退出再打开，外框宽高与退出前相差不超过 2 个物理像素；最大化和最小化均不覆盖普通窗口尺寸。
- 760 × 520 最小窗口下，六组配色、两侧颜色编辑、外观和底栏保持可见。
- 关闭隐藏、单实例恢复、原生窗口启动、托盘暂停／恢复及退出后后台进程清理。

验证摘要保留在 `artifacts/desktop-check-verdict.json`，便携包校验值保留在 `artifacts/gui-redesign-binaries.json`。截图、临时测试目录、详细日志和编译缓存已按要求清理；需要重现时可重新构建并运行桌面检查脚本。清理前的截图检查确认默认窗口完整展示设置与托盘说明，最小窗口通过内容区域滚动容纳下方设置，图标均正常载入。

桌面检查默认清理本次产生的测试目录和截图；仅在显式传入 `-KeepArtifacts` 时保留它们供视觉检查。测试读取设置文件时允许 Windows 文件替换，避免轮询本身阻塞应用的自动保存。

本轮环境未打开 Windows Terminal 会话，报告 `InitializedSessions=0`、`LegacyHandoffTested=false`；因此此次桌面检查不证明真实终端颜色跟随、CLI 接管或管理员会话接入。未重新执行 UAC 授权、注销／登录启动、跨显示器 DPI 和资源采样。旧版证据见 [历史桌面验证](DESKTOP_VALIDATION.md)，不能视为此次重设计的复测结果。
