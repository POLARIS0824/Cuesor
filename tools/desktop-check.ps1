param([string]$Executable = (Join-Path $PSScriptRoot '..\target\release\TerminalIME.exe'))
$ErrorActionPreference = 'Stop'
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
 public delegate bool Callback(IntPtr window,IntPtr data);
 [DllImport("user32.dll")] public static extern bool EnumWindows(Callback callback,IntPtr data);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr window,StringBuilder name,int count);
 [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr window,uint command);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern uint RegisterWindowMessageW(string name);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowW(string name, string title);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint message, UIntPtr wparam, IntPtr lparam);
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
function Wait-Condition([scriptblock]$Condition, [string]$Message) {
    $deadline = [DateTime]::UtcNow.AddSeconds(10)
    while (!(& $Condition)) {
        if ([DateTime]::UtcNow -gt $deadline) { throw $Message }
        Start-Sleep -Milliseconds 100
    }
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
    & (Join-Path $PSScriptRoot 'click-desktop.ps1') -ProcessId $gui.Id -X 405 -Y 383
    Wait-Condition { [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window) -ne [IntPtr]::Zero } 'Custom color dialog did not open (or GUI crashed).'
    $dialog = [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window)
    # Use the documented common-dialog selection message, not private control IDs.
    $setRgbMessage = [TerminalImeDesktopCheck]::RegisterWindowMessageW('commdlg_SetRGBColor')
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,$setRgbMessage,[UIntPtr]::Zero,[IntPtr]0xff0000)
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,0x111,[UIntPtr]1,[IntPtr]::Zero)
    Wait-Condition { (Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json).chinese_color -eq '#0000FF' } 'Custom color was not saved.'
    & (Join-Path $PSScriptRoot 'click-desktop.ps1') -ProcessId $gui.Id -X 405 -Y 383
    Wait-Condition { [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window) -ne [IntPtr]::Zero } 'Second color dialog did not open.'
    $dialog = [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window)
    $null = [TerminalImeDesktopCheck]::PostMessageW($dialog,0x111,[UIntPtr]2,[IntPtr]::Zero)
    Wait-Condition { [TerminalImeDesktopCheck]::FindDialog($gui.Id,$window) -eq [IntPtr]::Zero } 'Color cancel did not close the dialog.'
    if ((Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json).chinese_color -ne '#0000FF') { throw 'Cancel changed the saved color.' }
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]2,[IntPtr]::Zero)
    Wait-Condition { !(Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json).enabled } 'Tray pause did not persist.'
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]2,[IntPtr]::Zero)
    Wait-Condition { (Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json).enabled } 'Tray resume did not persist.'
    Start-Sleep -Seconds 3
    $runtime = Get-Content -LiteralPath (Join-Path $dataDir 'runtime.json') -Raw | ConvertFrom-Json
    if ($legacyTested -and $runtime.sessions.shell_pid -notcontains $existingSession.shell_pid) { throw 'Desktop did not take over the legacy console.' }
    $ownedProcesses = @($runtime.worker_pids) + @(Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -eq $gui.Id -and $_.Name -eq 'terminal-ime.exe' } | ForEach-Object ProcessId)
    $children = @($ownedProcesses | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
    $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]3,[IntPtr]::Zero)
    if (!$gui.WaitForExit(5000)) { throw 'Tray exit did not stop the application.' }
    foreach ($child in $children) { if (!$child.WaitForExit(5000)) { throw "Worker/agent $($child.Id) survived the application." } }
    $verdict = [PSCustomObject]@{GuiSubsystem=$true;CommonControlsManifest=$true;SettingsVisible=$true;TrayOwned=$true;CloseHides=$true;SingletonRestores=$true;CustomColorSaved=$true;ColorCancelPreservesValue=$true;PauseResumePersisted=$true;LegacyHandoffTested=$legacyTested;InitializedSessions=$runtime.sessions.Count;ExitCleansWorkers=$true;DataDirectory=$dataDir}
    $verdict | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $projectRoot 'artifacts\desktop-check-verdict.json') -Encoding utf8NoBOM
    $verdict | ConvertTo-Json
} finally {
    if ($legacyTested) { & $backend stop }
    $env:WT_SESSION = $oldSession
    $env:TERMINAL_IME_DATA_DIR = $oldDataDirectory
    if ($tray -ne [IntPtr]::Zero -and $gui -and !$gui.HasExited) {
        $null = [TerminalImeDesktopCheck]::PostMessageW($tray,0x111,[UIntPtr]3,[IntPtr]::Zero)
        $null = $gui.WaitForExit(5000)
    }
}
