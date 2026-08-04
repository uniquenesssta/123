param(
  [ValidateSet("Full", "Automated", "RuntimeOnly")]
  [string]$Mode = "Full",
  [string]$ProjectRoot = "",
  [string]$TestDatabaseUrl = $env:FOOTBALL_TEST_DATABASE_URL,
  [switch]$KeepAppRunning
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($env:OS -ne "Windows_NT") {
  throw "Windows 全链路验收只能在 Windows 10/11 开发环境执行。"
}
if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
  $ProjectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
} else {
  $ProjectRoot = (Resolve-Path $ProjectRoot).Path
}
$SourceRoot = Join-Path $ProjectRoot "项目源码"
$ContractPath = Join-Path $SourceRoot "contracts\windows-acceptance-contract.json"
if (-not (Test-Path (Join-Path $SourceRoot "package.json"))) {
  throw "无法识别项目源码目录：$SourceRoot"
}
if (-not (Test-Path $ContractPath)) { throw "缺少 Windows 验收契约：$ContractPath" }
$AcceptanceContract = Get-Content -Raw -Encoding UTF8 $ContractPath | ConvertFrom-Json
$LogRoot = Join-Path $ProjectRoot "logs"
New-Item -ItemType Directory -Force -Path $LogRoot | Out-Null
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$AcceptanceLog = Join-Path $LogRoot "windows-acceptance-$Timestamp.txt"
$AcceptanceReport = Join-Path $LogRoot "windows-acceptance-$Timestamp.json"
$script:Failed = $false
$script:AppProcess = $null

function Write-AcceptanceLog([string]$Message) {
  $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $Message
  Write-Host $line
  Add-Content -Path $AcceptanceLog -Value $line -Encoding UTF8
}
function Parse-Version([string]$Text) {
  $match = [regex]::Match($Text, "(?<version>\d+\.\d+\.\d+)")
  if (-not $match.Success) { throw "无法解析版本：$Text" }
  return [version]$match.Groups["version"].Value
}
function Invoke-Stage([string]$Name, [string]$Command, [string[]]$Arguments) {
  Write-AcceptanceLog "开始：$Name"
  & $Command @Arguments
  $exitCode = $LASTEXITCODE
  if ($exitCode -ne 0) {
    Write-AcceptanceLog "失败：$Name（退出码 $exitCode）"
    throw "$Name 未通过"
  }
  Write-AcceptanceLog "通过：$Name"
}
function Assert-TestDatabase([string]$Url) {
  if ([string]::IsNullOrWhiteSpace($Url)) {
    throw "Full 模式必须设置 FOOTBALL_TEST_DATABASE_URL，且必须指向专用测试数据库。"
  }
  try { $uri = [Uri]$Url } catch { throw "FOOTBALL_TEST_DATABASE_URL 不是有效 PostgreSQL URL。" }
  if ($uri.Scheme -notin @("postgres", "postgresql")) { throw "测试数据库 URL 必须使用 postgres:// 或 postgresql://。" }
  $databaseName = $uri.AbsolutePath.Trim("/")
  $forbiddenNames = @($AcceptanceContract.database_safety.forbidden_database_names)
  if ($databaseName -in $forbiddenNames) { throw "禁止使用系统数据库执行验收：$databaseName" }
  $requiredPattern = [string]$AcceptanceContract.database_safety.required_name_pattern
  if (-not [regex]::IsMatch($databaseName, $requiredPattern)) {
    throw "测试数据库名称不符合验收契约，当前名称：$databaseName"
  }
  Write-AcceptanceLog "测试数据库安全检查通过：$databaseName（凭据未写入验收日志）"
}
function Find-ReleaseExecutable {
  $candidates = @(
    (Join-Path $SourceRoot ".cargo-target\release\football-match-model-desktop.exe"),
    (Join-Path $SourceRoot ".cargo-target\release\足球赛事模型平台.exe")
  )
  foreach ($candidate in $candidates) { if (Test-Path $candidate) { return $candidate } }
  $found = Get-ChildItem (Join-Path $SourceRoot ".cargo-target\release") -Filter "*.exe" -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notmatch "^(build-script-|deps)" } | Sort-Object LastWriteTime -Descending | Select-Object -First 1
  if ($found) { return $found.FullName }
  throw "未找到 Tauri release 可执行文件。"
}
function Wait-NewRuntimeLog([datetime]$StartedAt) {
  $deadline = (Get-Date).AddSeconds(45)
  while ((Get-Date) -lt $deadline) {
    if ($script:AppProcess.HasExited) { throw "客户端启动后提前退出，退出码：$($script:AppProcess.ExitCode)" }
    $candidate = Get-ChildItem $LogRoot -Filter "football-runtime-*.jsonl" -File -ErrorAction SilentlyContinue |
      Where-Object { $_.CreationTime -ge $StartedAt.AddSeconds(-2) } | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($candidate -and $candidate.Length -gt 0) { return $candidate.FullName }
    Start-Sleep -Milliseconds 500
  }
  throw "客户端启动后 45 秒内没有生成新的运行日志。"
}

