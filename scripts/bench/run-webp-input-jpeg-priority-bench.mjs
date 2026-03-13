import { spawnSync } from 'node:child_process'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import sharp from 'sharp'

const PROJECT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const INPUT_DIR = 'Z:/PureBenchFolder/zip/test'
const SOURCE_RESULTS_MD = 'Z:/Playground/CurrentWorking/MediaPlayerX/bench-user-data/sharp-thumb-bench/results.md'
const OUTPUT_ROOT = path.resolve(PROJECT_ROOT, 'bench-user-data/webp-input-jpeg-priority-bench')
const BASE_WIDTH = 512
const BASE_QUALITY = 50
const WIDTH_STEP_RATIO = 0.1
const QUALITY_STEP = 5
const LEVELS = 5
const ROUNDS = 3

sharp.cache(false)

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

function resolveBenchBinaryPath() {
  const exeName = process.platform === 'win32' ? 'thumb_batch_bench.exe' : 'thumb_batch_bench'
  return path.resolve(PROJECT_ROOT, 'target/release', exeName)
}

function buildBenchBinary() {
  runCommand('cargo', ['build', '--release', '-p', 'media-thumb', '--bin', 'thumb_batch_bench'], PROJECT_ROOT)
}

async function listInputFiles(inputDir) {
  const names = await fs.readdir(inputDir)
  return names
    .filter((name) => name.toLowerCase().endsWith('.webp'))
    .sort((a, b) => a.localeCompare(b))
    .map((name) => path.join(inputDir, name))
}

function runRustJpegBatch({ binaryPath, width, quality, outputDir }) {
  const stdout = runCommand(
    binaryPath,
    [
      '--input-dir',
      INPUT_DIR,
      '--output-dir',
      outputDir,
      '--width',
      String(width),
      '--format',
      'jpeg',
      '--quality',
      String(quality),
    ],
    PROJECT_ROOT,
  )
  const parsed = JSON.parse(stdout)
  if (!parsed || typeof parsed.elapsedMs !== 'number' || typeof parsed.totalOutputBytes !== 'number') {
    throw new Error(`invalid rust bench output: ${stdout}`)
  }
  return {
    elapsedMs: parsed.elapsedMs,
    totalOutputBytes: parsed.totalOutputBytes,
    qualityMode: parsed.qualityMode ?? 'effective',
  }
}

async function runSharpJpegBatch({ files, width, quality, outputDir }) {
  await fs.rm(outputDir, { recursive: true, force: true })
  await fs.mkdir(outputDir, { recursive: true })

  const start = process.hrtime.bigint()
  await Promise.all(
    files.map(async (filePath) => {
      const baseName = path.parse(filePath).name
      const outPath = path.join(outputDir, `${baseName}.jpg`)
      await sharp(filePath)
        .resize({ width, withoutEnlargement: true })
        .jpeg({ quality })
        .toFile(outPath)
    }),
  )
  const end = process.hrtime.bigint()

  const outputNames = await fs.readdir(outputDir)
  const sizes = await Promise.all(
    outputNames.map(async (name) => {
      const stat = await fs.stat(path.join(outputDir, name))
      return stat.size
    }),
  )
  const totalOutputBytes = sizes.reduce((sum, value) => sum + value, 0)

  return {
    elapsedMs: Number(end - start) / 1e6,
    totalOutputBytes,
    qualityMode: 'effective',
  }
}

async function benchmarkSeries({
  files,
  type,
  seriesValues,
  runOneBatch,
  engine,
  imageCount,
}) {
  const rows = []
  for (const value of seriesValues) {
    const width = type === 'width' ? value : BASE_WIDTH
    const quality = type === 'quality' ? value : BASE_QUALITY
    const runTimes = []
    const runBytes = []
    let qualityMode = 'effective'

    const warmupDir = path.join(OUTPUT_ROOT, `${engine}-${type}-warmup-w${width}-q${quality}`)
    await runOneBatch({ files, width, quality, outputDir: warmupDir })

    for (let round = 1; round <= ROUNDS; round += 1) {
      const outDir = path.join(OUTPUT_ROOT, `${engine}-${type}-w${width}-q${quality}-r${round}`)
      const batch = await runOneBatch({ files, width, quality, outputDir: outDir })
      runTimes.push(batch.elapsedMs)
      runBytes.push(batch.totalOutputBytes)
      qualityMode = batch.qualityMode
    }

    const elapsedMsMedian = median(runTimes)
    const outputBytesMedian = median(runBytes)
    const imgPerSec = imageCount / (elapsedMsMedian / 1000)
    const avgOutputKbPerImage = outputBytesMedian / imageCount / 1024

    rows.push({
      engine,
      type,
      width,
      quality,
      total_images: imageCount,
      elapsed_ms_median: Number(elapsedMsMedian.toFixed(2)),
      img_per_sec: Number(imgPerSec.toFixed(2)),
      avg_output_kb_per_image: Number(avgOutputKbPerImage.toFixed(2)),
      runs_ms: runTimes.map((item) => Number(item.toFixed(2))),
      quality_mode: qualityMode,
    })

    console.log(
      `[done] ${engine} ${type}=${value} => width=${width}, quality=${quality}, median=${elapsedMsMedian.toFixed(2)}ms`,
    )
  }

  return rows
}

