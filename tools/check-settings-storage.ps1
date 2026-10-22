param([string]$Executable = (Join-Path $PSScriptRoot '..\target\release\terminal-ime.exe'))
$ErrorActionPreference = 'Stop'
$program = (Resolve-Path -LiteralPath $Executable).Path
$directory = Join-Path $env:LOCALAPPDATA 'TerminalIME'
$prefix = 'storage-check-' + [guid]::NewGuid()
$report = Join-Path $directory ($prefix + '.json')
$errorLog = Join-Path (Resolve-Path (Join-Path $PSScriptRoot '..\artifacts')).Path ($prefix + '.log')
try {
    foreach ($attempt in 1..2) {
        $probe = Start-Process -FilePath $program -ArgumentList @('diagnose',('"' + $report + '"')) -WindowStyle Hidden -Wait -PassThru -RedirectStandardError $errorLog
        if ($probe.ExitCode -ne 0) {
            throw "Settings-directory save failed on attempt ${attempt}: $(Get-Content -LiteralPath $errorLog -Raw)"
        }
        $value = Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
        if ($null -eq $value.sessions -or $null -eq $value.elevated_terminal_pids) {
            throw 'Saved report is incomplete.'
        }
    }
    'PASS: creating and replacing JSON in the real settings directory.'
} finally {
    if (Test-Path -LiteralPath $report) { Remove-Item -LiteralPath $report }
    Get-ChildItem -LiteralPath $directory -Filter "$prefix.*.tmp" | ForEach-Object { Remove-Item -LiteralPath $_.FullName }
    if (Test-Path -LiteralPath $errorLog) { Remove-Item -LiteralPath $errorLog }
}