try {
  Set-Location $SourceRoot
  Write-AcceptanceLog "Windows 全链路验收开始：模式=$Mode"
  $nodeText = (& node --version)
  $npmText = (& npm --version)
  $rustText = (& rustc --version)
  $cargoText = (& cargo --version)
  $minimumNode = [version]([string]$AcceptanceContract.minimum_versions.node)
  $minimumRust = [version]([string]$AcceptanceContract.minimum_versions.rust)
  if ((Parse-Version $nodeText) -lt $minimumNode) { throw "Node.js 版本过低：$nodeText，最低要求 $minimumNode" }
  if ((Parse-Version $rustText) -lt $minimumRust) { throw "Rust 版本过低：$rustText，最低要求 $minimumRust" }
  Write-AcceptanceLog "环境通过：Node $nodeText；npm $npmText；$rustText；$cargoText"
  if ($Mode -eq "Full") { Assert-TestDatabase $TestDatabaseUrl }

  if ($Mode -ne "RuntimeOnly") {
    Invoke-Stage "同步锁定前端依赖" "npm.cmd" @("run", "setup")
    Invoke-Stage "前端契约、类型、截图与生产构建" "npm.cmd" @("run", "verify:frontend")
    Invoke-Stage "Rust 格式、Clippy 与工作区测试" "npm.cmd" @("run", "verify:rust")
    if ($Mode -eq "Full") {
      $env:FOOTBALL_TEST_DATABASE_URL = $TestDatabaseUrl
      Invoke-Stage "PostgreSQL 空库迁移与忽略型集成测试" "cargo.exe" @("test", "--locked", "-p", "football-persistence-postgres", "--test", "postgres_integration", "--", "--ignored", "--nocapture")
    }
    Invoke-Stage "Tauri Windows release 构建" "npm.cmd" @("run", "tauri:build")
  }

  $exe = Find-ReleaseExecutable
  $env:FOOTBALL_RUNTIME_ROOT = $ProjectRoot
  $env:FOOTBALL_PROJECT_ROOT = $ProjectRoot
  $runtimeStartedAt = Get-Date
  Write-AcceptanceLog "启动 release 客户端：$exe"
  $script:AppProcess = Start-Process -FilePath $exe -WorkingDirectory $SourceRoot -PassThru
  $runtimeLog = Wait-NewRuntimeLog $runtimeStartedAt
  Write-AcceptanceLog "运行日志已建立：.\logs\$([IO.Path]::GetFileName($runtimeLog))"

  if ($Mode -eq "Full") {
    Write-Host ""
    Write-Host "请在客户端中按顺序完成以下验收链：" -ForegroundColor Cyan
    Write-Host "1. 连接专用测试数据库。"
    Write-Host "2. 导入 examples\全链路模拟\01_球队球员模拟导入.xlsx 并提交。"
    Write-Host "3. 导入 examples\全链路模拟\02_比赛阵容模拟导入.xlsx 并提交。"
    Write-Host "4. 在赛事推演完成完整度检查，并成功执行一次正式 P4 推演。"
    Write-Host "5. 打开推演历史并读取刚才的运行。"
    Write-Host "6. 录入赛果并生成赛后复盘；可继续执行正式赛后结算。"
    Write-Host "7. 在分析中心加入完整分析任务并刷新分析结果。"
    Write-Host "8. 在发布验收页执行一次不可变发布验收。"
    Write-Host ""
    [void](Read-Host "完成全部操作后按 Enter，脚本将检查本次运行日志")
  } else {
    Start-Sleep -Seconds 12
  }

  if (-not $KeepAppRunning -and $script:AppProcess -and -not $script:AppProcess.HasExited) {
    Stop-Process -Id $script:AppProcess.Id -Force
    $script:AppProcess.WaitForExit()
  }
  $profile = if ($Mode -eq "Full") { "full" } else { "startup" }
  Invoke-Stage "运行日志覆盖率与错误扫描" "node.exe" @("scripts/analyze-windows-acceptance-log.mjs", "--log", $runtimeLog, "--profile", $profile, "--report", $AcceptanceReport)
  $finalReport = Get-Content -Raw -Encoding UTF8 $AcceptanceReport | ConvertFrom-Json
  if ($finalReport.status -eq "warning") {
    Write-AcceptanceLog "Windows 全链路核心门禁通过，但存在建议项。报告：$AcceptanceReport"
  } else {
    Write-AcceptanceLog "Windows 全链路验收通过。报告：$AcceptanceReport"
  }
} catch {
  $script:Failed = $true
  Write-AcceptanceLog "Windows 全链路验收失败：$($_.Exception.Message)"
} finally {
  if (-not $KeepAppRunning -and $script:AppProcess -and -not $script:AppProcess.HasExited) {
    Stop-Process -Id $script:AppProcess.Id -Force -ErrorAction SilentlyContinue
  }
  Write-AcceptanceLog "验收过程记录：$AcceptanceLog"
}
if ($script:Failed) { exit 1 }
exit 0