function parseSourceSharpRows(markdownText) {
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
    rows.push({
      engine: 'source-sharp-webp',
      type,
      width,
      quality,
      elapsed_ms_median: Number(elapsed.toFixed(2)),
    })
  }
  return rows
}

function buildComparisonRows({ rustRows, sharpRows, sourceRows, type }) {
  const rustByKey = new Map(
    rustRows.filter((row) => row.type === type).map((row) => [`${row.width}:${row.quality}`, row]),
  )
  const sharpByKey = new Map(
    sharpRows.filter((row) => row.type === type).map((row) => [`${row.width}:${row.quality}`, row]),
  )
  const sourceByKey = new Map(
    sourceRows.filter((row) => row.type === type).map((row) => [`${row.width}:${row.quality}`, row]),
  )

  const keys = [...rustByKey.keys()]
  const output = []
  for (const key of keys) {
    const rust = rustByKey.get(key)
    const sharp = sharpByKey.get(key)
    const source = sourceByKey.get(key)
    if (!rust || !sharp || !source) {
      continue
    }

    output.push({
      type,
      width: rust.width,
      quality: rust.quality,
      source_sharp_webp_elapsed_ms: source.elapsed_ms_median,
      next_rust_jpeg_elapsed_ms: rust.elapsed_ms_median,
      next_sharp_jpeg_elapsed_ms: sharp.elapsed_ms_median,
      rust_jpeg_vs_source_ratio: Number((rust.elapsed_ms_median / source.elapsed_ms_median).toFixed(2)),
      sharp_jpeg_vs_source_ratio: Number((sharp.elapsed_ms_median / source.elapsed_ms_median).toFixed(2)),
      rust_vs_next_sharp_jpeg_ratio: Number((rust.elapsed_ms_median / sharp.elapsed_ms_median).toFixed(2)),
      next_rust_jpeg_avg_kb: rust.avg_output_kb_per_image,
      next_sharp_jpeg_avg_kb: sharp.avg_output_kb_per_image,
    })
  }

  output.sort((a, b) => {
    if (a.type !== b.type) {
      return a.type.localeCompare(b.type)
    }
    if (a.type === 'width') {
      return a.width - b.width
    }
    return a.quality - b.quality
  })

  return output
}

function toResultTable(rows) {
  const header =
    '| engine | type | width | quality | total_images | elapsed_ms_median | img_per_sec | avg_output_kb_per_image | runs_ms | quality_mode |\n' +
    '|---|---|---:|---:|---:|---:|---:|---:|---|---|'
  const body = rows
    .map(
      (row) =>
        `| ${row.engine} | ${row.type} | ${row.width} | ${row.quality} | ${row.total_images} | ${row.elapsed_ms_median} | ${row.img_per_sec} | ${row.avg_output_kb_per_image} | [${row.runs_ms.join(', ')}] | ${row.quality_mode} |`,
    )
    .join('\n')
  return `${header}\n${body}`
}

function toComparisonTable(rows) {
  const header =
    '| type | width | quality | source_sharp_webp_elapsed_ms | next_rust_jpeg_elapsed_ms | next_sharp_jpeg_elapsed_ms | rust_jpeg_vs_source_ratio | sharp_jpeg_vs_source_ratio | rust_vs_next_sharp_jpeg_ratio | next_rust_jpeg_avg_kb | next_sharp_jpeg_avg_kb |\n' +
    '|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|'
  const body = rows
    .map(
      (row) =>
        `| ${row.type} | ${row.width} | ${row.quality} | ${row.source_sharp_webp_elapsed_ms} | ${row.next_rust_jpeg_elapsed_ms} | ${row.next_sharp_jpeg_elapsed_ms} | ${row.rust_jpeg_vs_source_ratio} | ${row.sharp_jpeg_vs_source_ratio} | ${row.rust_vs_next_sharp_jpeg_ratio} | ${row.next_rust_jpeg_avg_kb} | ${row.next_sharp_jpeg_avg_kb} |`,
    )
    .join('\n')
  return `${header}\n${body}`
}

