param(
  [Parameter(Mandatory = $true)][string]$ModulePath,
  [Parameter(Mandatory = $true)][string]$FixtureRoot,
  [Parameter(Mandatory = $true)][string]$ActualSourceRoot
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module -Name $ModulePath -Force
$results = [Collections.Generic.List[object]]::new()
$FixtureRoot = (Resolve-Path -LiteralPath $FixtureRoot).Path
$utf8 = [Text.UTF8Encoding]::new($false)

function New-Source([string]$Path) {
  New-Item -ItemType Directory -Path (Join-Path $Path 'contracts') -Force | Out-Null
  [IO.File]::WriteAllText((Join-Path $Path 'package.json'), '{}', $utf8)
  [IO.File]::WriteAllText((Join-Path $Path 'contracts\windows-acceptance-contract.json'), '{}', $utf8)
  return $Path
}
function Equal($Actual, $Expected) {
  if ($Actual -cne $Expected) { throw "Expected [$Expected], received [$Actual]" }
}
function Reject([scriptblock]$Block) {
  $threw = $false
  try { & $Block | Out-Null } catch { $threw = $true }
  if (-not $threw) { throw 'Expected an explicit failure' }
}
function Case([string]$Name, [scriptblock]$Block) {
  try { & $Block; $results.Add([pscustomobject]@{name=$Name;passed=$true}) }
  catch { $results.Add([pscustomobject]@{name=$Name;passed=$false;error=$_.Exception.Message}) }
}

$flat = New-Source (Join-Path $FixtureRoot 'source [1] 球队')
$wrapped = Join-Path $FixtureRoot 'wrapped'
$legacy = New-Source (Join-Path $wrapped '项目源码')
$ambiguous = New-Source (Join-Path $FixtureRoot 'ambiguous')
$null = New-Source (Join-Path $ambiguous '项目源码')
$partial = Join-Path $FixtureRoot 'partial'
New-Item -ItemType Directory -Path $partial | Out-Null
[IO.File]::WriteAllText((Join-Path $partial 'package.json'), '{}', $utf8)
$target = Join-Path $FixtureRoot 'target'
New-Item -ItemType Directory -Path (Join-Path $target 'release') -Force | Out-Null

Case 'default standalone root uses runner source directory' { Equal (Resolve-AcceptanceProjectRoot -RunnerDirectory (Join-Path $flat 'scripts')) $flat }
Case 'default legacy root preserves outer runtime directory' { Equal (Resolve-AcceptanceProjectRoot -RunnerDirectory (Join-Path $legacy 'scripts')) $wrapped }
Case 'explicit root takes precedence over runner location' { Equal (Resolve-AcceptanceProjectRoot -RunnerDirectory (Join-Path $legacy 'scripts') -ProjectRoot $flat) $flat }
Case 'standalone checkout with spaces, Unicode and brackets' { Equal (Resolve-AcceptanceSourceRoot -ProjectRoot $flat) $flat }
Case 'legacy CI wrapper' { Equal (Resolve-AcceptanceSourceRoot -ProjectRoot $wrapped) $legacy }
Case 'explicit child source in legacy wrapper' { Equal (Resolve-AcceptanceSourceRoot -ProjectRoot $legacy) $legacy }
Case 'ambiguous roots are rejected' { Reject { Resolve-AcceptanceSourceRoot -ProjectRoot $ambiguous } }
Case 'package without acceptance contract is rejected' { Reject { Resolve-AcceptanceSourceRoot -ProjectRoot $partial } }
Case 'missing project is rejected' { Reject { Resolve-AcceptanceSourceRoot -ProjectRoot (Join-Path $FixtureRoot 'absent') } }
Case 'current actual checkout is recognized' { Equal (Resolve-AcceptanceSourceRoot -ProjectRoot $ActualSourceRoot) ((Resolve-Path -LiteralPath $ActualSourceRoot).Path) }
Case 'default logs remain under selected project' { Equal (Resolve-AcceptanceLogRoot -ProjectRoot $flat) (Join-Path $flat 'logs') }
Case 'relative logs resolve under selected project' { Equal (Resolve-AcceptanceLogRoot -ProjectRoot $flat -LogDirectory 'evidence\logs') (Join-Path $flat 'evidence\logs') }
Case 'absolute logs retain explicit destination' { Equal (Resolve-AcceptanceLogRoot -ProjectRoot $flat -LogDirectory (Join-Path $FixtureRoot 'logs')) (Join-Path $FixtureRoot 'logs') }
Case 'Cargo target fallback stays outside source' { Equal (Resolve-CargoTargetRoot -SourceRoot $flat) (Join-Path $FixtureRoot '.cargo-target') }
New-Item -ItemType Directory -Path (Join-Path $flat '.cargo') | Out-Null
([pscustomobject]@{target_root=$target} | ConvertTo-Json) | Set-Content -LiteralPath (Join-Path $flat '.cargo\target-location.json')
Case 'registered absolute Cargo target is honored' { Equal (Resolve-CargoTargetRoot -SourceRoot $flat) $target }
[IO.File]::WriteAllText((Join-Path $flat '.cargo\target-location.json'), '{"target_root":"relative"}', $utf8)
Case 'relative Cargo target stamp is rejected' { Reject { Resolve-CargoTargetRoot -SourceRoot $flat } }
[IO.File]::WriteAllText((Join-Path $target 'release\unrelated-tool.exe'), 'not an application', $utf8)
Case 'unrelated executable is never accepted as client' { Reject { Find-AcceptanceReleaseExecutable -CargoTargetRoot $target } }
$englishExe = Join-Path $target 'release\football-match-model-desktop.exe'
[IO.File]::WriteAllText($englishExe, 'path fixture only; never launched', $utf8)
Case 'canonical client executable is recognized' { Equal (Find-AcceptanceReleaseExecutable -CargoTargetRoot $target) $englishExe }
$localizedTarget = Join-Path $FixtureRoot 'localized'
New-Item -ItemType Directory -Path (Join-Path $localizedTarget 'release') -Force | Out-Null
$localizedExe = Join-Path $localizedTarget 'release\足球赛事模型平台.exe'
[IO.File]::WriteAllText($localizedExe, 'path fixture only; never launched', $utf8)
Case 'localized client executable is recognized' { Equal (Find-AcceptanceReleaseExecutable -CargoTargetRoot $localizedTarget) $localizedExe }
Case 'missing release directory is rejected' { Reject { Find-AcceptanceReleaseExecutable -CargoTargetRoot (Join-Path $FixtureRoot 'no-target') } }

$results | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $FixtureRoot 'native-path-results.json')
$failed = @($results | Where-Object { -not $_.passed })
if ($failed.Count) { $failed | ConvertTo-Json -Depth 4 | Write-Output; exit 1 }
Write-Output "Windows native path behavior: $($results.Count) checks PASS; $FixtureRoot"
