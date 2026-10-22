param([string]$Executable = (Join-Path $PSScriptRoot '..\target\release\TerminalIME.exe'), [switch]$KeepArtifacts)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
$program = (Resolve-Path -LiteralPath $Executable).Path
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$dataDir = Join-Path $projectRoot ('artifacts\desktop-check-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
$bytes = [IO.File]::ReadAllBytes($program)
$pe = [BitConverter]::ToInt32($bytes,0x3c)
if ([BitConverter]::ToUInt16($bytes,$pe+24+68) -ne 2) { throw 'Desktop must use the Windows GUI subsystem.' }
if (![Text.Encoding]::UTF8.GetString($bytes).Contains('Microsoft.Windows.Common-Controls')) { throw 'Missing Common Controls v6 manifest; TaskDialogIndirect may fail at startup.' }
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class TerminalImeDesktopCheck {
 [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left,Top,Right,Bottom; public int Width { get { return Right-Left; } } public int Height { get { return Bottom-Top; } } }
 public delegate bool Callback(IntPtr window,IntPtr data);
 [DllImport("user32.dll")] public static extern bool EnumWindows(Callback callback,IntPtr data);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr window,StringBuilder name,int count);
 [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr window,uint command);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern uint RegisterWindowMessageW(string name);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowW(string name, string title);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint message, UIntPtr wparam, IntPtr lparam);
 [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd,IntPtr after,int x,int y,int width,int height,uint flags);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd,out Rect rect);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd,int command);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 public static IntPtr FindDialog(uint processId,IntPtr owner) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((window,data) => {
        uint id; GetWindowThreadProcessId(window,out id);
        if (id == processId && GetWindow(window,4) == owner) {
            var name = new StringBuilder(256); GetClassNameW(window,name,256);
            if (name.ToString() == "#32770") found = window;
        }
        return found == IntPtr.Zero;
    },IntPtr.Zero);
    return found;
 }
}
'@
$oldDpiContext = [TerminalImeDesktopCheck]::SetThreadDpiAwarenessContext([IntPtr](-4))
function Wait-Condition([scriptblock]$Condition, [string]$Message) {
    $deadline = [DateTime]::UtcNow.AddSeconds(10)
    while (!(& $Condition)) {
        if ([DateTime]::UtcNow -gt $deadline) { throw $Message }
        Start-Sleep -Milliseconds 100
    }
}
# Poll without blocking the application's atomic rename on Windows.
function Read-SharedText([string]$Path) {
    $sharing = [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete
    $stream = [IO.FileStream]::new($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,$sharing)
    $reader = [IO.StreamReader]::new($stream)
    try { $reader.ReadToEnd() } finally { $reader.Dispose() }
}
function Get-SettingControl([string]$Name) {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($window)
    $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,$Name)
    $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
}
function Invoke-SettingControl([string]$Name) {
    Wait-Condition { $candidate = Get-SettingControl $Name; $candidate -and $candidate.Current.IsEnabled } "Accessible control did not become enabled: $Name"
    $control = Get-SettingControl $Name
    $pattern = $null
    if ($control.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern,[ref]$pattern)) {
        $pattern.Invoke()
    } else {
        if (!$control.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern,[ref]$pattern)) { throw "No activation pattern for: $Name" }
        $pattern.Toggle()
    }
}
function Set-CursorColor([bool]$Chinese,[int]$Rgb) {
    $label = if ($Chinese) { '自定义中文输入光标颜色' } else { '自定义英文输入光标颜色' }
    $property = if ($Chinese) { 'chinese_color' } else { 'english_color' }
    Invoke-SettingControl $label
    Wait-Condition { [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window) -ne [IntPtr]::Zero } 'Color dialog did not open.'
    $dialog = [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window)
    $message = [TerminalImeDesktopCheck]::RegisterWindowMessageW('commdlg_SetRGBColor')
    $bgr = (($Rgb -band 255) -shl 16) -bor ($Rgb -band 65280) -bor ($Rgb -shr 16)
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,$message,[UIntPtr]::Zero,[IntPtr]$bgr)
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,0x111,[UIntPtr]1,[IntPtr]::Zero)
    $expected = '#{0:X6}' -f $Rgb
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).$property -eq $expected } "Color did not save: $expected"
}
$oldDataDirectory = $env:TERMINAL_IME_DATA_DIR
$oldSession = $env:WT_SESSION
$env:TERMINAL_IME_DATA_DIR = $dataDir
$backend = Join-Path ([IO.Path]::GetDirectoryName($program)) 'terminal-ime.exe'
$legacyTested = $false
$existingSession = $null
$gui = $null
$tray = [IntPtr]::Zero
$children = @()
try {
    & $backend discover (Join-Path $dataDir 'discovered-before-gui.json')
    if ($LASTEXITCODE -ne 0) { throw 'Discovery probe failed.' }
    $existingSession = Get-Content -LiteralPath (Join-Path $dataDir 'discovered-before-gui.json') -Raw | ConvertFrom-Json | Select-Object -First 1
    if ($existingSession) {
        $env:WT_SESSION = [guid]::NewGuid().ToString()
        & $backend start --shell-pid $existingSession.shell_pid --log (Join-Path $dataDir 'legacy-worker.jsonl')
        if ($LASTEXITCODE -ne 0) { throw 'Legacy worker did not start.' }
        Wait-Condition { (& $backend status | ConvertFrom-Json).running } 'Legacy worker did not initialize.'
        $legacyTested = $true
    }
    $gui = Start-Process -FilePath $program -PassThru -RedirectStandardError (Join-Path $dataDir 'stderr.log')
    Wait-Condition { $gui.Refresh(); $gui.HasExited -or $gui.MainWindowHandle -ne 0 } 'Settings did not appear.'
    if ($gui.HasExited) { throw "Settings process exited: $($gui.ExitCode)." }
    if ($legacyTested) { Wait-Condition { !(& $backend status | ConvertFrom-Json).running } 'Legacy worker did not yield to the desktop app.' }
    $window = $gui.MainWindowHandle
    $defaultBounds = [TerminalImeDesktopCheck+Rect]::new()
    $null = [TerminalImeDesktopCheck]::GetWindowRect($window,[ref]$defaultBounds)
    $scale = [TerminalImeDesktopCheck]::GetDpiForWindow($window)/96.0
    if ($defaultBounds.Height/$scale -gt 640) { throw 'Default window is still too tall.' }
    $tray = [TerminalImeDesktopCheck]::FindWindowW('TerminalIME.Tray','Terminal IME tray')
    $trayPid = [uint32]0
    $null = [TerminalImeDesktopCheck]::GetWindowThreadProcessId($tray,[ref]$trayPid)
    if ($tray -eq [IntPtr]::Zero -or $trayPid -ne $gui.Id) { throw 'No owned tray window.' }
    $null = [TerminalImeDesktopCheck]::PostMessageW($window,0x10,[UIntPtr]::Zero,[IntPtr]::Zero)
    Wait-Condition { ![TerminalImeDesktopCheck]::IsWindowVisible($window) } 'Close did not hide settings.'
    if ($gui.HasExited) { throw 'Closing settings killed the tray app.' }
    $second = Start-Process -FilePath $program -PassThru -WindowStyle Hidden
    if (!$second.WaitForExit(5000)) { throw 'Second launch did not reuse the existing instance.' }
    Wait-Condition { [TerminalImeDesktopCheck]::IsWindowVisible($window) } 'Second launch did not restore settings.'
    $settingsPath = Join-Path $dataDir 'settings.json'
    Invoke-SettingControl '自定义中文输入光标颜色'
    Wait-Condition { [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window) -ne [IntPtr]::Zero } 'Custom color dialog did not open (or GUI crashed).'
    $dialog = [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window)
    # Use the documented common-dialog selection message, not private control IDs.
    $setRgbMessage = [TerminalImeDesktopCheck]::RegisterWindowMessageW('commdlg_SetRGBColor')
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,$setRgbMessage,[UIntPtr]::Zero,[IntPtr]0xff0000)
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,0x111,[UIntPtr]1,[IntPtr]::Zero)
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).chinese_color -eq '#0000FF' } 'Custom color was not saved.'
    Invoke-SettingControl '自定义中文输入光标颜色'
    Wait-Condition { [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window) -ne [IntPtr]::Zero } 'Second color dialog did not open.'
    $dialog = [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window)
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,0x111,[UIntPtr]2,[IntPtr]::Zero)
    Wait-Condition { [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window) -eq [IntPtr]::Zero } 'Color cancel did not close the dialog.'
    if ((Read-SharedText $settingsPath | ConvertFrom-Json).chinese_color -ne '#0000FF') { throw 'Cancel changed the saved color.' }
    $schemeNames = @('经典','珊瑚','暮紫','极光','柔粉','暖砂')
    $customColors = @()
    for ($index=0; $index -lt 6; $index++) {
        Invoke-SettingControl ('配色{0}：{1}' -f ($index+1),$schemeNames[$index])
        Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).active_scheme -eq $index } 'Scheme selection did not save.'
        $saved = Read-SharedText $settingsPath | ConvertFrom-Json
        if ($saved.chinese_color -ne $saved.color_schemes[$index].chinese_color -or $saved.english_color -ne $saved.color_schemes[$index].english_color) { throw 'Scheme did not apply both colors.' }
        $cn = 0xCC3300 + $index*0x111
        $en = 0x0077AA + $index*0x1010
        Set-CursorColor $true $cn
        Set-CursorColor $false $en
        $customColors += [PSCustomObject]@{Chinese=('#{0:X6}' -f $cn);English=('#{0:X6}' -f $en)}
    }
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]3,[IntPtr]::Zero)
    if (!$gui.WaitForExit(5000)) { throw 'GUI did not exit after editing the schemes.' }
    $gui = Start-Process -FilePath $program -PassThru
    Wait-Condition { $gui.Refresh(); $gui.HasExited -or $gui.MainWindowHandle -ne 0 } 'Color scheme restart did not create settings.'
    if ($gui.HasExited) { throw 'Color scheme restart exited unexpectedly.' }
    $window = $gui.MainWindowHandle
    $tray = [TerminalImeDesktopCheck]::FindWindowW('TerminalIME.Tray','Terminal IME tray')
    for ($index=0; $index -lt 6; $index++) {
        Invoke-SettingControl ('配色{0}：{1}' -f ($index+1),$schemeNames[$index])
        Wait-Condition {
            $saved = Read-SharedText $settingsPath | ConvertFrom-Json
            $saved.active_scheme -eq $index -and $saved.chinese_color -eq $customColors[$index].Chinese -and $saved.english_color -eq $customColors[$index].English
        } 'An edited color scheme lost its independent colors.'
    }
    Invoke-SettingControl '重置当前配色'
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).chinese_color -eq '#A18072' } 'Reset did not restore the selected recommendation.'
    $saved = Read-SharedText $settingsPath | ConvertFrom-Json
    if ($saved.color_schemes[0].chinese_color -ne $customColors[0].Chinese) { throw 'Reset changed a different scheme.' }
    # Restore the remaining recommendations for the visual inspection.
    for ($index=0; $index -lt 5; $index++) {
        Invoke-SettingControl ('配色{0}：{1}' -f ($index+1),$schemeNames[$index])
        Invoke-SettingControl '重置当前配色'
    }
    Invoke-SettingControl '配色2：珊瑚'
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).active_scheme -eq 1 } 'Final scheme did not select.'
    Wait-Condition { !(Get-SettingControl '重置当前配色').Current.IsEnabled } 'Reset must be disabled for unchanged recommendations.'
    if ((Get-SettingControl '登录 Windows 时启动').Current.IsOffscreen) { throw 'Default layout hides the startup setting.' }
    Invoke-SettingControl '外观：浅色'
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).appearance -eq 'light' } 'Light appearance did not save.'
    Start-Sleep -Milliseconds 400
    & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $gui.Id -OutputPath (Join-Path $dataDir 'light.png')
    $cnBounds = (Get-SettingControl '自定义中文输入光标颜色').Current.BoundingRectangle
    $enBounds = (Get-SettingControl '自定义英文输入光标颜色').Current.BoundingRectangle
    if ($cnBounds.Top -ne $enBounds.Top -or $cnBounds.Height -ne $enBounds.Height -or $cnBounds.Width -ne $enBounds.Width) { throw 'Color editor controls do not share the same row geometry.' }
    Invoke-SettingControl '外观：深色'
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).appearance -eq 'dark' } 'Dark appearance did not save.'
    Start-Sleep -Milliseconds 400
    & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $gui.Id -OutputPath (Join-Path $dataDir 'dark.png')
    $control = Get-SettingControl '启用光标颜色指示'
    $control.SetFocus()
    Start-Sleep -Milliseconds 150
    $null = [TerminalImeDesktopCheck]::PostMessageW($window,0x100,[UIntPtr]0x20,[IntPtr]::Zero)
    $null = [TerminalImeDesktopCheck]::PostMessageW($window,0x101,[UIntPtr]0x20,[IntPtr]::Zero)
    Wait-Condition { !(Read-SharedText $settingsPath | ConvertFrom-Json).enabled } 'Keyboard Space did not toggle the switch.'
    Start-Sleep -Milliseconds 300
    & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $gui.Id -OutputPath (Join-Path $dataDir 'paused-focus.png')
    $null = [TerminalImeDesktopCheck]::PostMessageW($window,0x100,[UIntPtr]0x09,[IntPtr]::Zero)
    $null = [TerminalImeDesktopCheck]::PostMessageW($window,0x101,[UIntPtr]0x09,[IntPtr]::Zero)
    Wait-Condition { [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name -eq '自定义中文输入光标颜色' } 'Tab did not advance to the color editor.'
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]2,[IntPtr]::Zero)
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).enabled } 'Tray resume after keyboard toggle failed.'
    $scale = [TerminalImeDesktopCheck]::GetDpiForWindow($window)/96.0
    $null = [TerminalImeDesktopCheck]::SetWindowPos($window,[IntPtr]::Zero,0,0,[int](760*$scale),[int](520*$scale),6)
    Start-Sleep -Milliseconds 400
    foreach ($name in @('自定义中文输入光标颜色','自定义英文输入光标颜色','外观：深色','重置当前配色','配色6：暖砂')) {
        if ((Get-SettingControl $name).Current.IsOffscreen) { throw "Control is clipped at minimum size: $name" }
    }
    & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $gui.Id -OutputPath (Join-Path $dataDir 'minimum.png')
    $null = [TerminalImeDesktopCheck]::SetWindowPos($window,[IntPtr]::Zero,0,0,[int](940*$scale),[int](620*$scale),6)
    $resizedBounds = [TerminalImeDesktopCheck+Rect]::new()
    $null = [TerminalImeDesktopCheck]::GetWindowRect($window,[ref]$resizedBounds)
    $windowStatePath = Join-Path $dataDir 'window.json'
    Wait-Condition { (Test-Path -LiteralPath $windowStatePath) -and (Read-SharedText $windowStatePath | ConvertFrom-Json).width -ge 900 } 'Window resize did not persist.'
    $normalSize = Read-SharedText $windowStatePath
    $null = [TerminalImeDesktopCheck]::ShowWindow($window,3)
    Start-Sleep -Milliseconds 600
    $null = [TerminalImeDesktopCheck]::ShowWindow($window,6)
    Start-Sleep -Milliseconds 600
    if ((Read-SharedText $windowStatePath) -ne $normalSize) { throw 'Maximize or minimize overwrote the normal size.' }
    $null = [TerminalImeDesktopCheck]::ShowWindow($window,9)
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]2,[IntPtr]::Zero)
    Wait-Condition { !(Read-SharedText $settingsPath | ConvertFrom-Json).enabled } 'Tray pause did not persist.'
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]2,[IntPtr]::Zero)
    Wait-Condition { (Read-SharedText $settingsPath | ConvertFrom-Json).enabled } 'Tray resume did not persist.'
    Start-Sleep -Seconds 3
    $runtime = Read-SharedText (Join-Path $dataDir 'runtime.json') | ConvertFrom-Json
    if ($legacyTested -and $runtime.sessions.shell_pid -notcontains $existingSession.shell_pid) { throw 'Desktop did not take over the legacy console.' }
    $ownedProcesses = @($runtime.worker_pids) + @(Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -eq $gui.Id -and $_.Name -eq 'terminal-ime.exe' } | ForEach-Object ProcessId)
    $children = @($ownedProcesses | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]3,[IntPtr]::Zero)
    if (!$gui.WaitForExit(5000)) { throw 'Tray exit did not stop the application.' }
    foreach ($child in $children) { if (!$child.WaitForExit(5000)) { throw "Worker/agent $($child.Id) survived the application." } }
    $gui = Start-Process -FilePath $program -PassThru
    Wait-Condition { $gui.Refresh(); $gui.HasExited -or $gui.MainWindowHandle -ne 0 } 'Restart did not create settings.'
    if ($gui.HasExited) { throw 'Restart exited unexpectedly.' }
    $window = $gui.MainWindowHandle
    $reopenedBounds = [TerminalImeDesktopCheck+Rect]::new()
    $null = [TerminalImeDesktopCheck]::GetWindowRect($window,[ref]$reopenedBounds)
    if ([math]::Abs($reopenedBounds.Width-$resizedBounds.Width) -gt 2 -or [math]::Abs($reopenedBounds.Height-$resizedBounds.Height) -gt 2) { throw 'Saved window dimensions were not restored after restart.' }
    Wait-Condition {
        $control = Get-SettingControl '配色2：珊瑚'
        $control -and $control.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState -eq [System.Windows.Automation.ToggleState]::On
    } 'Color scheme selection was not restored after restart.'
    $tray = [TerminalImeDesktopCheck]::FindWindowW('TerminalIME.Tray','Terminal IME tray')
    Wait-Condition {
        $control = Get-SettingControl '外观：深色'
        $control -and $control.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState -eq [System.Windows.Automation.ToggleState]::On
    } 'Saved appearance was not restored after restart.'
    $children = @(Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -eq $gui.Id -and $_.Name -eq 'terminal-ime.exe' } | ForEach-Object { Get-Process -Id $_.ProcessId })
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]3,[IntPtr]::Zero)
    if (!$gui.WaitForExit(5000)) { throw 'Restarted GUI did not exit.' }
    foreach ($child in $children) { if (!$child.WaitForExit(5000)) { throw 'Restarted controller did not exit.' } }
    $verdict = [PSCustomObject]@{
        GuiSubsystem=$true; CommonControlsManifest=$true; SettingsVisible=$true
        TrayOwned=$true; CloseHides=$true; SingletonRestores=$true
        CustomColorSaved=$true; ColorCancelPreservesValue=$true; SchemePairsApplied=$true; AllSixSchemesEditable=$true; CustomSchemesRestoredAfterRestart=$true; SchemeSelectionRestored=$true; ResetDefaults=$true
        AppearanceSaved=$true; AppearanceRestoredAfterRestart=$true
        CompactDefaultWindow=$true; WindowResizeRestored=$true; MaximizeMinimizePreservesSize=$true
        DefaultOuterWindowLogicalHeight=($defaultBounds.Height/$scale); RestoredWindowBounds=@{Width=$reopenedBounds.Width;Height=$reopenedBounds.Height}
        ColorEditorRowsAligned=$true
        ColorEditorControlBounds=@(@($cnBounds,$enBounds) | Select-Object X,Y,Width,Height)
        KeyboardSwitch=$true; KeyboardTabOrder=$true; MinimumWindowControlsVisible=$true
        PauseResumePersisted=$true; LegacyHandoffTested=$legacyTested
        InitializedSessions=$runtime.sessions.Count; ExitCleansWorkers=$true; DataDirectory=$(if($KeepArtifacts){$dataDir}else{$null}); TemporaryArtifactsRemoved=(!$KeepArtifacts)
    }
    $verdict | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $projectRoot 'artifacts\desktop-check-verdict.json') -Encoding utf8NoBOM
    $verdict | ConvertTo-Json -Depth 5
} finally {
    if ($KeepArtifacts -and $gui -and !$gui.HasExited -and $gui.MainWindowHandle -ne 0) {
        & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $gui.Id -OutputPath (Join-Path $dataDir 'last-visible.png')
    }
    if ($legacyTested) { & $backend stop }
    $env:WT_SESSION = $oldSession
    $env:TERMINAL_IME_DATA_DIR = $oldDataDirectory
    if ($gui -and !$gui.HasExited) {
        $ownedTray = [TerminalImeDesktopCheck]::FindWindowW('TerminalIME.Tray','Terminal IME tray')
        $ownerPid = [uint32]0
        $null = [TerminalImeDesktopCheck]::GetWindowThreadProcessId($ownedTray,[ref]$ownerPid)
        if ($ownerPid -eq $gui.Id) {
            $null = [TerminalImeDesktopCheck]::PostMessageW($ownedTray,0x111,[UIntPtr]3,[IntPtr]::Zero)
            $null = $gui.WaitForExit(5000)
        }
    }
    if (!$KeepArtifacts -and (Test-Path -LiteralPath $dataDir)) {
        $resolved = (Resolve-Path -LiteralPath $dataDir).Path
        $expectedPrefix = (Join-Path $projectRoot 'artifacts') + '\desktop-check-'
        if (!$resolved.StartsWith($expectedPrefix,[StringComparison]::OrdinalIgnoreCase) -or ((Get-Item -LiteralPath $resolved).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe temporary cleanup path.' }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
    $null = [TerminalImeDesktopCheck]::SetThreadDpiAwarenessContext($oldDpiContext)

}
