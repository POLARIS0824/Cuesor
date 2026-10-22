param([Parameter(Mandatory)][int]$ProcessId,[Parameter(Mandatory)][double]$X,[Parameter(Mandatory)][double]$Y)
$ErrorActionPreference = 'Stop'
if (!('TerminalImeOwnedInput' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TerminalImeOwnedInput {
 [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
 [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point point);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
 [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window, ref Point point);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr window, uint message, UIntPtr wparam, IntPtr lparam);
}
'@
}
$process = Get-Process -Id $ProcessId
if ($process.ProcessName -ne 'TerminalIME') { throw 'Only send input to the TerminalIME settings preview.' }
$window = $process.MainWindowHandle
if ($window -eq 0 -or ![TerminalImeOwnedInput]::IsWindowVisible($window)) { throw 'Settings is hidden.' }
$oldContext = [TerminalImeOwnedInput]::SetThreadDpiAwarenessContext([IntPtr](-4))
$scale = [TerminalImeOwnedInput]::GetDpiForWindow($window)/96.0
$point = [TerminalImeOwnedInput+Point]::new()
$point.X = [int][Math]::Round($X*$scale)
$point.Y = [int][Math]::Round($Y*$scale)
$packed = [IntPtr](($point.Y -shl 16) -bor ($point.X -band 0xffff))
$null = [TerminalImeOwnedInput]::ClientToScreen($window,[ref]$point)
$oldPoint = [TerminalImeOwnedInput+Point]::new()
$null = [TerminalImeOwnedInput]::GetCursorPos([ref]$oldPoint)
try {
    $null = [TerminalImeOwnedInput]::SetCursorPos($point.X,$point.Y)
    $null = [TerminalImeOwnedInput]::PostMessageW($window,0x200,[UIntPtr]::Zero,$packed)
    Start-Sleep -Milliseconds 200
    $null = [TerminalImeOwnedInput]::PostMessageW($window,0x201,[UIntPtr]1,$packed)
    Start-Sleep -Milliseconds 50
    $null = [TerminalImeOwnedInput]::PostMessageW($window,0x202,[UIntPtr]::Zero,$packed)
    Start-Sleep -Milliseconds 400
} finally {
    $null = [TerminalImeOwnedInput]::SetCursorPos($oldPoint.X,$oldPoint.Y)
    $null = [TerminalImeOwnedInput]::SetThreadDpiAwarenessContext($oldContext)
}
