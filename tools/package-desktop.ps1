$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$destination = Join-Path $projectRoot 'artifacts\TerminalIME-portable'
New-Item -ItemType Directory -Path $destination -Force | Out-Null
foreach ($name in @('TerminalIME.exe','terminal-ime.exe')) {
    $source = Join-Path $projectRoot "target\release\$name"
    if (!(Test-Path -LiteralPath $source)) { throw "Missing $source; run tools/build-desktop.ps1 first." }
    Copy-Item -LiteralPath $source -Destination (Join-Path $destination $name)
}
@'
Terminal IME

双击 TerminalIME.exe 打开设置。terminal-ime.exe 是后台组件，请保留在同一目录。
托盘菜单可打开设置、暂停/恢复、退出。关闭设置窗口会继续驻留托盘。
托盘图标可能在任务栏的“显示隐藏图标”区域。
打开 Windows Terminal 后会自动接入，无需执行 Nushell 命令或修改 shell 配置。
管理员终端需要在设置中切换管理员模式并确认 Windows 授权提示。
颜色与启用状态自动保存到 %LOCALAPPDATA%\TerminalIME\settings.json。
登录启动默认关闭，可在设置中开启。
'@ | Set-Content -LiteralPath (Join-Path $destination '使用说明.txt') -Encoding utf8
Get-ChildItem -LiteralPath $destination | Select-Object Name,Length
