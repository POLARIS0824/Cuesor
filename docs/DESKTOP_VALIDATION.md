# Cuesor 桌面版验证

本文区分 Cuesor 更名后的工作区检查与早期桌面版的历史记录。日期使用 Asia/Hong_Kong 时区。

## 更名后检查

2026-10-24，更名后的工作区通过 release 构建、格式检查、13 项后端测试，以及 PowerShell 和 Nushell 脚本语法检查。CLI 命令选项保持一致，后台版本入口输出 `cuesor-engine 0.2.0`。

实际窗口标题为 "Cuesor 设置"。Appearance、Colors、Tray 和 Pause 四条自动流程通过，覆盖外观恢复、十二个颜色的独立保存、当前组重置、暂停、关闭隐藏、再次启动恢复和托盘消息处理。各次运行均正常退出，没有强制结束进程。

证据位于 `artifacts/verify/cuesor-rename-appearance/`、`artifacts/verify/cuesor-rename-colors-02/`、`artifacts/verify/cuesor-rename-tray/` 和 `artifacts/verify/cuesor-rename-pause/`。这些运行使用隔离的 `CUESOR_DATA_DIR`，不改动日常配置或登录启动项。Git 追踪的截图保持原样。

本轮没有 Windows Terminal 会话，未复测实际终端颜色、UAC、实际登录启动或 Explorer 托盘手势。以下早期记录不能替代这些更名后未验证的路径。

## 早期记录

下文记录 2026-10-21 的 0.2.0 桌面版验证，早于 Cuesor 更名。该轮临时截图、测试目录和详细日志已于 2026-10-22 清理，本文保留结论与边界。

## 验证环境

环境为 Windows 10.0.26300、Windows Terminal 1.24.12741.0、Nu 0.116.0、Atuin 18.24.1 和 Rust 1.98.1 Windows GNU。GUI 使用 GPUI 0.2.2，托盘、颜色选择器和登录启动使用 Win32 API。后续权限和保存修复使用 Rust 1.99.1 GNU，未重新测量首轮资源数据。

## 已观察到的行为

| 项目 | 历史结果与证据范围 |
| --- | --- |
| 构建与显示 | release 构建和实际窗口启动成功，检查了 150% DPI 布局。补齐 Common Controls 6 清单后，原先的 `TaskDialogIndirect` 入口点错误不再复现。 |
| 无控制台入口 | GUI 的 PE subsystem 为 Windows GUI（2）。后台使用 `CREATE_NO_WINDOW`，不继承句柄。静态检查不能证明所有启动条件下都没有闪窗。 |
| 自动接入 | 发现已有及新开的 Nu、PowerShell 会话，最终 3 个会话初始化且无错误。JetBrains 内嵌 Nu 被排除。 |
| 托盘与单实例 | 桌面检查覆盖关闭隐藏、再次启动恢复原窗口、托盘消息处理及暂停恢复保存。未以此推定托盘溢出区域的实际位置。 |
| 颜色选择器 | 自动交互确认保存蓝色，取消第二次选择后颜色保持不变。 |
| 管理员接入 | 用户完成 UAC 重启并确认光标变化。令牌检查与运行记录显示提升后的 GUI、1 个 ready worker，且 `errors` 为空。 |
| 登录启动 | 设置开关实际写入包含 GUI 路径和 `--background` 的 HKCU Run 值，关闭后移除。测试未保留启动项，未注销或重新登录。 |
| 正常退出 | GUI 退出后，自动发现进程及全部 worker 退出。 |

自动发现曾将扫描进程自己选为 console 的最早客户端。回归测试先失败，修复后通过，并用真实新建 Nu 会话验证接入。测试没有自动修改 Nu、Atuin 或 Windows Terminal 配置。

## 权限与保存故障

等待会话故障发生时，GUI 为普通权限，Windows Terminal 与 Nu 为管理员权限。`AttachConsole` 返回错误码 5，发现结果却为空且没有提示。修复后，诊断和后台控制器均报告管理员终端，设置窗口提供管理员重启按钮。

最终人工验证中，GUI PID 41364 已提升权限，Nu PID 43304 的会话由 worker PID 16236 初始化，运行记录没有错误。该结果覆盖一次实际 UAC 确认及颜色跟随，未覆盖取消授权、其他 Windows 用户或所有权限组合。

保存故障在真实设置目录中复现为 `0x80070011`。独立 Win32 探针同样返回错误码 17。对 `ERROR_NOT_SAME_DEVICE` 启用 `MOVEFILE_COPY_ALLOWED` 后，新建与覆盖检查通过。此回退通过复制后删除完成，不具有同卷重命名的原子性。文件具有 Encrypted 属性，但这不足以确认错误的底层原因。

修复构建通过 9 项 Rust 测试、格式检查和 workspace release Clippy。原生包另通过 `--all-targets` Clippy。设置读写和后台创建移出 GPUI 回调，避免原生模态循环重入已经借用的 GPUI 上下文。

## 资源采样

release 版关闭设置窗口，保留托盘和 3 个会话，连续采样约 10 秒。

| 指标 | 历史结果 |
| --- | ---: |
| GUI 私有内存 | 96.27 MiB |
| 自动发现进程私有内存 | 3.39 MiB |
| GUI、发现进程和 3 个 worker 私有内存合计 | 109.40 MiB |
| CPU 占单核容量 | 1.718% |
| CPU 占 16 逻辑处理器整机容量 | 0.107% |

这是未固定前台的低活动短采样，未统计 GPU 显存。结果不代表持续输入峰值或其他机器。单个 native worker 约 3.4 MiB 的数据不能代表整个桌面应用。

桌面版没有 Nu 冷启动开关对照，也没有端到端颜色响应 P95。[CLI 历史数据](VALIDATION.md)中的启动增量只适用于当时的 Nu 接入脚本。

## 构建与验收边界

该机器没有 Windows SDK。GNU 构建通过系统 `d3dcompiler_47` 编译着色器，并从完整 GCC 安装补充导入库。Rust 1.99.1 与 Binutils 2.45.1 曾出现导入库 stub 文件丢失，构建工具通过为每次 `dlltool` 调用分配独立临时前缀解决本次失败。这些适配器只参与构建。

当时首次 GPUI release 构建约十分钟，缓存齐全后应用自身重建约一分钟。这是本机历史耗时，没有控制冷启动条件或重复采样。

取消 UAC、实际登录启动、Explorer 重启后的托盘恢复、多显示器 DPI、拖出标签页及其他输入法均未完整验收。强制结束 worker 仍可能留下颜色，需要手动 `reset`。2026-10-22 和 2026-10-24 的界面证据见 [界面参考与历史检查](GUI_REDESIGN.md)。
