# 使用独立 CLI 接入与诊断

桌面版自动接入终端。仅在独立使用 CLI 或排查问题时，执行本页命令。以下命令使用 Nushell，均在项目根目录运行。

## 接入当前 Nu 会话

先从托盘退出桌面版，再构建 CLI。

```nu
cargo build --release --locked
```

在 Windows Terminal 的 Nu 会话中加载接入脚本。

```nu
source ./integrations/nushell.nu
```

需要随 Nu 启动时，将脚本的绝对路径写入 `config.nu` 顶层的 `source` 命令。用 `$nu.config-path` 查看配置文件路径。移动项目后，更新脚本路径。

## 查看或停止当前会话

在需要管理的 Nu 会话中执行以下命令。

```nu
./target/release/terminal-ime.exe status
./target/release/terminal-ime.exe stop
./target/release/terminal-ime.exe reset
```

`status` 显示 worker 是否已初始化。`stop` 等待 worker 退出并尝试恢复主题色。`reset` 只恢复颜色，仍在运行的 worker 可能再次应用指示色。

停止后重新加载接入脚本即可启动。取消随 Nu 启动时，删除 `config.nu` 中的 `source` 行，再对已运行的会话执行 `stop`。Nu 退出后，其 worker 也会退出。

## 指定当前会话的颜色

先停止原 worker，再用六位十六进制颜色启动。

```nu
./target/release/terminal-ime.exe stop
./target/release/terminal-ime.exe start --shell-pid $nu.pid --cn-color '#FF7800' --en-color '#0077FF'
```

重复 `start` 不会覆盖运行中的选项。

## 导出桌面版连接诊断

桌面版一直等待会话时，导出诊断报告。

```nu
./target/release/terminal-ime.exe diagnose REPORT.json
```

报告中的 `elevated_terminal_pids` 列出当前权限无法接入的管理员终端。

## 检查输入法与颜色

先退出桌面版并停止 CLI worker，避免多个进程同时控制颜色。

```nu
./target/release/terminal-ime.exe stop
mkdir artifacts
./target/release/terminal-ime.exe probe
./target/release/terminal-ime.exe probe --follow --watch 20 --report artifacts/manual.jsonl
./target/release/terminal-ime.exe probe --color cn
./target/release/terminal-ime.exe probe --color en
./target/release/terminal-ime.exe reset
```

`probe` 默认只读取状态。`--follow` 持续应用采样颜色，正常结束时恢复主题色。`--color` 写入一次指定颜色。若中断后颜色未恢复，执行 `reset`。

需要后台日志时，在停止原 worker 后运行以下命令。

```nu
./target/release/terminal-ime.exe start --shell-pid $nu.pid --log artifacts/worker.jsonl
```

日志记录进程、窗口、输入法状态和颜色写入，不记录输入正文。默认不写日志。

查看 [原版 CLI 验证记录](VALIDATION.md) 或返回 [桌面版使用说明](../README.md)。
