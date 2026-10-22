# 0.2.0 桌面版验证

清理说明：本文记录的临时截图、测试目录和详细日志已于 2026-10-22 按要求清理；下文保留历史验证过程，文件路径不代表产物仍然存在。当前保留的便携包与本轮验证摘要见 [界面重设计记录](GUI_REDESIGN.md)。

日期：2026-10-21，Asia/Hong_Kong。环境沿用原版：Windows 10.0.26300、Windows Terminal 1.24.12741.0、Nu 0.116.0、Atuin 18.24.1、Rust 1.98.1 Windows GNU。GUI 使用发布的 GPUI 0.2.2；托盘、颜色选择器和登录启动使用原生 Win32 API。

下述权限与保存故障的后续修复构建使用本机当前 Rust 1.99.1 GNU；首轮资源采样不随构建版本更新而自动重测。

## 已获得的证据

| 验证 | 结果与证据 |
| --- | --- |
| GPUI 构建和实际显示 | GNU 工具链完成 release 构建，实际窗口标题为“Terminal IME · 设置”；窗口截图见 `artifacts/release-settings.png`。已检查 150% DPI 下的布局，并修复滚动区域顶部内容被挤压的问题。 |
| 无控制台桌面入口 | GUI 的 PE subsystem 为 Windows GUI（2）；后台由 `CreateProcessW` 的 `CREATE_NO_WINDOW` 且不继承句柄创建。不能把 PE 检查当作所有启动条件下都绝不闪窗的人工证据。 |
| Common Controls 启动回归 | 初次缺少 Common Controls 6 清单声明，用户截图报告 `TaskDialogIndirect` 入口点错误。补齐清单后实际窗口成功打开；桌面检查同时验证清单和真实启动。 |
| 已有终端自动接入 | 独立后台发现不加载 Nu 配置的 Nu 和 PowerShell 分屏；JetBrains 内嵌 Nu 被排除，范围限定 Windows Terminal。 |
| 后开终端自动接入 | 控制器启动后新开 Nu，最终 3 个会话均初始化，报告无错误。见 `artifacts/agent-new-session-check/runtime.json` 和 `artifacts/agent-new-session-verdict.json`。 |
| 托盘、关闭隐藏、单实例恢复 | `tools/desktop-check.ps1` 验证本应用的托盘消息窗口、关闭设置后进程存活、第二次启动退出并恢复原窗口。该项不代替 Windows 托盘溢出区域位置的人工观察。 |
| 暂停／恢复保存 | 桌面检查通过托盘命令实际切换设置，确认 `settings.json` 的 enabled 值变化。 |
| 颜色色板 | 向本应用窗口发送真实命中的鼠标消息，中文颜色从 `#FF7800` 保存为 `#F05C72`。 |
| 自定义颜色选择器 | `tools/desktop-check.ps1` 实际打开原生选择器，通过官方 `commdlg_SetRGBColor` 消息选择蓝色并确认保存，再取消第二次选择，确认颜色不变。原生模态循环改到独立线程，避免回调期间重入 GPUI 造成 `RefCell already borrowed` 崩溃。 |
| 管理员终端接入与实际颜色 | 本次用户在新 GUI 中完成管理员模式切换，确认“已经成功接入，光标也变化了”。随后实际令牌检查显示 GUI 已提升权限，运行记录有 1 个 Nushell 会话、1 个 ready worker、无错误。见 `artifacts/terminal-access-final-verdict.json`。 |
| 登录启动开关 | 实际点击设置开关，确认本应用的 HKCU Run 值包含 GUI 路径及 `--background`；再关闭开关，确认该值移除。测试前后均未保留启动项。见 `artifacts/desktop-settings-verdict.json`。未实际注销／登录验证。 |
| 退出清理 | 桌面检查正常退出 GUI 后，自动发现进程和全部 worker 均退出；初始化会话数为 3。见 `artifacts/desktop-check-verdict.json`。 |

自动发现最初选取 console 中创建时间最早的客户端，却未排除附着进去的扫描进程自己。当托盘先运行、shell 后打开时，会错误绑定到扫描进程。`newly_opened_shell_does_not_bind_to_the_older_discovery_process` 已先运行失败，再修复并通过；之后还用真实新建 Nu 验证，避免只用单元测试推断会话接入成功。

当前测试中没有自动写入 Nu／Atuin／Windows Terminal 配置，没有采集历史命令、终端正文或其他应用画面作为交付证据。调试、截图及输入辅助工具仅用于测试，产品运行无需 PowerShell／Python 辅助脚本。

## 管理员终端等待故障

用户报告打开软件和 Windows Terminal 后一直等待会话。实际检查发现 GUI 与发现进程均为普通权限，Windows Terminal 与 Nu 为管理员权限；只读查看终端设置也确认默认 Nushell profile 的 `elevate` 为 true。对该 Nu 的 `AttachConsole` 调用返回错误码 5（Access denied），旧版发现结果为 `[]`，但运行记录没有错误。旧版跳过附加失败，却未向设置页面报告权限边界。

`tools/check-terminal-access.ps1 -ProcessId 1444` 在实际故障进程上先运行失败，明确捕获“管理员终端存在但界面仍显示等待”的情况。修改后的发现流程读取 Windows Terminal 进程的提升令牌，分别报告已接入会话和 `elevated_terminal_pids`；设置页面据此提示权限不一致并提供管理员重启按钮。该提示不自动触发 UAC，也不更改终端的管理员配置。

