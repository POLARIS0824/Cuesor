# Terminal IME

面向 Windows Terminal 的 Rust 光标颜色指示器，提供 GPUI Kit 设置窗口、系统托盘和不依赖 shell 配置的自动接入。默认配色将中文输入显示为 Radix Orange `#F76B15`，英文输入显示为 Radix Blue `#0090FF`；无法确认状态或窗口失去前台焦点时恢复终端主题颜色。

同一 Windows Terminal 窗口内，已接入的会话跟随当前正在输入的中／英状态；不保存各分屏独立的语言偏好。当前桌面版为 0.2.0，原有 CLI 诊断方式仍保留。

## 桌面版使用

构建后双击 `target/release/TerminalIME.exe`，或者使用 `artifacts/TerminalIME-portable/TerminalIME.exe`。同目录的 `terminal-ime.exe` 是后台组件，需要一起保留；日常使用打开大写名称的 GUI 入口。

- 六个快捷按钮分别保存一组中文／英文配色，点击即可同时应用两种颜色。选择一组后，用上方两侧的“自定义颜色”分别编辑该组的中文和英文颜色，六组均可自定义并独立保存。
- 窗口右上角可选择跟随系统、浅色或深色外观，选择会自动保存。界面采用 Radix 浅色和深色色板；开关使用 GPUI Kit 的 Base 交互控件。按 Tab／Shift+Tab 移动焦点，按 Space 或 Enter 操作控件。
- “启用光标颜色指示”控制暂停／恢复；“重置当前配色”只恢复当前组的推荐颜色，保留其他组的修改。
- 默认窗口为 880 × 600 逻辑像素。手动调整的普通窗口尺寸会自动保存，下次启动时恢复；最大化和最小化不会覆盖它。
- 关闭设置窗口会继续在托盘运行。托盘图标可能位于任务栏的“显示隐藏图标”区域；双击图标打开设置，右键菜单按当前状态显示“暂停指示”或“恢复指示”，也可退出。
- 打开 Windows Terminal 后，后台会自动发现已有和新建的终端会话，数秒内接入。不需要再执行 Nu 命令或编辑 Nu、PowerShell 配置。
- 若终端以管理员身份运行，普通权限的软件无法接入，设置会显示“以管理员身份重新启动”。点击后在 Windows 的 UAC 提示中选择“是”。程序默认使用普通权限。
- 登录启动默认关闭，可通过设置开启；登录时只驻留托盘，不弹出设置窗口。移动程序目录后应重新关闭／开启此选项以更新路径。

六组配色、当前选择、启用状态、外观和登录启动偏好保存在 `%LOCALAPPDATA%/TerminalIME/settings.json`；窗口尺寸单独保存在同目录的 `window.json`。登录启动只操作当前用户 Run 项中的 `TerminalIME` 值。`runtime.json` 和有错误时的 `last-worker-error.json` 用于诊断，不记录终端输入正文。

