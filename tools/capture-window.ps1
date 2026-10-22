param([Parameter(Mandatory)][int]$ProcessId, [Parameter(Mandatory)][string]$OutputPath)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TerminalImeCapture {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr window, IntPtr dc, uint flags);
    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
}
'@
$process = Get-Process -Id $ProcessId
if ($process.ProcessName -ne 'TerminalIME') { throw 'Only capture the TerminalIME settings window.' }
$window = $process.MainWindowHandle
if ($window -eq 0) { throw 'Settings window is not visible.' }
$null = [TerminalImeCapture]::SetThreadDpiAwarenessContext([IntPtr](-4))
if (![TerminalImeCapture]::IsWindowVisible($window)) { throw 'Settings window is hidden; refusing a screen capture.' }
$rectangle = [TerminalImeCapture+Rect]::new()
if (![TerminalImeCapture]::GetWindowRect($window, [ref]$rectangle)) { throw 'Cannot read settings bounds.' }
$bitmap = [System.Drawing.Bitmap]::new($rectangle.Right-$rectangle.Left, $rectangle.Bottom-$rectangle.Top)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
try {
    $deviceContext = $graphics.GetHdc()
    try {
        if (![TerminalImeCapture]::PrintWindow($window, $deviceContext, 2)) { throw 'Window-only capture failed.' }
    } finally { $graphics.ReleaseHdc($deviceContext) }
    $bitmap.Save([IO.Path]::GetFullPath($OutputPath), [System.Drawing.Imaging.ImageFormat]::Png)
} finally { $graphics.Dispose(); $bitmap.Dispose() }