诊断入口为 `terminal-ime.exe diagnose REPORT.json`，原有 `discover` 的数组输出保持兼容。新构建使用本机当前 Rust 1.99.1 GNU；上文资源采样仍是此前的测量，未重新测量。

修复后已用原生诊断进程及完整后台控制器分别运行真实发现流程，两者都正确报告管理员终端 PID 50276。Rust 的 9 项测试、原生包的 Clippy 与格式检查通过；最终 GUI／UAC 验证单独记录，不以这些检查替代。

最终人工验证后，GUI PID 41364 的提升令牌为 true，已初始化 Nu PID 43304 的会话，worker PID 16236；`errors` 为空。该验证覆盖了此次实际 UAC 重启及颜色跟随路径，没有验证取消授权、其他 Windows 用户或所有权限组合。

最终 `cargo clippy --workspace --release --offline --locked -j 1 -- -D warnings` 通过；原生包也单独通过了 `--all-targets` Clippy 检查。验证汇总及发布文件 SHA-256 见 `artifacts/final-validation.json`。

## 本地设置保存故障

打开修复版时另复现 `The system cannot move the file to a different disk drive. (0x80070011)`。真实设置目录的新建与覆盖检查 `tools/check-settings-storage.ps1` 先运行失败；独立 Win32 探针在同目录的测试文件上也得到错误码 17，增加 `MOVEFILE_COPY_ALLOWED` 后返回成功且目标内容正确。该目录及新建临时文件有 Encrypted 属性，但测试没有修改其加密属性，也不能仅凭此确认底层为何报告跨卷。

`write_json` 保留正常文件系统的原子重命名；仅遇到 `ERROR_NOT_SAME_DEVICE` 时，按 [MoveFileExW 官方说明](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw) 使用允许复制的兼容模式。此回退是复制后删除，不能宣称与同卷重命名具有相同的原子性；运行时读取失败会跳过该次更新，worker 保留上次有效设置。真实设置目录的新建和覆盖检查已在修复后的原生程序上通过，9 项 Rust 测试也重新通过。

设置加载、保存和后台创建移到 GPUI 回调之外；创建窗口失败也在事件循环结束后报告，避免在已借用的 GPUI 上下文中运行原生错误对话框。用户的中文颜色 `#20A78B`、英文颜色 `#0077FF`、启用状态及登录启动偏好保持原值。

## 资源采样

release 版关闭设置窗口、保留托盘和 3 个会话，连续采样约 10 秒：

| 指标 | 本轮结果 |
| --- | ---: |
| GPUI 主进程私有内存 | 96.27 MiB |
| 自动发现进程私有内存 | 3.39 MiB |
| GUI＋发现进程＋3 worker 私有内存合计 | 109.40 MiB |
| CPU 占单核容量 | 1.718% |
| CPU 占 16 逻辑处理器整机容量 | 0.107% |

记录为 `artifacts/desktop-resources.json`。这是低活动短采样，没有固定前台，未统计 GPU 显存，不能当成持续输入峰值或其他机器的保证。GUI 增加了明显常驻内存；原来约 3.4 MiB 的数据只描述单个 native worker，不能用于整个桌面应用。

桌面版不在 Nu 启动配置中执行同步启动命令，但目前没有开启／关闭整个桌面应用的 Nu 冷启动对照数据，也没有端到端颜色响应 P95。原版的中位数 +21.68 ms、P95 +57.03 ms 数据仅适用于旧 Nu 接入脚本。

## 构建注意

本机没有 Windows SDK。项目内着色器编译适配器使用系统 `d3dcompiler_47` 的 `D3DCompileFromFile`，release 构建已通过 GPUI 的全部着色器编译。GNU 构建从完整 GCC 安装中补充 Rust 捆绑 MinGW 缺少的导入库搜索路径，并通过 `windres` 嵌入应用图标、DPI 与 Common Controls 清单。

首次并行构建出现 GNU dlltool 临时导入文件错误，当前构建脚本采用 `-j 1`。缓存齐全后，应用自身的 release 重建本机约一分钟，首次 GPUI release 构建约十分钟。依赖 `proc-macro-error2` 存在 Rust future incompatibility 提示；当前编译通过，尚未替换上游依赖。

当前 Rust 1.99.1／Binutils 2.45.1 的 `windows-sys` 构建在 `-j 1` 下仍重复出现导入库 stub 文件丢失。构建脚本增加 `dlltool-wrapper.rs`，将每次调用的临时前缀设为编译器输出目录中的独立路径；同一构建继续后成功通过原先失败的 `windows-sys`。这只影响构建，不随产品启动运行，也不修改全局工具链。

## 人工或环境验收边界

- 自定义颜色选择器已覆盖确认与取消的自动交互；任意自定义色、GUI 与 Nu 编辑交替、多标签的最终颜色观察，仍需人工覆盖；原版的颜色和 Atuin 人工反馈不可替代全部新交互。
- 本次 UAC 确认与重启已验证；取消授权、实际登录启动、Explorer 重启后的托盘恢复、多显示器 DPI、拖出标签页和其他第三方输入法尚未全部验收。
- 新旧 CLI／桌面控制通过 console 单实例及桌面检测协调，避免同一 console 同时写不同颜色；强制结束进程时仍可能需要手动 reset。
- 发布目录必须同时保留 `TerminalIME.exe` 与 `terminal-ime.exe`，不需要复制系统 DLL。当前仅本机产物，未创建安装器或对外发布。
