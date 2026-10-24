param([int]$ProcessId = 0)
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CuesorAccessCheck {
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr process,uint access,out IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetTokenInformation(IntPtr token,int info,out int value,int length,out int needed);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
 public static bool Elevated(uint pid) {
  var process=OpenProcess(0x1000,false,pid);
  if(process==IntPtr.Zero) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
  try {
   IntPtr token;
   if(!OpenProcessToken(process,8,out token)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
   try {
    int value,needed;
    if(!GetTokenInformation(token,20,out value,4,out needed)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
    return value!=0;
   } finally { CloseHandle(token); }
  } finally { CloseHandle(process); }
 }
}
'@
$gui = if ($ProcessId) { Get-Process -Id $ProcessId } else { Get-Process -Name Cuesor | Select-Object -First 1 }
if (!$gui -or $gui.ProcessName -ne 'Cuesor') { throw 'Open Cuesor before running this check.' }
$runtime = Get-Content -LiteralPath "$env:LOCALAPPDATA\Cuesor\runtime.json" -Raw | ConvertFrom-Json
if ($runtime.controller_pid -ne $gui.Id) { throw 'Runtime status does not belong to the current GUI.' }
$elevated = [CuesorAccessCheck]::Elevated($gui.Id)
$terminals = @(Get-Process -Name WindowsTerminal -ErrorAction SilentlyContinue)
if ($terminals.Count -eq 0) { throw 'Open a Windows Terminal session; permission handling cannot be verified without one.' }
$blocked = @($terminals | Where-Object { !$elevated -and [CuesorAccessCheck]::Elevated($_.Id) } | ForEach-Object Id)
foreach ($terminalId in $blocked) {
    if ($runtime.elevated_terminal_pids -notcontains $terminalId) {
        throw "REPRODUCED: elevated Windows Terminal $terminalId exists, but the GUI reports waiting without a permission diagnosis."
    }
}
if ($elevated -and $terminals.Count -gt 0 -and $runtime.sessions.Count -eq 0) {
    throw 'Administrator GUI still has no initialized terminal sessions.'
}
[pscustomobject]@{
    GuiPid = $gui.Id
    GuiElevated = $elevated
    BlockedTerminalPids = $blocked
    DiagnosedTerminalPids = @($runtime.elevated_terminal_pids)
    InitializedSessions = $runtime.sessions.Count
    Errors = $runtime.errors
} | ConvertTo-Json
