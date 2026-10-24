# 使用 Cuesor CLI 接入与诊断

桌面版自动接入 Windows Terminal。独立使用后台程序或排查问题时，使用本页命令。以下 Nushell 命令均在项目根目录运行。

## 接入当前 Nu 会话

先从托盘退出桌面版，再构建后台程序并加载接入脚本。

```nu
cargo build --release --locked
source ./integrations/nushell.nu
```

在 Windows Terminal 的交互式 Nu 中执行这些命令。需要随 Nu 启动时，运行 `'./integrations/nushell.nu' | path expand` 获取脚本绝对路径，再在 `config.nu` 顶层添加使用该路径的 `source` 命令。用 `$nu.config-path` 查看配置文件路径。移动项目后，更新脚本路径。

## 查看、停止或恢复颜色

在需要管理的 Nu 会话中运行对应命令。

```nu
./target/release/cuesor-engine.exe status
./target/release/cuesor-engine.exe stop
./target/release/cuesor-engine.exe reset
```

`status` 显示 worker 是否已初始化。`stop` 等待 worker 退出并尝试恢复主题色。`reset` 只恢复颜色，运行中的 worker 可能再次写入指示色。

停止后重新加载接入脚本即可启动。取消随 Nu 启动时，删除 `config.nu` 中的 `source` 行，再对已运行的会话执行 `stop`。Nu 退出时，其 worker 也会退出。

## 指定会话颜色或日志

先停止原 worker，再指定六位十六进制颜色。

```nu
./target/release/cuesor-engine.exe stop
./target/release/cuesor-engine.exe start --shell-pid $nu.pid --cn-color '#FF7800' --en-color '#0077FF'
```

重复 `start` 不会覆盖运行中的选项。如需日志，先停止 worker，再执行以下命令。

```nu
mkdir artifacts
./target/release/cuesor-engine.exe start --shell-pid $nu.pid --log artifacts/worker.jsonl
```

日志记录进程、窗口、输入法状态和颜色写入，不记录输入正文。默认不写日志。

## 排查桌面版等待会话

若输出目录尚不存在，先执行 `mkdir artifacts`。保持桌面版运行，导出连接诊断。

```nu
./target/release/cuesor-engine.exe diagnose artifacts/diagnose.json
```

报告中的 `elevated_terminal_pids` 列出当前权限无法接入的管理员终端。若设置窗口显示权限提示，通过窗口中的管理员重启按钮继续。

## 观察输入状态和颜色

先退出桌面版并停止 CLI worker，避免多个进程控制颜色。创建输出目录后，执行跟随探针并在 20 秒内切换输入模式。

```nu
./target/release/cuesor-engine.exe stop
mkdir artifacts
./target/release/cuesor-engine.exe probe --follow --watch 20 --report artifacts/manual.jsonl
```

`--follow` 持续应用采样颜色，正常结束时恢复主题色。仅查看当前状态时运行 `probe`。仅写入一次颜色时运行 `probe --color cn` 或 `probe --color en`。中断后若颜色未恢复，执行 `reset`。

历史实测范围见 [CLI 验证记录](VALIDATION.md)。桌面版的构建、升级和配置位置见 [README](../README.md)。
