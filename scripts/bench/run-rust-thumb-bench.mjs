import { spawnSync } from 'node:child_process'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const PROJECT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const INPUT_DIR = 'Z:/PureBenchFolder/zip/test'
const SOURCE_RESULTS_MD = 'Z:/Playground/CurrentWorking/MediaPlayerX/bench-user-data/sharp-thumb-bench/results.md'
const OUTPUT_ROOT = path.resolve(PROJECT_ROOT, 'bench-user-data/rust-thumb-bench')
const BASE_WIDTH = 512
const BASE_QUALITY = 50
const WIDTH_STEP_RATIO = 0.1
const QUALITY_STEP = 5
const LEVELS = 5
const ROUNDS = 3

function median(values) {
  const sorted = [...values].sort((a, b) => a - b)
  const mid = Math.floor(sorted.length / 2)
  return sorted.length % 2 === 0
    ? (sorted[mid - 1] + sorted[mid]) / 2
    : sorted[mid]
}

function makeSeries(base, step, levels, isRatio = false) {
  const arr = []
  for (let i = -levels; i <= levels; i += 1) {
    arr.push(isRatio ? Math.round(base * (1 + i * step)) : base + i * step)
  }
  return arr
}

function runCommand(command, args, workdir) {
  const result = spawnSync(command, args, {
    cwd: workdir,
    stdio: 'pipe',
    encoding: 'utf8',
  })
  if (result.status !== 0) {
    const stderr = result.stderr?.trim() ?? ''
    const stdout = result.stdout?.trim() ?? ''
    throw new Error(`command failed: ${command} ${args.join(' ')}\nstdout=${stdout}\nstderr=${stderr}`)
  }
  return result.stdout.trim()
}

async function listInputFiles(inputDir) {
  const names = await fs.readdir(inputDir)
  return names
    .filter((name) => name.toLowerCase().endsWith('.webp'))
    .sort((a, b) => a.localeCompare(b))
}

function resolveBenchBinaryPath() {
  const exeName = process.platform === 'win32' ? 'thumb_batch_bench.exe' : 'thumb_batch_bench'
  return path.resolve(PROJECT_ROOT, 'target/release', exeName)
}

function buildBenchBinary() {
  runCommand('cargo', ['build', '--release', '-p', 'media-thumb', '--bin', 'thumb_batch_bench'], PROJECT_ROOT)
}

function runRustBatch({ binaryPath, width, quality, outputDir }) {
  const stdout = runCommand(
    binaryPath,
    [
      '--input-dir',
      INPUT_DIR,
      '--output-dir',
      outputDir,
      '--width',
      String(width),
      '--quality',
      String(quality),
    ],
    PROJECT_ROOT,
  )
  const parsed = JSON.parse(stdout)
  if (!parsed || typeof parsed.elapsedMs !== 'number') {
    throw new Error(`invalid thumb_batch_bench output: ${stdout}`)
  }
  return parsed.elapsedMs
}

async function benchmarkSeries({ binaryPath, type, seriesValues }) {
  const rows = []

  for (const value of seriesValues) {
    const width = type === 'width' ? value : BASE_WIDTH
    const quality = type === 'quality' ? value : BASE_QUALITY
    const runTimes = []

    const warmupDir = path.join(OUTPUT_ROOT, `${type}-warmup-w${width}-q${quality}`)
    runRustBatch({ binaryPath, width, quality, outputDir: warmupDir })

    for (let round = 1; round <= ROUNDS; round += 1) {
      const outDir = path.join(OUTPUT_ROOT, `${type}-w${width}-q${quality}-r${round}`)
      const elapsedMs = runRustBatch({ binaryPath, width, quality, outputDir: outDir })
      runTimes.push(elapsedMs)
    }

    const elapsedMsMedian = median(runTimes)
    const imgPerSec = IMAGE_COUNT / (elapsedMsMedian / 1000)

    rows.push({
      type,
      width,
      quality,
      total_images: IMAGE_COUNT,
      elapsed_ms_median: Number(elapsedMsMedian.toFixed(2)),
      img_per_sec: Number(imgPerSec.toFixed(2)),
      runs_ms: runTimes.map((v) => Number(v.toFixed(2))),
      quality_mode: 'ignored(lossless-webp)',
    })

    console.log(
      `[done] rust ${type}=${value} => width=${width}, quality=${quality}(ignored), median=${elapsedMsMedian.toFixed(2)}ms`,
    )
  }

  return rows
}

function parseSharpRows(markdownText) {
  const rows = []
  for (const rawLine of markdownText.split(/\r?\n/)) {
    const line = rawLine.trim()
    if (!line.startsWith('| width |') && !line.startsWith('| quality |')) {
      continue
    }
    const parts = line.split('|').map((value) => value.trim())
    if (parts.length < 8) {
      continue
    }
    const type = parts[1]
    const width = Number(parts[2])
    const quality = Number(parts[3])
    const elapsed = Number(parts[5])
    if (!Number.isFinite(width) || !Number.isFinite(quality) || !Number.isFinite(elapsed)) {
      continue
    }
    rows.push({ type, width, quality, elapsed_ms_median: elapsed })
  }
  return rows
}

