param(
  [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "module-boundaries"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$baselinePath = Join-Path $projectRoot "config\quality\module-boundaries-baseline.json"
$summaryPath = Join-Path $resolvedOutputRoot "module-boundaries-summary.json"

function Get-TextLineCount {
  param([string]$Content)

  if ([string]::IsNullOrEmpty($Content)) {
    return 0
  }

  return ([regex]::Matches($Content, "`r`n|`n|`r").Count + 1)
}

function Get-ModuleRule {
  param([string]$RelativePath)

  if ($RelativePath -match '^apps/desktop/src/app/.+\.tsx$') {
    return [pscustomobject]@{
      id = "react-container"
      targetLines = 300
      warningLines = 400
      hardMaxLines = 500
      metrics = @(
        [pscustomobject]@{
          name = "hookUsages"
          hardMax = 25
          baselineField = "maxHookUsages"
        }
      )
    }
  }

  if ($RelativePath -match '^apps/desktop/src/.+\.css$') {
    return [pscustomobject]@{
      id = "css-module"
      targetLines = 250
      warningLines = 300
      hardMaxLines = 350
      metrics = @()
    }
  }

  if ($RelativePath -match '^apps/desktop/src/.+\.(ts|tsx)$') {
    return [pscustomobject]@{
      id = "react-module"
      targetLines = 200
      warningLines = 250
      hardMaxLines = 300
      metrics = @()
    }
  }

  if ($RelativePath -match '^src-tauri/src/.+\.rs$') {
    return [pscustomobject]@{
      id = "rust-tauri-adapter"
      targetLines = 300
      warningLines = 400
      hardMaxLines = 450
      metrics = @(
        [pscustomobject]@{
          name = "tauriCommandAnnotations"
          hardMax = 8
          baselineField = "maxTauriCommandAnnotations"
        }
      )
    }
  }

  if ($RelativePath -match '^crates/.+/tests/.+\.rs$') {
    return [pscustomobject]@{
      id = "rust-test"
      targetLines = 450
      warningLines = 600
      hardMaxLines = 700
      metrics = @()
    }
  }

  if ($RelativePath -match '^crates/.+/src/repositories\.rs$') {
    return [pscustomobject]@{
      id = "rust-repository-adapter"
      targetLines = 350
      warningLines = 450
      hardMaxLines = 550
      metrics = @(
        [pscustomobject]@{
          name = "implForCount"
          hardMax = 3
          baselineField = "maxImplForCount"
        }
      )
    }
  }

  if ($RelativePath -match '^crates/.+/src/.+\.rs$') {
    return [pscustomobject]@{
      id = "rust-domain"
      targetLines = 350
      warningLines = 450
      hardMaxLines = 550
      metrics = @()
    }
  }

  return $null
}

function Get-MetricValue {
  param(
    [string]$MetricName,
    [string]$Content
  )

  switch ($MetricName) {
    "hookUsages" {
      return [regex]::Matches($Content, '\buse(?:State|Effect|Memo|Callback|Ref)\s*\(').Count
    }
    "tauriCommandAnnotations" {
      return [regex]::Matches($Content, '#\[\s*tauri::command\s*\]').Count
    }
    "implForCount" {
      return [regex]::Matches($Content, '(?m)^\s*impl\s+[^\n{]+\s+for\s+[^\n{]+\s*\{').Count
    }
    default {
      throw "unsupported metric name: $MetricName"
    }
  }
}

$baselineConfig = $null
$baselineLookup = @{}
if (Test-Path $baselinePath) {
  $baselineConfig = Get-Content $baselinePath -Raw | ConvertFrom-Json
  foreach ($entry in @($baselineConfig.frozenFiles)) {
    $baselineLookup[$entry.path] = $entry
  }
}

$scanRoots = @(
  (Join-Path $projectRoot "apps\desktop\src"),
  (Join-Path $projectRoot "src-tauri\src"),
  (Join-Path $projectRoot "crates")
)

$allowedExtensions = @(".ts", ".tsx", ".css", ".rs")

$files = @()
foreach ($scanRoot in $scanRoots) {
  if (-not (Test-Path $scanRoot)) {
    continue
  }

  $files += Get-ChildItem -Path $scanRoot -Recurse -File | Where-Object {
    $_.Extension -in $allowedExtensions
  }
}

$files = @($files | Sort-Object FullName -Unique)

$violations = @()
$warnings = @()
$frozenDebtFiles = @()
$improvements = @()
$checkedFiles = @()
$seenPaths = @{}

foreach ($file in $files) {
  $relativePath = Get-RepoRelativePath $file.FullName
  $rule = Get-ModuleRule -RelativePath $relativePath
  if ($null -eq $rule) {
    continue
  }

  $seenPaths[$relativePath] = $true

  $content = [System.IO.File]::ReadAllText($file.FullName)
  $lineCount = Get-TextLineCount -Content $content
  $lineOverHardMax = $lineCount -gt [int]$rule.hardMaxLines

  $baselineEntry = if ($baselineLookup.ContainsKey($relativePath)) { $baselineLookup[$relativePath] } else { $null }
  $baselineMaxLines = $null
  if ($null -ne $baselineEntry -and $null -ne $baselineEntry.maxLines) {
    $baselineMaxLines = [int]$baselineEntry.maxLines
  }

  $fileViolations = @()
  $metricResults = @()
  $isFrozenDebt = $false

  if ($lineOverHardMax) {
    if ($null -ne $baselineMaxLines) {
      if ($lineCount -le $baselineMaxLines) {
        $isFrozenDebt = $true
      } else {
        $fileViolations += [pscustomobject]@{
          kind = "line-growth"
          hardMax = [int]$rule.hardMaxLines
          baselineMax = $baselineMaxLines
          current = $lineCount
        }
      }
    } else {
      $fileViolations += [pscustomobject]@{
        kind = "line-over-limit"
        hardMax = [int]$rule.hardMaxLines
        baselineMax = $null
        current = $lineCount
      }
    }
  }

  foreach ($metricRule in @($rule.metrics)) {
    $metricValue = Get-MetricValue -MetricName $metricRule.name -Content $content
    $metricBaselineMax = $null

    if ($null -ne $baselineEntry -and ($baselineEntry.PSObject.Properties.Name -contains $metricRule.baselineField)) {
      $rawBaselineMetric = $baselineEntry.($metricRule.baselineField)
      if ($null -ne $rawBaselineMetric) {
        $metricBaselineMax = [int]$rawBaselineMetric
      }
    }

    $metricOverHardMax = $metricValue -gt [int]$metricRule.hardMax
    if ($metricOverHardMax) {
      if ($null -ne $metricBaselineMax) {
        if ($metricValue -le $metricBaselineMax) {
          $isFrozenDebt = $true
        } else {
          $fileViolations += [pscustomobject]@{
            kind = "metric-growth"
            metric = $metricRule.name
            hardMax = [int]$metricRule.hardMax
            baselineMax = $metricBaselineMax
            current = $metricValue
          }
        }
      } else {
        $fileViolations += [pscustomobject]@{
          kind = "metric-over-limit"
          metric = $metricRule.name
          hardMax = [int]$metricRule.hardMax
          baselineMax = $null
          current = $metricValue
        }
      }
    }

    $metricResults += [pscustomobject]@{
      name = $metricRule.name
      value = $metricValue
      hardMax = [int]$metricRule.hardMax
      baselineMax = $metricBaselineMax
      overHardMax = $metricOverHardMax
    }
  }

  if (($lineCount -gt [int]$rule.warningLines) -and (-not $lineOverHardMax)) {
    $warnings += [pscustomobject]@{
      path = $relativePath
      category = $rule.id
      kind = "line-warning"
      current = $lineCount
      warning = [int]$rule.warningLines
      hardMax = [int]$rule.hardMaxLines
    }
  }

  if ($null -ne $baselineMaxLines -and $lineCount -lt $baselineMaxLines) {
    $improvements += [pscustomobject]@{
      path = $relativePath
      category = $rule.id
      kind = "line-reduced"
      baselineMax = $baselineMaxLines
      current = $lineCount
    }
  }

  foreach ($metricResult in $metricResults) {
    if ($null -ne $metricResult.baselineMax -and $metricResult.value -lt $metricResult.baselineMax) {
      $improvements += [pscustomobject]@{
        path = $relativePath
        category = $rule.id
        kind = "metric-reduced"
        metric = $metricResult.name
        baselineMax = $metricResult.baselineMax
        current = $metricResult.value
      }
    }
  }

  if ($fileViolations.Count -gt 0) {
    $violations += [pscustomobject]@{
      path = $relativePath
      category = $rule.id
      lineCount = $lineCount
      hardMaxLines = [int]$rule.hardMaxLines
      violations = @($fileViolations)
    }
  }

  if ($isFrozenDebt) {
    $frozenDebtFiles += [pscustomobject]@{
      path = $relativePath
      category = $rule.id
      lineCount = $lineCount
      hardMaxLines = [int]$rule.hardMaxLines
      baselineMaxLines = $baselineMaxLines
      metrics = @($metricResults | Where-Object { $_.overHardMax })
    }
  }

  $checkedFiles += [pscustomobject]@{
    path = $relativePath
    category = $rule.id
    lineCount = $lineCount
    targetLines = [int]$rule.targetLines
    warningLines = [int]$rule.warningLines
    hardMaxLines = [int]$rule.hardMaxLines
    baselineMaxLines = $baselineMaxLines
    lineOverHardMax = $lineOverHardMax
    metrics = @($metricResults)
    hasViolations = ($fileViolations.Count -gt 0)
    isFrozenDebt = $isFrozenDebt
  }
}

$missingFrozenFiles = @()
foreach ($entry in @($baselineLookup.GetEnumerator() | Sort-Object Key)) {
  if (-not $seenPaths.ContainsKey($entry.Key)) {
    $missingFrozenFiles += [pscustomobject]@{
      path = $entry.Key
      category = $entry.Value.category
      note = "frozen file no longer exists in scan scope"
    }
  }
}

$ruleSummaries = @(
  [pscustomobject]@{
    id = "react-container"
    targetLines = 300
    warningLines = 400
    hardMaxLines = 500
    metrics = @(
      [pscustomobject]@{ name = "hookUsages"; hardMax = 25 }
    )
  },
  [pscustomobject]@{
    id = "react-module"
    targetLines = 200
    warningLines = 250
    hardMaxLines = 300
    metrics = @()
  },
  [pscustomobject]@{
    id = "css-module"
    targetLines = 250
    warningLines = 300
    hardMaxLines = 350
    metrics = @()
  },
  [pscustomobject]@{
    id = "rust-tauri-adapter"
    targetLines = 300
    warningLines = 400
    hardMaxLines = 450
    metrics = @(
      [pscustomobject]@{ name = "tauriCommandAnnotations"; hardMax = 8 }
    )
  },
  [pscustomobject]@{
    id = "rust-repository-adapter"
    targetLines = 350
    warningLines = 450
    hardMaxLines = 550
    metrics = @(
      [pscustomobject]@{ name = "implForCount"; hardMax = 3 }
    )
  },
  [pscustomobject]@{
    id = "rust-domain"
    targetLines = 350
    warningLines = 450
    hardMaxLines = 550
    metrics = @()
  },
  [pscustomobject]@{
    id = "rust-test"
    targetLines = 450
    warningLines = 600
    hardMaxLines = 700
    metrics = @()
  }
)

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  passed = ($violations.Count -eq 0)
  baselinePath = $(if (Test-Path $baselinePath) { Get-RepoRelativePath $baselinePath } else { $null })
  scope = [pscustomobject]@{
    roots = @("apps/desktop/src", "src-tauri/src", "crates")
    include = @("*.ts", "*.tsx", "*.css", "*.rs")
  }
  rules = @($ruleSummaries)
  totals = [pscustomobject]@{
    checkedFileCount = $checkedFiles.Count
    warningCount = $warnings.Count
    violationCount = $violations.Count
    frozenDebtFileCount = $frozenDebtFiles.Count
    missingFrozenFileCount = $missingFrozenFiles.Count
  }
  violations = @($violations)
  warnings = @($warnings)
  frozenDebtFiles = @($frozenDebtFiles)
  missingFrozenFiles = @($missingFrozenFiles)
  improvements = @($improvements)
  topLargeFiles = @($checkedFiles | Sort-Object lineCount -Descending | Select-Object -First 20)
}

$summary | Write-JsonFile -Path $summaryPath

if (-not $summary.passed) {
  throw "module boundaries baseline exceeded, see $(Get-RepoRelativePath $summaryPath)"
}
