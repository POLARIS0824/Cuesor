# Cuesor 诊断与配置

日常操作使用 `Cuesor.exe` 的设置窗口和托盘。`cuesor-engine.exe` 是同目录的后台组件，由 GUI 管理终端接入和退出。

## 导出终端报告

在项目根目录使用 Nushell 运行以下命令。

```nu
mkdir artifacts
./target/release/cuesor-engine.exe discover artifacts/discover.json
./target/release/cuesor-engine.exe diagnose artifacts/diagnose.json
```

`discover` 输出终端会话数组。`diagnose` 同时输出会话和需要管理员权限的终端 PID。报告中的 `shell_pid` 是控制台最早的客户端 PID，可能属于 shell 启动器。与实际进程身份核对后再判断接入情况。

这两个命令只检查终端，不启动光标指示。手动 `start`、`stop`、`status`、`reset` 和 `probe` 已移除。暂停、恢复和退出通过 GUI 或托盘完成。

## 配置和运行状态

默认目录为 `%LOCALAPPDATA%\Cuesor\`。验证时可用 `CUESOR_DATA_DIR` 指定独立目录。

| 文件 | 内容 |
| --- | --- |
| `settings.json` | 启用状态、登录启动偏好、外观、六组配色和选中组。 |
| `window.json` | 普通窗口的逻辑尺寸。 |
| `runtime.json` | 当前 GUI PID、已初始化会话、worker PID 和接入错误。 |
| `last-worker-error.json` | 最近一次 worker 启动失败的记录。 |

当前颜色从 `color_schemes[active_scheme]` 读取。新配置不再保存顶层 `chinese_color` 和 `english_color`。读取旧配置时，原先生效的顶层颜色导入选中组，其他组保留。旧配置没有配色数组时导入第一组。启动保存后完成格式转换，旧版程序不支持新格式的完整读取。

JSON 格式错误、无效颜色或越界的选中下标会阻止加载。程序不会以默认设置覆盖这些文件。

## 解释验证结果

`runtime.json` 中的会话和存活 worker 能证明接入状态。证明实际光标变色还需要观察测试终端。GUI 配色预览、构建成功和报告成功均不能替代实际终端观察。