设置窗口按 [GPUI Kit Design Guides](https://gpui-kit.com/docs/design-guides/) 重新设计，并为浅色、深色外观注册 [Radix Colors](https://www.radix-ui.com/colors) 语义主题；终端预览中的颜色块是用户选择的实际光标颜色。已有设置文件无需手动迁移，新增的 `appearance` 默认为 `system`。组件对应关系与本轮验证见 [界面重设计记录](docs/GUI_REDESIGN.md)。

遇到一直等待会话时，可用同目录的 `terminal-ime.exe diagnose REPORT.json` 导出发现结果；`elevated_terminal_pids` 列出当前权限无法接入的管理员终端。开发检查 `tools/check-terminal-access.ps1` 比较实际进程令牌与界面运行记录，避免将权限问题显示成没有终端。

若设置目录将文件移动报告为跨卷，保存会回退到 Windows 允许复制的移动模式。已有文件因共享读取而拒绝替换时，保存会使用带备份的 Windows 文件替换接口；替换完成后清理备份。`tools/check-settings-storage.ps1` 使用独立测试文件验证真实设置目录的新建与覆盖，不改动现有配色。

桌面运行由 GUI、一个自动发现进程以及每个 console 的轻量 worker 组成。设置变化由 worker 重新加载，不需要重启 shell。应用退出时 worker 会尽力恢复主题色并退出；关闭某个根 shell 时其 worker 也会退出。旧 Nu 接入脚本调用的 CLI worker 会在桌面版运行时让出控制权，避免争抢颜色。

## 桌面版构建与验证

已在本机 Windows GNU 工具链上构建。首次构建需要联网获取依赖，GPUI 的首次 release 构建明显慢于原来只有 CLI 的项目。

```powershell
./tools/build-desktop.ps1
./tools/package-desktop.ps1
./tools/desktop-check.ps1
```

打包脚本生成可直接运行的 `artifacts/TerminalIME-portable/` 目录。桌面检查使用独立测试设置，验证实际窗口启动、Common Controls 清单、托盘、单实例、关闭隐藏、暂停保存及退出清理；执行前先从托盘退出日常实例。

构建脚本优先使用 `GPUI_FXC_PATH` 或 PATH 中的 Windows SDK `fxc.exe`。没有 SDK 时，会构建项目内的 `tools/shader-compiler.rs`，使用 Windows 自带的 `D3DCompileFromFile` 编译 GPUI 发布构建的着色器；它只实现 GPUI 使用的参数子集。GNU 构建还需要完整 GCC 中的 `windres` 和 Windows 导入库。MSVC 构建仍应安装官方 Windows SDK／C++ 构建工具。本机未改动 GPUI 上游源码或全局 Rust 配置。

GNU 构建脚本还使用项目内的 `dlltool-wrapper.rs` 隔离每次导入库调用的临时文件，避免共享前缀导致 stub 丢失；辅助工具只在构建期间运行。

代码检查：

```powershell
cargo fmt --all --check
cargo test -p terminal-ime --locked
cargo clippy --workspace --locked --all-targets -j 1 -- -D warnings
```

## 桌面版资源与边界

本机 release 版隐藏设置窗口、接入 3 个测试会话的 10 秒低活动采样中，GUI 私有内存约 **96.27 MiB**，自动发现进程约 **3.39 MiB**，全部进程合计约 **109.40 MiB**，整机 CPU 约 **0.107%**。未统计 GPU 显存，采样没有固定前台活动，不代表最高开销。

GUI 常驻有额外内存开销。桌面版自动接入不在 Nu 启动配置里执行同步命令；下方约 22 ms 的启动增量是旧版 Nu 脚本接入的历史测量，不能直接用于桌面版。当前尚未重新测量桌面版开启／关闭时 Nu 冷启动和颜色响应 P95。

本机自动接入已验证 Nu 和 Windows PowerShell，以及在后台启动后新开的 Nu。原版颜色引擎和日常 Atuin 操作已有人工确认；桌面版的完整人工验收、实际 Windows 注销／登录、管理员模式、标签拖出和其他第三方输入法的证据另见 [桌面版验证记录](docs/DESKTOP_VALIDATION.md)。

## 原有 CLI 接入与诊断

以下方式用于独立 CLI 调试。桌面版正常使用时无需执行这些 Nu 接入命令；CLI 的 `status`／`stop` 管理旧会话 worker，桌面版使用托盘退出来停止。

### CLI 构建与接入

需要 Windows、Rust 工具链、Windows Terminal 和 Nushell。在本项目目录构建：

```powershell
cargo build --release --locked
```

在 Windows Terminal 的 Nu 窗口执行以下命令，仅接入当前会话：

```nu
source /absolute/path/to/Terminal_IME/integrations/nushell.nu
```

如果希望每次打开 Nu 自动接入，可自行将这一行放到 `config.nu` 顶层；用 `$nu.config-path` 查看配置路径。接入脚本只在 Windows Terminal 内的交互式 Nu 中启动，重复加载不会重复保留 worker。移动项目后需要更新 `source` 路径。

本项目没有自动修改用户的 Nu 配置或 Windows Terminal profile。已完成的日常 Atuin 验证使用的是当前会话接入。

## 查看、停止与恢复

以下命令在本项目目录、需要管理的那个 Nu 会话中执行：

```nu
./target/release/terminal-ime.exe status
./target/release/terminal-ime.exe stop
./target/release/terminal-ime.exe reset
```

`status` 输出 `{"running":true}` 或 `{"running":false}`。`start` 异步创建 worker，刚启动时初始化可能尚未完成。`stop` 等待当前会话 worker 退出，并尽力恢复主题色；`reset` 只恢复颜色，仍在运行的 worker 后续可能重新应用指示色。

停止后重新 `source` 接入脚本即可启动。取消持久接入时，移除 `config.nu` 中的 `source` 行，并在已运行的会话中执行 `stop`。绑定的 Nu 进程退出后，worker 会自行退出。

自定义颜色时先停止原 worker，再手动启动：

```nu
./target/release/terminal-ime.exe stop
./target/release/terminal-ime.exe start --shell-pid $nu.pid --cn-color '#FF7800' --en-color '#0077FF'
```

颜色只接受六位十六进制格式。重复 `start` 不会覆盖已运行实例的选项；默认自动接入颜色定义在 Rust 的 `Colors::default()` 中。

## 原版 CLI 性能与兼容性

在本机实际 Nu 环境及配置下，交替测量关闭／启用各 30 次，Nu 启动时间中位数从 **273.00 ms** 增至 **294.68 ms**，增加约 **21.68 ms**；P95 从 **364.74 ms** 增至 **421.76 ms**，增加约 **57.03 ms**。最初的中位数增量 ≤10 ms、P95 增量 ≤30 ms 目标尚未达到。

当前保留同步接入短命启动器的方式，长期工作由原生 worker 承担，不添加 Nu 后台任务。低活动采样中，每个 worker 私有内存约 **3.4 MiB**，10 个合计约 **33.5 MiB**。前台按约 100 ms 间隔采样，非前台按约 500 ms 间隔采样；采样间隔不等于已经测得的切换响应延迟。

原版已在 Nu 0.116.0、Atuin 18.24.1 上验证 Ctrl+R／上方向键搜索、取消／选择历史记录后继续编辑，以及中英文颜色跟随。接入脚本不修改 Nu 的 hooks 或 keybindings。完整方法、CPU 数据、证据位置及未验收项见 [原版验证记录](docs/VALIDATION.md)。

## 诊断

先停止后台 worker，避免与探针同时控制颜色。在本项目目录执行：

```nu
./target/release/terminal-ime.exe stop
mkdir artifacts
./target/release/terminal-ime.exe probe
./target/release/terminal-ime.exe probe --follow --watch 20 --report artifacts/manual.jsonl
./target/release/terminal-ime.exe probe --color cn
./target/release/terminal-ime.exe probe --color en
./target/release/terminal-ime.exe reset
```

`probe` 默认只读取状态；`--follow` 持续应用采样颜色，正常结束时恢复主题色；`--color` 写入一次指定颜色。探针占用当前前台命令，适合验证输入法和颜色通路，不代表后台 Nu 编辑兼容性。若被手动中断后颜色未恢复，可执行 `reset`。

也可以 `source tools/probe.nu`，按提示完成 20 秒验证并用 `y`／`n` 记录视觉结果。需要后台日志时，在停止原 worker 后运行：

```nu
./target/release/terminal-ime.exe start --shell-pid $nu.pid --log artifacts/worker.jsonl
```

日志记录进程、窗口、输入法状态和颜色写入等元数据，不记录输入正文。默认不写日志；`artifacts/` 和 `target/` 已被 Git 忽略。

## 实现与边界

- 输入法读取参考父项目 Rust 实现：前台／焦点窗口、`ImmGetDefaultIMEWnd` 和 `WM_IME_CONTROL`。单次消息查询超时为 25 ms，并在查询后核对焦点；失败归为 Unknown。
- 每个 WT 会话使用独立 worker，通过用户身份、完整性级别和 `WT_SESSION` 区分实例。worker 绑定启动它的 Nu PID 与创建时间，处理 Nu 退出和显式停止。
- 启动器通过 `CreateProcessW` 创建无窗口 worker，禁止继承句柄，避免长期持有 Nu 输出管道导致启动阻塞。
- worker 附着 Nu 的 console，通过 `CONOUT$` 写入 OSC 12／112，只在状态变化时更新颜色，不读取终端输入，不改变光标形状或共享 console 模式。
- 产品运行不需要 Python，不注入按键，不添加服务或计划任务。CLI worker 使用启动它的 shell 权限；桌面版只在用户点击管理员模式时请求 UAC 授权。

本机已验证 Windows Terminal 1.24.12741.0、微软拼音和美式英文键盘。其他输入法、跨权限组合、拖出标签页、全部 TUI，以及 WSL／SSH 场景尚未全面验证。主动重设光标颜色的程序可能覆盖指示色；本版不持续争抢颜色控制权。

## 开发验证

```powershell
cargo fmt --check
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
```

首次构建未缓存依赖时，去掉 `--offline`。`tools/session-check.nu` 和 `tools/atuin-check.nu` 用于隔离会话与实际配置检查。启动基准及 1／5／10 worker 资源采样可在 PowerShell 中执行：

```powershell
./tools/benchmark.ps1 -Samples 30
./tools/load-check.ps1
```

这两个测试脚本会打开并清理自己的 Windows Terminal 测试会话，要求本机存在名为 `Windows PowerShell` 的 profile。启动基准还需要 Python：辅助程序只向测试脚本创建的 Nu 会话写入固定计时命令，测量首个提示符后的 `$nu.startup-time`，不属于产品运行路径。重复基准会覆盖 `artifacts/benchmark/` 内同名测量记录。