async function main() {
  const files = await listInputFiles(INPUT_DIR)
  const imageCount = files.length
  if (imageCount === 0) {
    throw new Error(`No .webp files found in ${INPUT_DIR}`)
  }

  await fs.mkdir(OUTPUT_ROOT, { recursive: true })
  buildBenchBinary()
  const benchBinaryPath = resolveBenchBinaryPath()

  const widthValues = makeSeries(BASE_WIDTH, WIDTH_STEP_RATIO, LEVELS, true)
  const qualityValues = makeSeries(BASE_QUALITY, QUALITY_STEP, LEVELS, false)

  const rustWidthRows = await benchmarkSeries({
    files,
    type: 'width',
    seriesValues: widthValues,
    engine: 'next-rust-jpeg',
    imageCount,
    runOneBatch: ({ width, quality, outputDir }) =>
      runRustJpegBatch({ binaryPath: benchBinaryPath, width, quality, outputDir }),
  })
  const rustQualityRows = await benchmarkSeries({
    files,
    type: 'quality',
    seriesValues: qualityValues,
    engine: 'next-rust-jpeg',
    imageCount,
    runOneBatch: ({ width, quality, outputDir }) =>
      runRustJpegBatch({ binaryPath: benchBinaryPath, width, quality, outputDir }),
  })

  const sharpWidthRows = await benchmarkSeries({
    files,
    type: 'width',
    seriesValues: widthValues,
    engine: 'next-sharp-jpeg',
    imageCount,
    runOneBatch: ({ files: inputFiles, width, quality, outputDir }) =>
      runSharpJpegBatch({ files: inputFiles, width, quality, outputDir }),
  })
  const sharpQualityRows = await benchmarkSeries({
    files,
    type: 'quality',
    seriesValues: qualityValues,
    engine: 'next-sharp-jpeg',
    imageCount,
    runOneBatch: ({ files: inputFiles, width, quality, outputDir }) =>
      runSharpJpegBatch({ files: inputFiles, width, quality, outputDir }),
  })

  const sourceMarkdown = await fs.readFile(SOURCE_RESULTS_MD, 'utf8')
  const sourceRows = parseSourceSharpRows(sourceMarkdown)

  const widthComparisonRows = buildComparisonRows({
    rustRows: rustWidthRows,
    sharpRows: sharpWidthRows,
    sourceRows,
    type: 'width',
  })
  const qualityComparisonRows = buildComparisonRows({
    rustRows: rustQualityRows,
    sharpRows: sharpQualityRows,
    sourceRows,
    type: 'quality',
  })

  const result = {
    meta: {
      input_dir: INPUT_DIR,
      image_count: imageCount,
      width_values: widthValues,
      quality_values: qualityValues,
      rounds: ROUNDS,
      source_baseline_results: SOURCE_RESULTS_MD,
      scenario: 'webp-input-majority, jpeg-priority-output',
      note: 'rust jpeg path includes scaled decode(jpeg-decoder), simd resize(fast_image_resize), and jpeg encode chain',
    },
    next_rust_jpeg_width: rustWidthRows,
    next_rust_jpeg_quality: rustQualityRows,
    next_sharp_jpeg_width: sharpWidthRows,
    next_sharp_jpeg_quality: sharpQualityRows,
    compare_width_with_source: widthComparisonRows,
    compare_quality_with_source: qualityComparisonRows,
  }

  const jsonPath = path.join(OUTPUT_ROOT, 'results.json')
  const mdPath = path.join(OUTPUT_ROOT, 'results.md')
  await fs.writeFile(jsonPath, JSON.stringify(result, null, 2), 'utf8')

  const markdown = [
    '# WebP Input JPEG Priority Benchmark (MediaPlayerNext)',
    '',
    `- input_dir: ${INPUT_DIR}`,
    `- image_count: ${imageCount}`,
    `- width_series(10% step, levels ±5): [${widthValues.join(', ')}]`,
    `- quality_series(step 5, levels ±5): [${qualityValues.join(', ')}]`,
    `- rounds_per_setting: ${ROUNDS} (median)`,
    `- source_baseline: ${SOURCE_RESULTS_MD}`,
    '- scenario: webp-input-majority, output jpeg priority',
    '- note: rust jpeg path uses jpeg-decoder scale + fast_image_resize + jpeg encoder',
    '',
    '## Next Rust JPEG - Width Results',
    '',
    toResultTable(rustWidthRows),
    '',
    '## Next Rust JPEG - Quality Results',
    '',
    toResultTable(rustQualityRows),
    '',
    '## Next Sharp JPEG - Width Results',
    '',
    toResultTable(sharpWidthRows),
    '',
    '## Next Sharp JPEG - Quality Results',
    '',
    toResultTable(sharpQualityRows),
    '',
    '## Width Comparison vs Source Sharp WebP',
    '',
    toComparisonTable(widthComparisonRows),
    '',
    '## Quality Comparison vs Source Sharp WebP',
    '',
    toComparisonTable(qualityComparisonRows),
    '',
  ].join('\n')

  await fs.writeFile(mdPath, markdown, 'utf8')

  console.log('Benchmark completed.')
  console.log(`JSON: ${jsonPath}`)
  console.log(`Markdown: ${mdPath}`)
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
