Set-StrictMode -Version Latest

function Resolve-AcceptanceProjectRoot {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)][string]$RunnerDirectory,
    [string]$ProjectRoot = ""
  )
  if (-not [string]::IsNullOrWhiteSpace($ProjectRoot)) {
    return (Resolve-Path -LiteralPath $ProjectRoot).Path
  }
  $sourceRoot = [IO.Path]::GetFullPath((Join-Path $RunnerDirectory ".."))
  if ((Split-Path -Leaf $sourceRoot) -eq "项目源码") {
    return (Resolve-Path -LiteralPath (Split-Path -Parent $sourceRoot)).Path
  }
  return (Resolve-Path -LiteralPath $sourceRoot).Path
}

function Resolve-AcceptanceSourceRoot {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot
  )

  $resolvedRoot = [IO.Path]::GetFullPath($ProjectRoot)
  $candidates = @($resolvedRoot, (Join-Path $resolvedRoot "项目源码"))
  $matches = @($candidates | Where-Object {
    (Test-Path -LiteralPath (Join-Path $_ "package.json") -PathType Leaf) -and
    (Test-Path -LiteralPath (Join-Path $_ "contracts\windows-acceptance-contract.json") -PathType Leaf)
  })
  if ($matches.Count -eq 0) {
    throw "无法识别验收源码目录：$resolvedRoot（需包含 package.json 和 Windows 验收契约）"
  }
  if ($matches.Count -ne 1) {
    throw "验收源码目录不唯一：$resolvedRoot；请明确指定待验收源码目录"
  }
  return (Resolve-Path -LiteralPath $matches[0]).Path
}


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
  if (Test-Path -LiteralPath $stampPath -PathType Leaf) {
    try {
      $stamp = Get-Content -Raw -Encoding UTF8 -LiteralPath $stampPath | ConvertFrom-Json
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
  if (-not (Test-Path -LiteralPath $releaseRoot -PathType Container)) {
    throw "未找到 Tauri release 目录：$releaseRoot"
  }

  $candidates = @(
    (Join-Path $releaseRoot "football-match-model-desktop.exe"),
    (Join-Path $releaseRoot "足球赛事模型平台.exe")
  )
  foreach ($candidate in $candidates) {
    if (Test-Path -LiteralPath $candidate -PathType Leaf) { return $candidate }
  }

  throw "未找到 Tauri release 可执行文件：$releaseRoot"
}

Export-ModuleMember -Function Resolve-AcceptanceProjectRoot, Resolve-AcceptanceSourceRoot, Resolve-AcceptanceLogRoot, Resolve-CargoTargetRoot, Find-AcceptanceReleaseExecutable
