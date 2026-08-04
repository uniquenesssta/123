Set-StrictMode -Version Latest

function Resolve-AcceptanceLogRoot {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot,
    [string]$LogDirectory = ""
  )

  $resolvedProjectRoot = [IO.Path]::GetFullPath($ProjectRoot)
  if ([string]::IsNullOrWhiteSpace($LogDirectory)) {
    return [IO.Path]::GetFullPath((Join-Path $resolvedProjectRoot "logs"))
  }
  if ([IO.Path]::IsPathRooted($LogDirectory)) {
    return [IO.Path]::GetFullPath($LogDirectory)
  }
  return [IO.Path]::GetFullPath((Join-Path $resolvedProjectRoot $LogDirectory))
}

function Resolve-CargoTargetRoot {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)]
    [string]$SourceRoot
  )

  $resolvedSourceRoot = [IO.Path]::GetFullPath($SourceRoot)
  $stampPath = Join-Path $resolvedSourceRoot ".cargo\target-location.json"
  if (Test-Path $stampPath -PathType Leaf) {
    try {
      $stamp = Get-Content -Raw -Encoding UTF8 $stampPath | ConvertFrom-Json
    } catch {
      throw "Cargo 目标目录登记文件无法解析：$stampPath"
    }
    $registeredTarget = [string]$stamp.target_root
    if ([string]::IsNullOrWhiteSpace($registeredTarget) -or -not [IO.Path]::IsPathRooted($registeredTarget)) {
      throw "Cargo 目标目录登记文件缺少有效绝对路径：$stampPath"
    }
    return [IO.Path]::GetFullPath($registeredTarget)
  }

  return [IO.Path]::GetFullPath((Join-Path $resolvedSourceRoot "..\.cargo-target"))
}

function Find-AcceptanceReleaseExecutable {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)]
    [string]$CargoTargetRoot
  )

  $resolvedTargetRoot = [IO.Path]::GetFullPath($CargoTargetRoot)
  $releaseRoot = Join-Path $resolvedTargetRoot "release"
  if (-not (Test-Path $releaseRoot -PathType Container)) {
    throw "未找到 Tauri release 目录：$releaseRoot"
  }

  $candidates = @(
    (Join-Path $releaseRoot "football-match-model-desktop.exe"),
    (Join-Path $releaseRoot "足球赛事模型平台.exe")
  )
  foreach ($candidate in $candidates) {
    if (Test-Path $candidate -PathType Leaf) { return $candidate }
  }

  $found = Get-ChildItem $releaseRoot -Filter "*.exe" -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notmatch "^(build-script-|deps)" } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
  if ($found) { return $found.FullName }

  throw "未找到 Tauri release 可执行文件：$releaseRoot"
}

Export-ModuleMember -Function Resolve-AcceptanceLogRoot, Resolve-CargoTargetRoot, Find-AcceptanceReleaseExecutable
