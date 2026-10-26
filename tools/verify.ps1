#Requires -Version 7.0
param(
    [Parameter(Mandatory)][ValidateSet('Launch','Doctor','Drive','Snapshot','Cleanup')][string]$Action,
    [Parameter(Mandatory)][ValidatePattern('^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$')][string]$RunId,
    [string]$Executable = (Join-Path $PSScriptRoot '..\target\release\Cuesor.exe'),
    [ValidateSet('Appearance','Colors','Tray','Pause')][string]$Feature = 'Appearance'
)
$ErrorActionPreference = 'Stop'
if (!$IsWindows) { throw 'This driver needs an interactive Windows desktop and PowerShell 7.' }
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class CuesorVerify {
    public delegate bool Callback(IntPtr window, IntPtr data);
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
    public struct Entry {
        public uint Size, Usage, Pid; public UIntPtr Heap; public uint Module, Threads, Parent;
        public int Priority; public uint Flags;
        [MarshalAs(UnmanagedType.ByValTStr,SizeConst=260)] public string Name;
    }
    public class ProcessEntry { public uint Pid, Parent; public string Name; }
    [DllImport("kernel32.dll")] static extern IntPtr CreateToolhelp32Snapshot(uint flags,uint pid);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] static extern bool Process32FirstW(IntPtr h,ref Entry e);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] static extern bool Process32NextW(IntPtr h,ref Entry e);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
    public static ProcessEntry[] Processes() {
        var h=CreateToolhelp32Snapshot(2,0);
        if(h==new IntPtr(-1)) throw new InvalidOperationException("Cannot enumerate processes.");
        try {
            var e=new Entry(); e.Size=(uint)Marshal.SizeOf(e);
            var entries=new List<ProcessEntry>();
            if(!Process32FirstW(h,ref e)) throw new InvalidOperationException("Cannot read process snapshot.");
            do { entries.Add(new ProcessEntry { Pid=e.Pid,Parent=e.Parent,Name=e.Name }); }
            while(Process32NextW(h,ref e));
            return entries.ToArray();
        } finally { CloseHandle(h); }
    }
    [DllImport("user32.dll")] public static extern bool EnumWindows(Callback cb,IntPtr data);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h,StringBuilder text,int count);
    [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr h,uint cmd);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowW(string cls,string title);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h,uint msg,UIntPtr w,IntPtr l);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern uint RegisterWindowMessageW(string name);
    public static IntPtr Dialog(uint pid,IntPtr owner) {
        IntPtr found=IntPtr.Zero;
        EnumWindows((h,data)=> {
            uint actual; GetWindowThreadProcessId(h,out actual);
            var name=new StringBuilder(256); GetClassNameW(h,name,256);
            if(actual==pid && GetWindow(h,4)==owner && name.ToString()=="#32770") found=h;
            return found==IntPtr.Zero;
        },IntPtr.Zero);
        return found;
    }
}
'@
$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$runDir = Join-Path $repo "artifacts\verify\$RunId"
$dataDir = Join-Path $runDir 'data'
$evidence = Join-Path $runDir 'evidence'
$statePath = Join-Path $runDir 'state.json'
$state = $null
function Write-Json($Value,[string]$Path) {
    $Value | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath $Path -Encoding utf8NoBOM
}
function Read-Json([string]$Path) {
    $stream = [IO.FileStream]::new($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    $reader = [IO.StreamReader]::new($stream)
    try { $reader.ReadToEnd() | ConvertFrom-Json } finally { $reader.Dispose() }
}
function Read-ActiveColors([string]$Path) {
    $settings = Read-Json $Path
    $settings.color_schemes[$settings.active_scheme]
}
function Record([string]$Event,$Detail) {
    [ordered]@{time_utc=[DateTime]::UtcNow.ToString('o'); event=$Event; detail=$Detail} |
        ConvertTo-Json -Depth 12 -Compress | Add-Content -LiteralPath (Join-Path $evidence 'actions.jsonl') -Encoding utf8NoBOM
}
function Wait-For([scriptblock]$Condition,[string]$Failure) {
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    while (!(& $Condition)) {
        if ([DateTime]::UtcNow -ge $deadline) { throw $Failure }
        Start-Sleep -Milliseconds 100
    }
}
function Identity($Process,[string]$LaunchedPath) {
    $current = Get-Process -Id $Process.Id -ErrorAction Stop
    $path = $current.Path
    if (!$path) { $path = $LaunchedPath }
    if (!$path) { throw "Cannot identify the executable for PID $($Process.Id)." }
    [pscustomobject]@{pid=$current.Id; start_ticks=$current.StartTime.ToUniversalTime().Ticks; path=$path}
}
function Get-Owned($Owner) {
    if (!$Owner) { return $null }
    $process = Get-Process -Id $Owner.pid -ErrorAction SilentlyContinue
    if (!$process) { return $null }
    if ($process.StartTime.ToUniversalTime().Ticks -ne $Owner.start_ticks -or $process.Path -ne $Owner.path) {
        throw "PID $($Owner.pid) no longer matches this run's recorded process."
    }
    $process
}
function Save-State { Write-Json $state $statePath }
function Remember-Children {
    $entries = [CuesorVerify]::Processes()
    $parents = @($state.gui.pid)
    for ($depth=0; $depth -lt 6 -and $parents.Count; $depth++) {
        $children = @($entries | Where-Object { $_.Parent -in $parents -and $_.Name -ieq 'cuesor-engine.exe' })
        $parents = @($children | ForEach-Object Pid)
        foreach ($entry in $children) {
            $process = Get-Process -Id $entry.Pid -ErrorAction SilentlyContinue
            if ($process -and $process.Path -eq $state.backend) {
                $owner = Identity $process
                if (!@($state.children | Where-Object { $_.pid -eq $owner.pid -and $_.start_ticks -eq $owner.start_ticks }).Count) {
                    $state.children = @($state.children) + @($owner)
                }
            }
        }
    }
    Save-State
}
function Get-Gui {
    $process = Get-Owned $state.gui
    if (!$process) { throw 'The owned GUI is no longer running; inspect evidence and start a fresh run.' }
    $process.Refresh()
    $process
}
function Get-Control([string]$Name) {
    $gui = Get-Gui
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($gui.MainWindowHandle)
    $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,$Name)
    $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
}
function Invoke-Control([string]$Name) {
    Record 'action' @{method='UIAutomation'; name=$Name}
    Wait-For { $candidate = Get-Control $Name; $candidate -and $candidate.Current.IsEnabled } "Control is missing or disabled: $Name"
    $control = Get-Control $Name
    $pattern = $null
    if ($control.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern,[ref]$pattern)) { $pattern.Invoke() }
    elseif ($control.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern,[ref]$pattern)) { $pattern.Toggle() }
    else { throw "Control has no Invoke/Toggle pattern: $Name" }
}
function Assert-Selected([string]$Name) {
    $control = Get-Control $Name
    if (!$control -or $control.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState -ne [System.Windows.Automation.ToggleState]::On) {
        throw "Control was not restored as selected: $Name"
    }
}
function Get-Tray {
    $tray = [CuesorVerify]::FindWindowW('Cuesor.Tray','Cuesor tray')
    $ownerId = [uint32]0
    $null = [CuesorVerify]::GetWindowThreadProcessId($tray,[ref]$ownerId)
    if ($tray -eq [IntPtr]::Zero -or $ownerId -ne $state.gui.pid) { throw 'Tray HWND does not belong to the owned GUI.' }
    $tray
}
function Tray-Command([int]$Command) {
    Record 'action' @{method='owned tray WM_COMMAND'; command=$Command; explorer_gesture=$false}
    $null = [CuesorVerify]::PostMessageW((Get-Tray),0x111,[UIntPtr]$Command,[IntPtr]::Zero)
}
function Doctor {
    $gui = Get-Gui
    if ((Get-FileHash -LiteralPath $state.executable).Hash -ne $state.gui_sha256 -or (Get-FileHash -LiteralPath $state.backend).Hash -ne $state.backend_sha256) {
        throw 'Executable bytes changed since Launch; clean up and launch the intended build again.'
    }
    if (!$gui.MainWindowHandle -or ![CuesorVerify]::IsWindowVisible($gui.MainWindowHandle)) { throw 'The owned settings window is hidden or unavailable.' }
    $null = Get-Tray
    foreach ($name in @('启用光标颜色指示','自定义中文输入光标颜色','外观：深色','配色6：暖砂')) {
        if (!(Get-Control $name)) { throw "Expected accessible control is missing: $name" }
    }
    $settings = Read-Json (Join-Path $dataDir 'settings.json')
    $runtime = Read-Json (Join-Path $dataDir 'runtime.json')
    if ($settings.color_schemes.Count -ne 6 -or $runtime.controller_pid -ne $gui.Id) { throw 'Settings/runtime do not identify a ready owned instance.' }
    $entries = [CuesorVerify]::Processes()
    if (!@($entries | Where-Object { $_.Parent -eq $gui.Id -and $_.Name -ieq 'cuesor-engine.exe' }).Count) { throw 'The discovery controller is not running.' }
    foreach ($workerId in $runtime.worker_pids) {
        $worker = Get-Process -Id $workerId -ErrorAction SilentlyContinue
        if (!$worker -or $worker.Path -ne $state.backend) { throw "Reported worker $workerId is not live at the expected path." }
    }
    [pscustomobject]@{ready=$true; gui_pid=$gui.Id; executable=$state.executable; gui_sha256=$state.gui_sha256; source_head=$state.source_head; data_directory=$dataDir; sessions=@($runtime.sessions).Count; errors=$runtime.errors; elevated_terminal_pids=$runtime.elevated_terminal_pids; cursor_colors_visually_verified=$false}
}
function Snapshot([string]$Label) {
    $gui = Get-Gui
    $prefix = [DateTime]::UtcNow.ToString('HHmmssfff') + '-' + $Label
    Start-Sleep -Milliseconds 350
    & (Join-Path $PSScriptRoot 'capture-window.ps1') -ProcessId $gui.Id -OutputPath (Join-Path $evidence "$prefix.png")
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($gui.MainWindowHandle)
    $controls = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition)
    $names = @($controls | ForEach-Object {
        $current = $_.Current
        if ($current.Name) {
            $toggle = $null
            $selected = if ($_.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern,[ref]$toggle)) { $toggle.Current.ToggleState.ToString() } else { $null }
            @{name=$current.Name; enabled=$current.IsEnabled; offscreen=$current.IsOffscreen; toggle=$selected}
        }
    })
    Write-Json $names (Join-Path $evidence "$prefix.uia.json")
    foreach ($file in @('settings.json','runtime.json','window.json')) {
        $path = Join-Path $dataDir $file
        if (Test-Path -LiteralPath $path) { Write-Json (Read-Json $path) (Join-Path $evidence "$prefix.$file") }
    }
    Remember-Children
    Record 'snapshot' @{prefix=$prefix; feature=$Feature}
}
function Start-Gui {
    $oldDataDir = $env:CUESOR_DATA_DIR
    try {
        $env:CUESOR_DATA_DIR = $dataDir
        Record 'action' @{method='launch'; executable=$state.executable; data_directory=$dataDir}
        # This is the interactive UI under test; background subprocesses are hidden by the app.
        $gui = Start-Process -FilePath $state.executable -PassThru -RedirectStandardError (Join-Path $evidence ('stderr-' + [guid]::NewGuid() + '.log'))
        $state.gui = Identity $gui $state.executable
        Save-State
        Wait-For { $gui.Refresh(); $gui.HasExited -or $gui.MainWindowHandle -ne 0 } 'Settings did not create a window.'
        if ($gui.HasExited) { throw "GUI exited early with code $($gui.ExitCode); a singleton may already be running." }
        Wait-For { (Test-Path -LiteralPath (Join-Path $dataDir 'runtime.json')) -and (Read-Json (Join-Path $dataDir 'runtime.json')).controller_pid -eq $gui.Id } 'Discovery controller did not publish owned runtime status.'
        Wait-For { $null -ne (Get-Control '外观：深色') } 'Settings controls did not become usable.'
        Remember-Children
    } finally { $env:CUESOR_DATA_DIR = $oldDataDir }
}
function Stop-Gui {
    Remember-Children
    $gui = Get-Gui
    Tray-Command 3
    if (!$gui.WaitForExit(10000)) { throw 'Owned GUI did not exit through its tray handler.' }
    foreach ($owner in @($state.children)) {
        $child = Get-Owned $owner
        if ($child -and !$child.WaitForExit(10000)) { throw "Owned child $($owner.pid) survived normal exit." }
    }
    Record 'exit' @{gui_pid=$gui.Id; children_stopped=$true}
}
function Cleanup {
    $forced = @()
    if (Get-Owned $state.gui) {
        Remember-Children
        try { Stop-Gui } catch { Record 'graceful_cleanup_error' $_.Exception.Message }
    }
    foreach ($owner in @($state.gui) + @($state.children)) {
        $process = Get-Owned $owner
        if ($process) {
            $process.Kill()
            if (!$process.WaitForExit(10000)) { throw "Owned PID $($owner.pid) could not be stopped." }
            $forced += $owner.pid
        }
    }
    $archive = Join-Path $evidence 'final-data'
    if (Test-Path -LiteralPath $dataDir) {
        New-Item -ItemType Directory -Path $archive -Force | Out-Null
        Get-ChildItem -LiteralPath $dataDir -File | Copy-Item -Destination $archive -Force
        $resolved = (Resolve-Path -LiteralPath $dataDir).Path
        if ($resolved -ne (Join-Path $runDir 'data') -or !(Test-Path -LiteralPath $statePath)) { throw 'Unexpected cleanup target.' }
        if (@(Get-Item -LiteralPath $resolved; Get-ChildItem -LiteralPath $resolved -Recurse -Force) | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }) { throw 'Refusing to remove a data directory containing reparse points.' }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
    $state.cleaned = $true
    Save-State
    $files = @(Get-ChildItem -LiteralPath $evidence -File -Recurse | Select-Object -ExpandProperty FullName)
    $result = [ordered]@{owned_processes_stopped=$true; forced_pids=$forced; data_removed=!(Test-Path -LiteralPath $dataDir); evidence_directory=$evidence; evidence_files=$files; evidence_preserved=($files.Count -gt 0)}
    Write-Json $result (Join-Path $evidence 'cleanup.json')
    Record 'cleanup' $result
    [pscustomobject]$result
}
function Set-Color([bool]$Chinese,[int]$Rgb,[switch]$Cancel) {
    Invoke-Control $(if ($Chinese) {'自定义中文输入光标颜色'} else {'自定义英文输入光标颜色'})
    $gui = Get-Gui
    Wait-For { [CuesorVerify]::Dialog($gui.Id,$gui.MainWindowHandle) -ne [IntPtr]::Zero } 'Native color dialog did not open.'
    $dialog = [CuesorVerify]::Dialog($gui.Id,$gui.MainWindowHandle)
    $bgr = (($Rgb -band 255) -shl 16) -bor ($Rgb -band 65280) -bor ($Rgb -shr 16)
    Record 'action' @{method='native common color dialog'; rgb=('#{0:X6}' -f $Rgb); cancel=[bool]$Cancel}
    $message = [CuesorVerify]::RegisterWindowMessageW('commdlg_SetRGBColor')
    $null = [CuesorVerify]::PostMessageW($dialog,$message,[UIntPtr]::Zero,[IntPtr]$bgr)
    $null = [CuesorVerify]::PostMessageW($dialog,0x111,[UIntPtr]$(if ($Cancel) {2} else {1}),[IntPtr]::Zero)
    Wait-For { [CuesorVerify]::Dialog($gui.Id,$gui.MainWindowHandle) -eq [IntPtr]::Zero } 'Native color dialog did not close.'
    if (!$Cancel) {
        $field = if ($Chinese) {'chinese_color'} else {'english_color'}
        Wait-For { (Read-ActiveColors (Join-Path $dataDir 'settings.json')).$field -eq ('#{0:X6}' -f $Rgb) } 'Color did not persist.'
    }
}
try {
    if ($Action -eq 'Launch') {
        if (Test-Path -LiteralPath $runDir) { throw 'RunId already exists; keep its evidence and choose a new RunId.' }
        $conflicts = @([CuesorVerify]::Processes() | Where-Object { $_.Name -iin @('Cuesor.exe','cuesor-engine.exe') })
        if ($conflicts.Count) { throw "Existing GUI/worker PIDs $($conflicts.Pid -join ', ') prevent isolated verification. Exit the daily app through its tray first." }
        $program = (Resolve-Path -LiteralPath $Executable).Path
        if ([IO.Path]::GetFileName($program) -cne 'Cuesor.exe') { throw 'Select the GUI entry Cuesor.exe, not the backend cuesor-engine.exe.' }
        $backend = Join-Path ([IO.Path]::GetDirectoryName($program)) 'cuesor-engine.exe'
        if (!(Test-Path -LiteralPath $backend)) { throw 'The GUI requires cuesor-engine.exe beside it.' }
        New-Item -ItemType Directory -Path $dataDir,$evidence | Out-Null
        $state = [pscustomobject]@{run_id=$RunId; executable=$program; backend=$backend; gui_sha256=(Get-FileHash -LiteralPath $program).Hash; backend_sha256=(Get-FileHash -LiteralPath $backend).Hash; source_head=(& git -C $repo rev-parse HEAD); backend_version=(& $backend --version); gui=$null; children=@(); cleaned=$false}
        Save-State
        Start-Gui
        $health = Doctor
        Record 'doctor' $health
        Snapshot 'launch'
        $health | ConvertTo-Json -Depth 8
    } else {
        $state = Read-Json $statePath
        if ($state.cleaned -and $Action -ne 'Cleanup') { throw 'This run was cleaned; use a new RunId for another launch.' }
        switch ($Action) {
            'Doctor' { Doctor | ConvertTo-Json -Depth 8 }
            'Snapshot' { $null = Doctor; Snapshot 'requested'; $evidence }
            'Cleanup' { Cleanup | Select-Object -ExcludeProperty evidence_files | ConvertTo-Json -Depth 8 }
            'Drive' {
                $health = Doctor
                Record 'doctor' $health
                Snapshot "$Feature-before"
                $settingsPath = Join-Path $dataDir 'settings.json'
                switch ($Feature) {
                    'Appearance' {
                        foreach ($pair in @(@('浅色','light'),@('深色','dark'),@('系统','system'),@('深色','dark'))) {
                            Invoke-Control ('外观：' + $pair[0])
                            Wait-For { (Read-Json $settingsPath).appearance -eq $pair[1] } 'Appearance did not persist.'
                            Assert-Selected ('外观：' + $pair[0])
                            Snapshot ('appearance-' + $pair[1])
                        }
                        Stop-Gui
                        Start-Gui
                        $null = Doctor
                        Assert-Selected '外观：深色'
                        if ((Read-Json $settingsPath).appearance -ne 'dark') { throw 'Dark appearance was lost after restart.' }
                        Snapshot 'appearance-reopened'
                    }
                    'Colors' {
                        $names = @('经典','珊瑚','暮紫','极光','柔粉','暖砂')
                        $pairs = @()
                        for ($i=0; $i -lt 6; $i++) {
                            Invoke-Control ('配色{0}：{1}' -f ($i+1),$names[$i])
                            Wait-For { (Read-Json $settingsPath).active_scheme -eq $i } 'Scheme did not select.'
                            $saved = Read-Json $settingsPath
                            if ($saved.PSObject.Properties.Name -contains 'chinese_color' -or $saved.PSObject.Properties.Name -contains 'english_color') { throw 'Settings still contain duplicated active colors.' }
                            $cn = 0xCC3300 + $i*0x111
                            $en = 0x0077AA + $i*0x1010
                            Set-Color $true $cn
                            Set-Color $false $en
                            $pairs += @{cn=('#{0:X6}' -f $cn); en=('#{0:X6}' -f $en)}
                            Snapshot "scheme-$($i+1)-edited"
                        }
                        Set-Color $true 0x123456 -Cancel
                        if ((Read-ActiveColors $settingsPath).chinese_color -ne $pairs[5].cn) { throw 'Cancelling changed the saved color.' }
                        Stop-Gui
                        Start-Gui
                        $null = Doctor
                        Assert-Selected '配色6：暖砂'
                        if ((Read-Json $settingsPath).active_scheme -ne 5) { throw 'Active scheme was lost after restart.' }
                        for ($i=0; $i -lt 6; $i++) {
                            Invoke-Control ('配色{0}：{1}' -f ($i+1),$names[$i])
                            Wait-For { $saved=Read-Json $settingsPath; $saved.active_scheme -eq $i -and $saved.color_schemes[$i].chinese_color -eq $pairs[$i].cn -and $saved.color_schemes[$i].english_color -eq $pairs[$i].en } 'An independent color pair was lost after restart.'
                            Assert-Selected ('配色{0}：{1}' -f ($i+1),$names[$i])
                        }
                        $beforeReset = Read-Json $settingsPath
                        Invoke-Control '重置当前配色'
                        Wait-For { (Read-ActiveColors $settingsPath).chinese_color -eq '#A18072' } 'Current scheme reset failed.'
                        $saved = Read-Json $settingsPath
                        if ($saved.color_schemes[$saved.active_scheme].english_color -ne '#3E63DD') { throw 'Reset failed to restore both default colors.' }
                        for ($i=0; $i -lt 5; $i++) {
                            if ($saved.color_schemes[$i].chinese_color -ne $beforeReset.color_schemes[$i].chinese_color -or $saved.color_schemes[$i].english_color -ne $beforeReset.color_schemes[$i].english_color) { throw "Reset changed nonactive scheme $($i+1)." }
                        }
                        Snapshot 'colors-restored-and-reset'
                    }
                    'Pause' {
                        $before = (Read-Json $settingsPath).enabled
                        Invoke-Control '启用光标颜色指示'
                        Wait-For { (Read-Json $settingsPath).enabled -ne $before } 'Pause switch did not save.'
                        Snapshot 'pause-toggled'
                        Invoke-Control '启用光标颜色指示'
                        Wait-For { (Read-Json $settingsPath).enabled -eq $before } 'Pause switch did not restore.'
                        Snapshot 'pause-restored'
                    }
                    'Tray' {
                        $gui = Get-Gui
                        Record 'action' @{method='settings close WM_CLOSE'; gui_pid=$gui.Id}
                        $null = [CuesorVerify]::PostMessageW($gui.MainWindowHandle,0x10,[UIntPtr]::Zero,[IntPtr]::Zero)
                        Wait-For { ![CuesorVerify]::IsWindowVisible($gui.MainWindowHandle) } 'Close did not hide settings.'
                        if ($gui.HasExited) { throw 'Closing settings stopped the GUI.' }
                        $controllerAlive = @([CuesorVerify]::Processes() | Where-Object {$_.Parent -eq $gui.Id -and $_.Name -ieq 'cuesor-engine.exe'}).Count -gt 0
                        if (!$controllerAlive) { throw 'Closing settings stopped the controller.' }
                        Record 'hidden' @{gui_pid=$gui.Id; process_alive=$true; controller_alive=$true}
                        Record 'action' @{method='second executable launch'; executable=$state.executable}
                        $second = Start-Process -FilePath $state.executable -WindowStyle Hidden -PassThru
                        $state.children = @($state.children) + @(Identity $second $state.executable)
                        Save-State
                        if (!$second.WaitForExit(5000) -or $second.ExitCode -ne 0) { throw 'Second GUI did not exit successfully.' }
                        Wait-For { [CuesorVerify]::IsWindowVisible($gui.MainWindowHandle) } 'Second launch did not restore the original settings window.'
                        $before = (Read-Json $settingsPath).enabled
                        Tray-Command 2
                        Wait-For { (Read-Json $settingsPath).enabled -ne $before } 'Tray pause/resume handler did not save.'
                        Snapshot 'tray-toggled'
                        Tray-Command 2
                        Wait-For { (Read-Json $settingsPath).enabled -eq $before } 'Tray pause/resume handler did not restore.'
                        Snapshot 'tray-restored'
                    }
                }
                $verdict = [ordered]@{feature=$Feature; passed=$true; entry_point=$(if ($Feature -eq 'Tray') {'WM_CLOSE, second executable launch, owned tray command handler'} else {'Windows UI Automation settings controls'}); evidence_directory=$evidence; gui_sha256=$state.gui_sha256; backend_sha256=$state.backend_sha256; source_head=$state.source_head; source_rebuilt_by_driver=$false; cursor_colors_visually_verified=$false; explorer_tray_gestures_verified=$false; login_startup_verified=$false; uac_verified=$false}
                Write-Json $verdict (Join-Path $evidence 'verdict.json')
                Record 'verdict' $verdict
                $verdict | ConvertTo-Json -Depth 8
            }
        }
    }
} catch {
    if ($Action -ne 'Doctor' -and $state -and (Test-Path -LiteralPath $evidence)) {
        Record 'failure' @{action=$Action; feature=$Feature; message=$_.Exception.Message}
        if ($Action -in @('Launch','Drive')) {
            try { if (Get-Owned $state.gui) { Snapshot 'failure' } } catch { Record 'failure_snapshot_error' $_.Exception.Message }
            try { $null = Cleanup } catch { Record 'cleanup_failure' $_.Exception.Message }
        }
    }
    throw
}
