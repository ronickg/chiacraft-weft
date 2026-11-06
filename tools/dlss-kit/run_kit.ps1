# chiacraft DLSS test kit (Windows): runs the Bliss spots (Bliss's imported world and spot C) with each
# DLSS mode, a fresh world per run, and keeps the shots, frame times and logs in results\<time>\.
#
#   powershell -ExecutionPolicy Bypass -File run_kit.ps1            # all modes, both spots (~25 min, unattended)
#   powershell -ExecutionPolicy Bypass -File run_kit.ps1 -Quick     # off and DLAA at spot C only (~4 min)
#   powershell -ExecutionPolicy Bypass -File run_kit.ps1 -Dev       # NVIDIA's development DLLs (more in sl.log)
#   powershell -ExecutionPolicy Bypass -File run_kit.ps1 -Only dlaa,rr
#
# Nothing here changes Windows or the driver; close the game window only if a run hangs (each is
# stopped after 5 minutes anyway).
param([switch]$Quick, [switch]$Dev, [string]$Only = "", [string]$Size = "2560x1440")

$ErrorActionPreference = "Stop"
$Kit = $PSScriptRoot
$GameDir = Join-Path $Kit "chiacraft"
$Game = Join-Path $GameDir "weft.exe"
$Stamp = Get-Date -Format "yyyyMMdd-HHmm"
$Out = Join-Path $Kit "results\$Stamp"
New-Item -ItemType Directory -Force $Out | Out-Null
$Summary = Join-Path $Out "summary.txt"

$Sl = if ($Dev) { Join-Path $GameDir "streamline-dev" } else { Join-Path $GameDir "streamline" }
$env:WEFT_STREAMLINE_DIR = $Sl

# core:upscaler: 0 off, 1 DLAA, 2 SR Quality, 3 SR Balanced, 4 SR Performance, 5 RR (screen size), 6 RR from Quality, 7 DLSS 5.
$Modes = @(
    # The game as players get it: Streamline not loaded at all (DLAA asked for, so the log says it is off).
    @{ Name = "no-streamline"; Cfg = "core:upscaler=1"; Env = @{ WEFT_NO_STREAMLINE = "1" } },
    @{ Name = "off";         Cfg = "core:upscaler=0" },
    @{ Name = "dlaa";        Cfg = "core:upscaler=1" },
    @{ Name = "sr-quality";  Cfg = "core:upscaler=2" },
    @{ Name = "rr";          Cfg = "core:upscaler=6" },
    # The jitter's sign is the usual trap: DLAA again with it flipped. If this one is sharper and
    # steadier than "dlaa", say so (the default flips).
    @{ Name = "dlaa-jitterflip"; Cfg = "core:upscaler=1"; Env = @{ WEFT_DLSS_JITTER = "1,1" } },
    # DLSS 5 needs NVIDIA's neural rendering plugin (sl.dlss_nr.dll), not in the public SDK: skipped
    # until it is in streamline\ and the game binds it.
    @{ Name = "dlss5-0.5";   Cfg = "core:upscaler=7,core:dlss5_intensity=0.5"; Needs = "sl.dlss_nr.dll" },
    @{ Name = "dlss5-1.0";   Cfg = "core:upscaler=7,core:dlss5_intensity=1.0"; Needs = "sl.dlss_nr.dll" }
)
$Spots = @(
    @{ Name = "mcworld"; Script = "kit_mcworld.txt"; Args = @("--import-mc", (Join-Path $Kit "mc-world")) },
    @{ Name = "spotc";   Script = "kit_spotc.txt";   Args = @("--seed", "1") }
)
if ($Quick) {
    $Modes = $Modes | Where-Object { $_.Name -in @("no-streamline", "off", "dlaa") }
    $Spots = $Spots | Where-Object { $_.Name -eq "spotc" }
}
if ($Only) {
    $want = $Only.Split(",") | ForEach-Object { $_.Trim() }
    $Modes = $Modes | Where-Object { $_.Name -in $want }
}

function Quote([string]$a) { if ($a -match '[\s"]') { '"' + $a.Replace('"', '\"') + '"' } else { $a } }

"chiacraft DLSS kit, $Stamp" | Set-Content $Summary
"GPU / driver: " + ((Get-CimInstance Win32_VideoController | ForEach-Object { "$($_.Name) $($_.DriverVersion)" }) -join "; ") | Add-Content $Summary
"Streamline: $Sl" | Add-Content $Summary
"" | Add-Content $Summary

foreach ($spot in $Spots) {
    foreach ($mode in $Modes) {
        $name = "$($spot.Name)_$($mode.Name)"
        if ($mode.Needs -and -not (Test-Path (Join-Path $Sl $mode.Needs))) {
            "$name : skipped, needs $($mode.Needs) (DLSS 5 is not in the public Streamline SDK)" | Add-Content $Summary
            continue
        }
        $run = Join-Path $Out $name
        New-Item -ItemType Directory -Force $run | Out-Null
        $world = "kit_${name}_$Stamp".Replace("-", "_").Replace(".", "")
        $argv = @("--run-dir", (Join-Path $GameDir "run"), "--world", $world,
            "--script", (Join-Path $Kit $spot.Script), "--shot-dir", $run, "--size", $Size,
            "--view", "32", "--shader-pack", "chiacraft",
            "--config", "core:hdr_world=1,core:froxels=1,$($mode.Cfg)") + $spot.Args
        $saved = @{}
        if ($mode.Env) { foreach ($k in $mode.Env.Keys) { $saved[$k] = [Environment]::GetEnvironmentVariable($k); [Environment]::SetEnvironmentVariable($k, $mode.Env[$k]) } }
        Write-Host "== $name" -ForegroundColor Cyan
        $p = Start-Process -FilePath $Game -ArgumentList (($argv | ForEach-Object { Quote $_ }) -join " ") -WorkingDirectory $GameDir `
            -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $run "stdout.log") -RedirectStandardError (Join-Path $run "stderr.log")
        $done = $p.WaitForExit(300000)
        if (-not $done) { Stop-Process -Id $p.Id -Force; "$name : STOPPED after 5 minutes" | Add-Content $Summary }
        foreach ($k in $saved.Keys) { [Environment]::SetEnvironmentVariable($k, $saved[$k]) }
        $log = Join-Path $GameDir "run\logs\latest.log"
        if (Test-Path $log) { Copy-Item $log (Join-Path $run "game.log") }
        $text = if (Test-Path (Join-Path $run "game.log")) { Get-Content (Join-Path $run "game.log") } else { Get-Content (Join-Path $run "stdout.log") }
        "$name : exit $($p.ExitCode)" | Add-Content $Summary
        $text | Select-String -Pattern "upscaler|streamline: .*(ready|failed|missing|supported)|core:upscaler|BENCH|ERROR|panicked|GPU:" |
            ForEach-Object { "    " + ($_.Line -replace "\x1b\[[0-9;]*m", "").Trim() } | Add-Content $Summary
        "" | Add-Content $Summary
    }
}

Write-Host ""
Write-Host "Done. Results: $Out" -ForegroundColor Green
Write-Host "summary.txt has each run's DLSS lines and frame times (BENCH)."
