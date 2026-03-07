param(
  [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "duplicate-deps"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$cargoRunner = Join-Path $projectRoot "scripts\run-cargo-with-msvc.cmd"
$baselinePath = Join-Path $projectRoot "config\quality\duplicate-deps-baseline.json"
$logPath = Join-Path $resolvedOutputRoot "duplicate-deps.log"
$summaryPath = Join-Path $resolvedOutputRoot "duplicate-deps-summary.json"

function Get-DuplicatePackageGroups {
  param([string]$Output)

  $packageGroups = @{}
  $matches = [regex]::Matches($Output, '(?m)^(?<name>[A-Za-z0-9_.+-]+) v(?<version>[0-9][0-9A-Za-z.+-]*)\b')
  foreach ($match in $matches) {
    $name = $match.Groups['name'].Value
    $version = $match.Groups['version'].Value

    if (-not $packageGroups.ContainsKey($name)) {
      $packageGroups[$name] = @{
        occurrenceCount = 0
        versions = @{}
      }
    }

    $packageGroups[$name].occurrenceCount += 1
    $packageGroups[$name].versions[$version] = $true
  }

  return @($packageGroups.GetEnumerator() | ForEach-Object {
    [pscustomobject]@{
      name = $_.Key
      occurrenceCount = $_.Value.occurrenceCount
      versions = @($_.Value.versions.Keys | Sort-Object)
    }
  } | Sort-Object name)
}

function Get-FamilyVersionLookup {
  param($Families)

  $lookup = @{}
  foreach ($family in @($Families)) {
    $lookup[$family.name] = @($family.versions | Sort-Object -Unique)
  }

  return $lookup
}

function Compare-DuplicateFamilies {
  param(
    [hashtable]$CurrentLookup,
    [hashtable]$BaselineLookup
  )

  $addedFamilies = @()
  $removedFamilies = @()
  $addedVersions = @()
  $removedVersions = @()

  foreach ($name in @($CurrentLookup.Keys | Sort-Object)) {
    if (-not $BaselineLookup.ContainsKey($name)) {
      $addedFamilies += [pscustomobject]@{
        name = $name
        versions = @($CurrentLookup[$name])
      }
      continue
    }

    $newVersions = @($CurrentLookup[$name] | Where-Object { $_ -notin $BaselineLookup[$name] } | Sort-Object)
    if ($newVersions.Count -gt 0) {
      $addedVersions += [pscustomobject]@{
        name = $name
        versions = $newVersions
      }
    }
  }

  foreach ($name in @($BaselineLookup.Keys | Sort-Object)) {
    if (-not $CurrentLookup.ContainsKey($name)) {
      $removedFamilies += [pscustomobject]@{
        name = $name
        versions = @($BaselineLookup[$name])
      }
      continue
    }

    $oldVersions = @($BaselineLookup[$name] | Where-Object { $_ -notin $CurrentLookup[$name] } | Sort-Object)
    if ($oldVersions.Count -gt 0) {
      $removedVersions += [pscustomobject]@{
        name = $name
        versions = $oldVersions
      }
    }
  }

  return [pscustomobject]@{
    addedFamilies = @($addedFamilies)
    removedFamilies = @($removedFamilies)
    addedVersions = @($addedVersions)
    removedVersions = @($removedVersions)
  }
}

$commandResult = Invoke-LoggedCommand `
  -Executable $cargoRunner `
  -Arguments @("tree", "-d", "--workspace", "--charset", "ascii") `
  -WorkingDirectory $projectRoot `
  -LogPath $logPath

$duplicateHeaders = @()
$duplicateFamilies = @()
$repeatedSingleVersionPackages = @()
$baselineFamilies = @()
$comparison = [pscustomobject]@{
  addedFamilies = @()
  removedFamilies = @()
  addedVersions = @()
  removedVersions = @()
}

if ($commandResult.ExitCode -eq 0 -and -not [string]::IsNullOrWhiteSpace($commandResult.Output)) {
  $headerMatches = [regex]::Matches($commandResult.Output, '(?m)^[A-Za-z0-9_.+-]+ v[0-9][^\r\n]*$')
  foreach ($match in $headerMatches) {
    $duplicateHeaders += $match.Value
  }
  $duplicateHeaders = @($duplicateHeaders | Sort-Object -Unique)

  $packageGroups = Get-DuplicatePackageGroups -Output $commandResult.Output
  $duplicateFamilies = @($packageGroups | Where-Object { $_.versions.Count -gt 1 } | ForEach-Object {
    [pscustomobject]@{
      name = $_.name
      versions = @($_.versions)
    }
  })
  $repeatedSingleVersionPackages = @($packageGroups | Where-Object { $_.versions.Count -eq 1 -and $_.occurrenceCount -gt 1 } | ForEach-Object {
    [pscustomobject]@{
      name = $_.name
      version = $_.versions[0]
      occurrenceCount = $_.occurrenceCount
    }
  })
}

if (Test-Path $baselinePath) {
  $baselineConfig = Get-Content $baselinePath -Raw | ConvertFrom-Json
  $baselineFamilies = @($baselineConfig.families | ForEach-Object {
    [pscustomobject]@{
      name = $_.name
      versions = @($_.versions | Sort-Object -Unique)
    }
  } | Sort-Object name)

  $comparison = Compare-DuplicateFamilies `
    -CurrentLookup (Get-FamilyVersionLookup -Families $duplicateFamilies) `
    -BaselineLookup (Get-FamilyVersionLookup -Families $baselineFamilies)
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  command = $commandResult.Command
  passed = ($commandResult.ExitCode -eq 0 -and $comparison.addedFamilies.Count -eq 0 -and $comparison.addedVersions.Count -eq 0)
  exitCode = $commandResult.ExitCode
  durationMs = $commandResult.DurationMs
  baselinePath = $(if (Test-Path $baselinePath) { Get-RepoRelativePath $baselinePath } else { $null })
  baselineMatched = ($comparison.addedFamilies.Count -eq 0 -and $comparison.addedVersions.Count -eq 0)
  rawDuplicateHeaderCount = $duplicateHeaders.Count
  rawDuplicateHeaders = @($duplicateHeaders)
  duplicateFamilyCount = $duplicateFamilies.Count
  duplicateFamilies = @($duplicateFamilies)
  repeatedSingleVersionPackageCount = $repeatedSingleVersionPackages.Count
  repeatedSingleVersionPackages = @($repeatedSingleVersionPackages)
  debtRemaining = ($duplicateFamilies.Count -gt 0)
  addedFamilies = @($comparison.addedFamilies)
  addedVersions = @($comparison.addedVersions)
  removedFamilies = @($comparison.removedFamilies)
  removedVersions = @($comparison.removedVersions)
  logPath = Get-RepoRelativePath $logPath
}

$summary | Write-JsonFile -Path $summaryPath

if (-not $summary.passed) {
  if ($commandResult.ExitCode -ne 0) {
    throw "duplicate deps check failed, see $($summary.logPath)"
  }

  throw "duplicate deps baseline exceeded, see $($summary.logPath)"
}
