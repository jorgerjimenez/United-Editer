<#
.SYNOPSIS
  Phase 0 baseline: time and peak memory of each app on your own files.

.DESCRIPTION
  For every photo in -Folder it runs, one at a time:
    LightCraft  render at 2048 px (preview size) and at full size, on the GPU and on the CPU
    PhotoCraft  open and convert to PNG (CR2, DNG, TIFF, PSD)
  and records wall-clock seconds and peak RAM (working set) of the process.
  Results go to a CSV next to the outputs, plus a summary on screen.

  Nothing in -Folder is changed; renders are written to -Out.

.EXAMPLE
  .\baseline.ps1 -Folder "\\cocha\raid5\Photography\2025\Soph 2025-06-02" -Bin "$HOME\Downloads\builds" -Limit 10
#>
param(
  [Parameter(Mandatory)] [string] $Folder,
  # Folder holding the unzipped lightcraft-windows-x64 and photocraft-windows-x64 artifacts.
  [Parameter(Mandatory)] [string] $Bin,
  [string] $Out = (Join-Path $env:TEMP "united-editer-baseline"),
  # How many files per format to measure (first N, sorted by name).
  [int] $Limit = 5
)

$ErrorActionPreference = "Stop"

function Find-Exe([string] $name) {
  $hit = Get-ChildItem -Path $Bin -Recurse -Filter $name -File -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $hit) { throw "Can't find $name under $Bin. Unzip both artifacts there first." }
  $hit.FullName
}

$lc = Find-Exe "lightcraft-cli.exe"
$pc = Find-Exe "photocraft-cli.exe"
New-Item -ItemType Directory -Force $Out | Out-Null

# Runs one command, polling its peak working set until it exits.
function Measure-Run([string] $exe, [string[]] $argv, [hashtable] $envVars) {
  $psi = New-Object System.Diagnostics.ProcessStartInfo
  $psi.FileName = $exe
  # Windows PowerShell 5.1 has no ArgumentList: quote each argument by hand.
  $psi.Arguments = ($argv | ForEach-Object { '"' + ($_ -replace '"', '\"') + '"' }) -join " "
  $psi.UseShellExecute = $false
  $psi.RedirectStandardOutput = $true
  $psi.RedirectStandardError = $true
  foreach ($k in $envVars.Keys) { $psi.Environment[$k] = $envVars[$k] }

  $sw = [System.Diagnostics.Stopwatch]::StartNew()
  $p = [System.Diagnostics.Process]::Start($psi)
  $stdout = $p.StandardOutput.ReadToEndAsync()
  $stderr = $p.StandardError.ReadToEndAsync()
  $peak = 0
  while (-not $p.HasExited) {
    try { $p.Refresh(); $peak = [Math]::Max($peak, $p.PeakWorkingSet64) } catch { }
    Start-Sleep -Milliseconds 50
  }
  $p.WaitForExit()
  $sw.Stop()
  [pscustomobject]@{
    Seconds = [Math]::Round($sw.Elapsed.TotalSeconds, 2)
    PeakMB  = [Math]::Round($peak / 1MB)
    Exit    = $p.ExitCode
    Error   = ($stderr.Result -split "`n" | Where-Object { $_ -match '\S' } | Select-Object -Last 1)
  }
}

$byExt = @{}
Get-ChildItem -Path $Folder -File | Sort-Object Name | ForEach-Object {
  $ext = $_.Extension.ToLowerInvariant()
  if ($ext -in ".cr2", ".dng", ".tif", ".tiff", ".psd", ".arw") {
    if (-not $byExt[$ext]) { $byExt[$ext] = @() }
    if ($byExt[$ext].Count -lt $Limit) { $byExt[$ext] += $_ }
  }
}
$files = $byExt.Values | ForEach-Object { $_ }
if (-not $files) { throw "No CR2, ARW, DNG, TIFF or PSD files found in $Folder" }

$rows = @()
$i = 0
foreach ($f in $files) {
  $i++
  $stem = [IO.Path]::GetFileNameWithoutExtension($f.Name)
  $ext = $f.Extension.ToLowerInvariant()
  Write-Host "[$i/$($files.Count)] $($f.Name)"

  $runs = @()
  if ($ext -ne ".psd") {
    foreach ($gpu in @("1", "0")) {
      $mode = if ($gpu -eq "1") { "gpu" } else { "cpu" }
      $runs += ,@("LightCraft", "render 2048 $mode", $lc, @("render", $f.FullName, "-o", (Join-Path $Out "$stem-lc-2048-$mode.jpg"), "--size", "2048"), @{ LIGHTCRAFT_GPU = $gpu })
      $runs += ,@("LightCraft", "render full $mode", $lc, @("render", $f.FullName, "-o", (Join-Path $Out "$stem-lc-full-$mode.jpg")), @{ LIGHTCRAFT_GPU = $gpu })
    }
  }
  $runs += ,@("PhotoCraft", "open + convert", $pc, @("convert", $f.FullName, (Join-Path $Out "$stem-pc.png")), @{})

  foreach ($r in $runs) {
    $m = Measure-Run $r[2] $r[3] $r[4]
    $rows += [pscustomobject]@{
      App = $r[0]; Task = $r[1]; File = $f.Name; Format = $ext.TrimStart(".")
      Seconds = $m.Seconds; PeakMB = $m.PeakMB; Ok = ($m.Exit -eq 0); Error = if ($m.Exit -ne 0) { $m.Error } else { "" }
    }
  }
}

$csv = Join-Path $Out ("baseline-{0:yyyyMMdd-HHmm}.csv" -f (Get-Date))
$rows | Export-Csv -NoTypeInformation -Path $csv

Write-Host ""
Write-Host "Median per app, task and format (failures excluded):"
$rows | Where-Object Ok | Group-Object App, Task, Format | ForEach-Object {
  $s = @($_.Group.Seconds | Sort-Object); $m = @($_.Group.PeakMB | Sort-Object)
  [pscustomobject]@{
    App = $_.Group[0].App; Task = $_.Group[0].Task; Format = $_.Group[0].Format; Files = $_.Count
    MedianSeconds = $s[[int][Math]::Floor($s.Count / 2)]; MedianPeakMB = $m[[int][Math]::Floor($m.Count / 2)]; MaxPeakMB = $m[-1]
  }
} | Format-Table -AutoSize

$failed = @($rows | Where-Object { -not $_.Ok })
if ($failed) {
  Write-Host "Failures:" -ForegroundColor Yellow
  $failed | Format-Table App, Task, File, Error -AutoSize -Wrap
}
Write-Host "Full results: $csv"