function toMarkdownTable(rows, includeRatio = false) {
  const header = includeRatio
    ? '| type | width | quality | sharp_elapsed_ms | rust_elapsed_ms | rust_vs_sharp_ratio |\n|---|---:|---:|---:|---:|---:|'
    : '| type | width | quality | total_images | elapsed_ms_median | img_per_sec | runs_ms | quality_mode |\n|---|---:|---:|---:|---:|---:|---|---|'
  const body = rows
    .map((row) => {
      if (includeRatio) {
        return `| ${row.type} | ${row.width} | ${row.quality} | ${row.sharp_elapsed_ms} | ${row.rust_elapsed_ms} | ${row.rust_vs_sharp_ratio} |`
      }
      return `| ${row.type} | ${row.width} | ${row.quality} | ${row.total_images} | ${row.elapsed_ms_median} | ${row.img_per_sec} | [${row.runs_ms.join(', ')}] | ${row.quality_mode} |`
    })
    .join('\n')
  return `${header}\n${body}`
}

function compareRows({ rustRows, sharpRows }) {
  const sharpByKey = new Map(sharpRows.map((row) => [`${row.type}:${row.width}:${row.quality}`, row]))
  return rustRows
    .map((row) => {
      const baseline = sharpByKey.get(`${row.type}:${row.width}:${row.quality}`)
      if (!baseline) {
        return null
      }
      const ratio = row.elapsed_ms_median / baseline.elapsed_ms_median
      return {
        type: row.type,
        width: row.width,
        quality: row.quality,
        sharp_elapsed_ms: Number(baseline.elapsed_ms_median.toFixed(2)),
        rust_elapsed_ms: Number(row.elapsed_ms_median.toFixed(2)),
        rust_vs_sharp_ratio: Number(ratio.toFixed(2)),
      }
    })
    .filter(Boolean)
}

let IMAGE_COUNT = 0

async function main() {
  const inputFiles = await listInputFiles(INPUT_DIR)
  IMAGE_COUNT = inputFiles.length
  if (IMAGE_COUNT === 0) {
    throw new Error(`No .webp files found in ${INPUT_DIR}`)
  }

  await fs.mkdir(OUTPUT_ROOT, { recursive: true })
  buildBenchBinary()
  const benchBinaryPath = resolveBenchBinaryPath()

  const widthValues = makeSeries(BASE_WIDTH, WIDTH_STEP_RATIO, LEVELS, true)
  const qualityValues = makeSeries(BASE_QUALITY, QUALITY_STEP, LEVELS, false)

  const widthRows = await benchmarkSeries({
    binaryPath: benchBinaryPath,
    type: 'width',
    seriesValues: widthValues,
  })
  const qualityRows = await benchmarkSeries({
    binaryPath: benchBinaryPath,
    type: 'quality',
    seriesValues: qualityValues,
  })

  const sharpMarkdown = await fs.readFile(SOURCE_RESULTS_MD, 'utf8')
  const sharpRows = parseSharpRows(sharpMarkdown)
  const widthCompareRows = compareRows({
    rustRows: widthRows,
    sharpRows: sharpRows.filter((row) => row.type === 'width'),
  })
  const qualityCompareRows = compareRows({
    rustRows: qualityRows,
    sharpRows: sharpRows.filter((row) => row.type === 'quality'),
  })

  const result = {
    meta: {
      input_dir: INPUT_DIR,
      image_count: IMAGE_COUNT,
      width_values: widthValues,
      quality_values: qualityValues,
      rounds: ROUNDS,
      engine: 'rust-image-lossless',
      quality_mode: 'ignored(lossless-webp)',
      baseline_source_results: SOURCE_RESULTS_MD,
    },
    width_table: widthRows,
    quality_table: qualityRows,
    compare_width_vs_sharp: widthCompareRows,
    compare_quality_vs_sharp: qualityCompareRows,
  }

  const jsonPath = path.join(OUTPUT_ROOT, 'results.json')
  const mdPath = path.join(OUTPUT_ROOT, 'results.md')
  await fs.writeFile(jsonPath, JSON.stringify(result, null, 2), 'utf8')

  const markdown = [
    '# Rust Thumbnail Benchmark (MediaPlayerNext)',
    '',
    `- input_dir: ${INPUT_DIR}`,
    `- image_count: ${IMAGE_COUNT}`,
    `- width_series(10% step, levels ±5): [${widthValues.join(', ')}]`,
    `- quality_series(step 5, levels ±5): [${qualityValues.join(', ')}]`,
    `- rounds_per_setting: ${ROUNDS} (median)`,
    '- engine: rust image crate + Lanczos3 + lossless WebP',
    '- quality_mode: quality 参数在当前引擎中不生效（仅用于和 Sharp 同参数对齐跑批）',
    `- sharp_baseline: ${SOURCE_RESULTS_MD}`,
    '',
    '## Rust Width Results (quality input=50)',
    '',
    toMarkdownTable(widthRows),
    '',
    '## Rust Quality Results (width=512, quality input varied but ignored)',
    '',
    toMarkdownTable(qualityRows),
    '',
    '## Width Comparison vs Sharp Baseline',
    '',
    toMarkdownTable(widthCompareRows, true),
    '',
    '## Quality Comparison vs Sharp Baseline',
    '',
    toMarkdownTable(qualityCompareRows, true),
    '',
  ].join('\n')
  await fs.writeFile(mdPath, markdown, 'utf8')

  console.log(`Benchmark completed. JSON: ${jsonPath}`)
  console.log(`Benchmark completed. Markdown: ${mdPath}`)
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
