param([switch]$Debug)
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$previousCompiler = $env:GPUI_FXC_PATH
$previousDllTool = $env:CUESOR_DLLTOOL
$previousPath = $env:PATH
Push-Location -LiteralPath $projectRoot
try {
    if ((& rustc --print cfg) -contains 'target_env="gnu"') {
        $realDllTool = if ($env:CUESOR_DLLTOOL) { $env:CUESOR_DLLTOOL } else { (Get-Command dlltool.exe -ErrorAction Stop).Source }
        $wrapperDir = Join-Path $projectRoot 'target\build-tools\isolated-dlltool'
        New-Item -ItemType Directory -Path $wrapperDir -Force | Out-Null
        & rustc (Join-Path $PSScriptRoot 'dlltool-wrapper.rs') --edition 2021 -O -o (Join-Path $wrapperDir 'dlltool.exe')
        if ($LASTEXITCODE -ne 0) { throw 'Import-library helper build failed.' }
        $env:CUESOR_DLLTOOL = $realDllTool
        $env:PATH = "$wrapperDir;$env:PATH"
    }
    if (!$Debug -and !$env:GPUI_FXC_PATH -and !(Get-Command fxc.exe -ErrorAction SilentlyContinue)) {
        $compilerDir = Join-Path $projectRoot 'target\build-tools'
        New-Item -ItemType Directory -Path $compilerDir -Force | Out-Null
        $compilerExe = Join-Path $compilerDir 'cuesor-shader-compiler.exe'
        & rustc (Join-Path $PSScriptRoot 'shader-compiler.rs') --edition 2021 -O -o $compilerExe
        if ($LASTEXITCODE -ne 0) { throw 'Shader compiler adapter build failed.' }
        $env:GPUI_FXC_PATH = $compilerExe
    }
    $buildArguments = @('build','--workspace','--locked','-j','1')
    if (!$Debug) { $buildArguments += '--release' }
    & cargo @buildArguments
    if ($LASTEXITCODE -ne 0) { throw 'Desktop build failed.' }
} finally {
    $env:GPUI_FXC_PATH = $previousCompiler
    $env:CUESOR_DLLTOOL = $previousDllTool
    $env:PATH = $previousPath
    Pop-Location
}
