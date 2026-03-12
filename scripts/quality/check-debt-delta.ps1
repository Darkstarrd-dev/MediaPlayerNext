param(
  [string]$OutputRoot,
  [switch]$UpdateBaseline
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "debt-delta"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$baselinePath = Join-Path $projectRoot "config\quality\debt-baseline.json"
$logPath = Join-Path $resolvedOutputRoot "debt-delta.log"
$summaryPath = Join-Path $resolvedOutputRoot "debt-delta-summary.json"
$scanRoots = @(
  (Join-Path $projectRoot "crates"),
  (Join-Path $projectRoot "src-tauri")
)

$categoryDefinitions = @(
  [pscustomobject]@{ name = "unsafe"; pattern = '(?<![A-Za-z0-9_])unsafe(?![A-Za-z0-9_])'; description = "Rust unsafe keyword" },
  [pscustomobject]@{ name = "unwrap"; pattern = '\.unwrap\s*\('; description = "Rust unwrap call" },
  [pscustomobject]@{ name = "expect"; pattern = '\.expect\s*\('; description = "Rust expect call" },
  [pscustomobject]@{ name = "todo"; pattern = '\btodo!\s*\('; description = "Rust todo macro" },
  [pscustomobject]@{ name = "unimplemented"; pattern = '\bunimplemented!\s*\('; description = "Rust unimplemented macro" },
  [pscustomobject]@{ name = "panic"; pattern = '\bpanic!\s*\('; description = "Rust panic macro" },
  [pscustomobject]@{ name = "allow"; pattern = '#\s*\[\s*allow(?:\s*\(|\s*\])'; description = "Rust allow attribute" },
  [pscustomobject]@{ name = "dbg"; pattern = '\bdbg!\s*\('; description = "Rust dbg macro" }
)

function ConvertTo-CountLookup {
  param($Categories)

  $lookup = @{}
  foreach ($category in @($Categories)) {
    $fileLookup = @{}
    foreach ($file in @($category.files)) {
      $fileLookup[$file.path] = [int]$file.count
    }

    $lookup[$category.name] = [pscustomobject]@{
      totalCount = [int]$category.totalCount
      files = $fileLookup
    }
  }

  return $lookup
}

function Compare-DebtLookup {
  param(
    [hashtable]$CurrentLookup,
    [hashtable]$BaselineLookup
  )

  $addedEntries = @()
  $reducedEntries = @()
  $removedEntries = @()

  foreach ($category in @($categoryDefinitions)) {
    $categoryName = $category.name
    $currentFiles = @{}
    $baselineFiles = @{}

    if ($CurrentLookup.ContainsKey($categoryName)) {
      $currentFiles = $CurrentLookup[$categoryName].files
    }
    if ($BaselineLookup.ContainsKey($categoryName)) {
      $baselineFiles = $BaselineLookup[$categoryName].files
    }

    foreach ($path in @($currentFiles.Keys | Sort-Object)) {
      $currentCount = [int]$currentFiles[$path]
      $baselineCount = if ($baselineFiles.ContainsKey($path)) { [int]$baselineFiles[$path] } else { 0 }

      if ($currentCount -gt $baselineCount) {
        $addedEntries += [pscustomobject]@{
          category = $categoryName
          path = $path
          baselineCount = $baselineCount
          currentCount = $currentCount
          delta = $currentCount - $baselineCount
        }
      } elseif ($currentCount -lt $baselineCount) {
        $reducedEntries += [pscustomobject]@{
          category = $categoryName
          path = $path
          baselineCount = $baselineCount
          currentCount = $currentCount
          delta = $currentCount - $baselineCount
        }
      }
    }

    foreach ($path in @($baselineFiles.Keys | Sort-Object)) {
      if (-not $currentFiles.ContainsKey($path)) {
        $removedEntries += [pscustomobject]@{
          category = $categoryName
          path = $path
          baselineCount = [int]$baselineFiles[$path]
          currentCount = 0
          delta = -1 * [int]$baselineFiles[$path]
        }
      }
    }
  }

  return [pscustomobject]@{
    addedEntries = @($addedEntries | Sort-Object category, path)
    reducedEntries = @($reducedEntries | Sort-Object category, path)
    removedEntries = @($removedEntries | Sort-Object category, path)
  }
}

$files = @()
foreach ($scanRoot in $scanRoots) {
  if (-not (Test-Path $scanRoot)) {
    continue
  }

  $files += @(Get-ChildItem -Path $scanRoot -Recurse -Filter *.rs -File | Where-Object {
    $_.FullName -notmatch '[\\/]target[\\/]'
  })
}
$files = @($files | Sort-Object FullName -Unique)

$categoryFileCounts = @{}
foreach ($category in $categoryDefinitions) {
  $categoryFileCounts[$category.name] = @{}
}

foreach ($file in $files) {
  $relativePath = Get-RepoRelativePath $file.FullName
  $lines = [System.IO.File]::ReadAllLines($file.FullName)

  for ($lineIndex = 0; $lineIndex -lt $lines.Length; $lineIndex += 1) {
    $line = $lines[$lineIndex]

    foreach ($category in $categoryDefinitions) {
      $matchCount = [regex]::Matches($line, $category.pattern).Count
      if ($matchCount -le 0) {
        continue
      }

      if (-not $categoryFileCounts[$category.name].ContainsKey($relativePath)) {
        $categoryFileCounts[$category.name][$relativePath] = 0
      }

      $categoryFileCounts[$category.name][$relativePath] += $matchCount
    }
  }
}

$currentCategories = @($categoryDefinitions | ForEach-Object {
  $categoryName = $_.name
  $fileEntries = @($categoryFileCounts[$categoryName].GetEnumerator() | Sort-Object Key | ForEach-Object {
    [pscustomobject]@{
      path = $_.Key
      count = [int]$_.Value
    }
  })
  $categoryMeasure = @($fileEntries | Measure-Object -Property count -Sum)
  $totalCount = if ($categoryMeasure.Count -gt 0 -and $null -ne $categoryMeasure[0].Sum) {
    [int]$categoryMeasure[0].Sum
  } else {
    0
  }

  [pscustomobject]@{
    name = $categoryName
    description = $_.description
    totalCount = $totalCount
    files = $fileEntries
  }
})

$baselineConfig = $null
$baselineCategories = @()
$comparison = [pscustomobject]@{
  addedEntries = @()
  reducedEntries = @()
  removedEntries = @()
}

if ((Test-Path $baselinePath) -and -not $UpdateBaseline) {
  $baselineConfig = Get-Content $baselinePath -Raw | ConvertFrom-Json
  $baselineCategories = @($baselineConfig.categories | ForEach-Object {
    [pscustomobject]@{
      name = $_.name
      totalCount = [int]$_.totalCount
      files = @($_.files | ForEach-Object {
        [pscustomobject]@{
          path = $_.path
          count = [int]$_.count
        }
      })
    }
  })

  $comparison = Compare-DebtLookup -CurrentLookup (ConvertTo-CountLookup -Categories $currentCategories) -BaselineLookup (ConvertTo-CountLookup -Categories $baselineCategories)
}

$debtRemaining = [int](@($currentCategories | Measure-Object -Property totalCount -Sum).Sum) -gt 0
$baselineExists = Test-Path $baselinePath
$baselineMatched = ($comparison.addedEntries.Count -eq 0)

if ($UpdateBaseline) {
  $baselineDocument = [pscustomobject]@{
    updatedAt = (Get-Date).ToString("s")
    policy = "No new Rust debt occurrences by file/category; only allow reductions within the baseline."
    scope = [pscustomobject]@{
      roots = @("crates", "src-tauri")
      include = @("**/*.rs")
    }
    categories = @($currentCategories)
  }

  $baselineDirectory = Split-Path -Parent $baselinePath
  if (-not (Test-Path $baselineDirectory)) {
    New-Item -ItemType Directory -Force -Path $baselineDirectory | Out-Null
  }

  $baselineDocument | Write-JsonFile -Path $baselinePath
  $baselineExists = $true
  $baselineMatched = $true
}

$passed = $false
if ($UpdateBaseline) {
  $passed = $true
} elseif (-not $baselineExists) {
  $passed = $false
} else {
  $passed = $baselineMatched
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  passed = $passed
  updateBaseline = $UpdateBaseline.IsPresent
  baselinePath = $(if ($baselineExists) { Get-RepoRelativePath $baselinePath } else { $null })
  baselineMatched = $baselineMatched
  baselineExists = $baselineExists
  scannedRustFileCount = $files.Count
  debtRemaining = $debtRemaining
  totalOccurrenceCount = [int](@($currentCategories | Measure-Object -Property totalCount -Sum).Sum)
  categories = @($currentCategories)
  addedEntries = @($comparison.addedEntries)
  reducedEntries = @($comparison.reducedEntries)
  removedEntries = @($comparison.removedEntries)
  logPath = Get-RepoRelativePath $logPath
}

$logLines = @(
  "mode: $(if ($UpdateBaseline) { 'update-baseline' } else { 'check' })",
  "scanned_rust_file_count: $($files.Count)",
  "baseline_path: $(if ($baselineExists) { Get-RepoRelativePath $baselinePath } else { '<missing>' })",
  "baseline_matched: $baselineMatched",
  "debt_remaining: $debtRemaining",
  "total_occurrence_count: $($summary.totalOccurrenceCount)",
  "",
  "category_totals:"
)

foreach ($category in $currentCategories) {
  $logLines += "- $($category.name): $($category.totalCount)"
}

if ($comparison.addedEntries.Count -gt 0) {
  $logLines += ""
  $logLines += "added_entries:"
  foreach ($entry in $comparison.addedEntries) {
    $logLines += "- $($entry.category): $($entry.path) baseline=$($entry.baselineCount) current=$($entry.currentCount) delta=$($entry.delta)"
  }
}

Set-Content -Path $logPath -Value $logLines -Encoding utf8
$summary | Write-JsonFile -Path $summaryPath

if (-not $summary.passed) {
  if (-not $baselineExists) {
    throw "debt baseline missing, create $((Get-RepoRelativePath $baselinePath)) before running the gate"
  }

  throw "debt delta baseline exceeded, see $($summary.logPath)"
}
