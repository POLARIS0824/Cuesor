$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$reportDir = Join-Path $projectRoot ('artifacts\load-' + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
$nuExe = (Get-Command nu.exe).Source
$tool = Join-Path $projectRoot 'target\release\cuesor-engine.exe'
$nuScript = Join-Path $PSScriptRoot 'load-check.nu'
$records = @()
$results = @()
$launched = 0
try {
    foreach ($count in @(1,5,10)) {
        while ($launched -lt $count) {
            $arguments = @('-w','Cuesor-load-test','new-tab','-p','Windows PowerShell',
                '--title','Cuesor load test','--suppressApplicationTitle','-d',$projectRoot,
                $nuExe,'-n','--no-history',$nuScript,$reportDir)
            & wt.exe @arguments
            $launched++
        }
        $deadline = [DateTime]::UtcNow.AddSeconds(15)
        do {
            $records = @(Get-ChildItem -LiteralPath $reportDir -Filter 'session-*.json' | ForEach-Object {
                try { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json } catch { }
            })
            if ([DateTime]::UtcNow -gt $deadline) { throw "Only $($records.Count)/$count workers initialized." }
            if ($records.Count -lt $count) { Start-Sleep -Milliseconds 50 }
        } while ($records.Count -lt $count)
        $processes = @($records | ForEach-Object { Get-Process -Id $_.worker_pid })
        $cpuBefore = ($processes | Measure-Object -Property CPU -Sum).Sum
        $timer = [System.Diagnostics.Stopwatch]::StartNew()
        Start-Sleep -Seconds 10
        $processes = @($records | ForEach-Object { Get-Process -Id $_.worker_pid })
        $cpuAfter = ($processes | Measure-Object -Property CPU -Sum).Sum
        $corePercent = 100 * ($cpuAfter - $cpuBefore) / $timer.Elapsed.TotalSeconds
        $result = [PSCustomObject]@{
            Workers=$count;SampleSeconds=$timer.Elapsed.TotalSeconds
            PrivateMiB=[Math]::Round((($processes | Measure-Object -Property PrivateMemorySize64 -Sum).Sum / 1MB),2)
            OneCoreCPUPercent=[Math]::Round($corePercent,3)
            MachineCPUPercent=[Math]::Round($corePercent/[Environment]::ProcessorCount,3)
            Measurement='10-second sample; fresh noninteractive Nu hosts in the same WT test window; no synthetic key events'
        }
        $results += $result
        $result | ConvertTo-Json
    }
} finally {
    # Operate only on PIDs in this unique run directory, after checking executable names.
    $records = @(Get-ChildItem -LiteralPath $reportDir -Filter 'session-*.json' | ForEach-Object {
        try { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json } catch { }
    })
    foreach ($record in $records) {
        $env:WT_SESSION = $record.session
        & $tool stop
        $workerProcess = Get-Process -Id $record.worker_pid -ErrorAction SilentlyContinue
        $shellProcess = Get-Process -Id $record.shell_pid -ErrorAction SilentlyContinue
        if ($workerProcess -and $workerProcess.Path -eq $tool) { throw "Test worker failed to stop: $($record.worker_pid)" }
        if ($shellProcess -and $shellProcess.ProcessName -eq 'nu') {
            $shellDetails = Get-CimInstance Win32_Process -Filter "ProcessId=$($shellProcess.Id)"
            if ($shellDetails.CommandLine.Contains($nuScript) -and $shellDetails.CommandLine.Contains($reportDir)) {
                Stop-Process -Id $shellProcess.Id
            }
        }
    }
    $results | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $reportDir 'summary.json') -Encoding utf8NoBOM
    Write-Output "Report: $reportDir"
}
