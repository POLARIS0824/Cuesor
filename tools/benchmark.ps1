param([ValidateRange(1, 100)][int]$Samples = 30)
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$outputDir = Join-Path $projectRoot 'artifacts\benchmark'
New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
$nuExe = (Get-Command nu.exe).Source
$userConfig = (& $nuExe -n -c '$nu.config-path' | Out-String).Trim()
$integration = Join-Path $projectRoot 'integrations\nushell.nu'
if (@($userConfig, $integration, $outputDir) | Where-Object { $_.Contains("'") }) {
    throw 'Benchmark paths containing single quotes are not supported.'
}

# AttachConsole changes process console handles. Keep the controller detached
# from test consoles by doing input injection in a short-lived Python process.
$inputHelper = Join-Path $PSScriptRoot 'console-input.py'

$rows = [System.Collections.Generic.List[object]]::new()
for ($sampleIndex = 0; $sampleIndex -lt $Samples; $sampleIndex++) {
    # Alternate order to reduce cache/temperature bias.
    $states = if ($sampleIndex % 2 -eq 0) { @($false, $true) } else { @($true, $false) }
    foreach ($enabled in $states) {
        $label = '{0:D2}-{1}' -f $sampleIndex, $enabled
        $configFile = Join-Path $outputDir "$label-config.nu"
        $entryFile = Join-Path $outputDir "$label-entry.nu"
        $readyFile = Join-Path $outputDir "$label-ready.json"
        $metricFile = Join-Path $outputDir "$label-time.txt"
        # These files are local test artifacts, not changes to the user's configuration.
        $configText = "source '$($userConfig.Replace('\','/'))'`n"
        if ($enabled) { $configText += "source '$($integration.Replace('\','/'))'`n" }
        Set-Content -LiteralPath $configFile -Value $configText -Encoding utf8NoBOM
        $readyPath = $readyFile.Replace('\','/')
        $entryText = '{pid: $nu.pid, session: $env.WT_SESSION} | to json | save --force ' + "'$readyPath'"
        Set-Content -LiteralPath $entryFile -Value $entryText -Encoding utf8NoBOM
        # Remove only exact prior artifacts so a stale result cannot masquerade as a pass.
        foreach ($oldFile in @($readyFile, $metricFile)) {
            if (Test-Path -LiteralPath $oldFile) { Remove-Item -LiteralPath $oldFile }
        }
        $arguments = @('-w','Cuesor-benchmark','new-tab','-p','Windows PowerShell',
            '--title','Cuesor startup benchmark','--suppressApplicationTitle','-d',$projectRoot,
            $nuExe,'--no-history','--config',$configFile,'-e',"source $($entryFile.Replace('\','/'))")
        & wt.exe @arguments
        if ($LASTEXITCODE -ne 0) { throw "Windows Terminal launch failed: $label" }
        $deadline = [DateTime]::UtcNow.AddSeconds(15)
        while (!(Test-Path -LiteralPath $readyFile)) {
            if ([DateTime]::UtcNow -gt $deadline) { throw "Nu did not initialize: $label" }
            Start-Sleep -Milliseconds 10
        }
        $ready = Get-Content -LiteralPath $readyFile -Raw | ConvertFrom-Json
        $testNu = Get-Process -Id $ready.pid
        if ($testNu.ProcessName -ne 'nu') { throw 'Ready PID is not Nushell.' }
        try {
            # Queued input executes AFTER the first prompt; startup-time is then final.
            $metricPath = $metricFile.Replace('\','/')
            $command = '$nu.startup-time | into int | save --force ' + "'$metricPath'; exit 0"
            $commandFile = Join-Path $outputDir "$label-command.txt"
            Set-Content -LiteralPath $commandFile -Value $command -NoNewline -Encoding utf8NoBOM
            & python $inputHelper --pid $ready.pid --command-file $commandFile
            while (!(Test-Path -LiteralPath $metricFile)) {
                if ([DateTime]::UtcNow -gt $deadline) { throw "No first-prompt metric: $label" }
                Start-Sleep -Milliseconds 10
            }
            $durationMs = [double](Get-Content -LiteralPath $metricFile -Raw).Trim() / 1000000
            if ($durationMs -le 0) { throw "Invalid startup duration: $durationMs" }
            if (!$testNu.WaitForExit(3000)) { throw "Benchmark Nu did not exit: $label" }
            $cleaned = $true
            if ($enabled) {
                $env:WT_SESSION = $ready.session
                $tool = Join-Path $projectRoot 'target\release\cuesor-engine.exe'
                for ($retry = 0; $retry -lt 20; $retry++) {
                    $state = & $tool status | ConvertFrom-Json
                    if (!$state.running) { break }
                    Start-Sleep -Milliseconds 25
                }
                $cleaned = !$state.running
                if (!$cleaned) { throw "Worker survived its root Nu: $label" }
            }
            $rows.Add([PSCustomObject]@{Sample=$sampleIndex;Enabled=$enabled;StartupMs=$durationMs;WorkerCleanup=$cleaned})
        } finally {
            if (!$testNu.HasExited) { Stop-Process -Id $testNu.Id -ErrorAction SilentlyContinue }
        }
        $rows | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $outputDir 'samples.json') -Encoding utf8NoBOM
        Write-Output ("sample={0} enabled={1} startup_ms={2:F2}" -f $sampleIndex,$enabled,$durationMs)
    }
}
function Percentile($values, [double]$fraction) {
    $sorted = @($values | Sort-Object)
    $sorted[[Math]::Max(0,[Math]::Ceiling($sorted.Count*$fraction)-1)]
}
$off = @($rows | Where-Object { !$_.Enabled } | Select-Object -ExpandProperty StartupMs)
$on = @($rows | Where-Object { $_.Enabled } | Select-Object -ExpandProperty StartupMs)
$summary = [PSCustomObject]@{
    SamplesPerState=$Samples
    DisabledMedianMs=(Percentile $off 0.5);EnabledMedianMs=(Percentile $on 0.5)
    DisabledP95Ms=(Percentile $off 0.95);EnabledP95Ms=(Percentile $on 0.95)
    MedianDeltaMs=((Percentile $on 0.5)-(Percentile $off 0.5))
    P95DeltaMs=((Percentile $on 0.95)-(Percentile $off 0.95))
    Measurement='Nu process creation to first prompt ($nu.startup-time); actual user env/config; --no-history; normal integrity; sequential alternating cases'
}
$summary | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $outputDir 'summary.json') -Encoding utf8NoBOM
$summary | ConvertTo-Json
