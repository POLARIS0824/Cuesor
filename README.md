# Terminal IME

面向 Windows Terminal 和 Nushell 的 Rust 光标颜色指示器。微软拼音中文状态使用橙色 `#FF7800`，英文状态和美式英文键盘使用蓝色 `#0077FF`；无法确认状态或窗口失去前台焦点时恢复终端主题颜色。

首版已实现后台运行和 Nu 接入，并通过本机颜色、多标签／分屏、日常 Atuin 使用验证。同一 Windows Terminal 窗口内，已接入的会话跟随当前正在输入的中／英状态；不保存各分屏独立的语言偏好。

## 构建与接入

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

## 性能与兼容性

在本机实际 Nu 环境及配置下，交替测量关闭／启用各 30 次，Nu 启动时间中位数从 **273.00 ms** 增至 **294.68 ms**，增加约 **21.68 ms**；P95 从 **364.74 ms** 增至 **421.76 ms**，增加约 **57.03 ms**。最初的中位数增量 ≤10 ms、P95 增量 ≤30 ms 目标尚未达到。

当前保留同步接入短命启动器的方式，长期工作由原生 worker 承担，不添加 Nu 后台任务。低活动采样中，每个 worker 私有内存约 **3.4 MiB**，10 个合计约 **33.5 MiB**。前台按约 100 ms 间隔采样，非前台按约 500 ms 间隔采样；采样间隔不等于已经测得的切换响应延迟。

已在 Nu 0.116.0、Atuin 18.24.1 上验证 Ctrl+R／上方向键搜索、取消／选择历史记录后继续编辑，以及中英文颜色跟随。接入脚本不修改 Nu 的 hooks 或 keybindings。完整方法、CPU 数据、证据位置及未验收项见 [验证记录](docs/VALIDATION.md)。

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
- 产品运行不需要 Python、不注入按键、不添加服务或计划任务。worker 使用启动它的 Nu 权限，不请求提权。

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
