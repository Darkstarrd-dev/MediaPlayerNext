import fs from 'node:fs/promises'
import path from 'node:path'
import sharp from 'sharp'

const INPUT_DIR = 'Z:/PureBenchFolder/zip/test'
const OUTPUT_ROOT = path.resolve('bench-user-data/lossy-codec-quality-bench')
const WIDTH = 512
const QUALITY_LEVELS = [40, 50, 60]
const ROUNDS = 3
const FORMATS = ['webp', 'jpeg']

sharp.cache(false)

function median(values) {
  const sorted = [...values].sort((a, b) => a - b)
  const mid = Math.floor(sorted.length / 2)
  return sorted.length % 2 === 0
    ? (sorted[mid - 1] + sorted[mid]) / 2
    : sorted[mid]
}

function toMs(startNs, endNs) {
  return Number(endNs - startNs) / 1e6
}

async function listInputFiles(inputDir) {
  const names = await fs.readdir(inputDir)
  return names
    .filter((name) => name.toLowerCase().endsWith('.webp'))
    .sort((a, b) => a.localeCompare(b))
    .map((name) => path.join(inputDir, name))
}

async function runOneBatch({ files, format, quality, outputDir }) {
  await fs.rm(outputDir, { recursive: true, force: true })
  await fs.mkdir(outputDir, { recursive: true })

  const start = process.hrtime.bigint()
  await Promise.all(
    files.map(async (filePath) => {
      const baseName = path.parse(filePath).name
      const ext = format === 'jpeg' ? '.jpg' : '.webp'
      const outPath = path.join(outputDir, `${baseName}${ext}`)

      const pipeline = sharp(filePath)
        .resize({ width: WIDTH, withoutEnlargement: true })

      if (format === 'webp') {
        await pipeline.webp({ quality }).toFile(outPath)
        return
      }

      await pipeline.jpeg({ quality }).toFile(outPath)
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
    elapsedMs: toMs(start, end),
    totalOutputBytes,
  }
}

async function benchmarkLossyQuality({ files }) {
  const rows = []

  for (const format of FORMATS) {
    for (const quality of QUALITY_LEVELS) {
      const runTimes = []
      const runBytes = []

      const warmupDir = path.join(OUTPUT_ROOT, `${format}-q${quality}-warmup`)
      await runOneBatch({ files, format, quality, outputDir: warmupDir })

      for (let round = 1; round <= ROUNDS; round += 1) {
        const outDir = path.join(OUTPUT_ROOT, `${format}-q${quality}-r${round}`)
        const result = await runOneBatch({ files, format, quality, outputDir: outDir })
        runTimes.push(result.elapsedMs)
        runBytes.push(result.totalOutputBytes)
      }

      const elapsedMsMedian = median(runTimes)
      const outputBytesMedian = median(runBytes)
      const imgPerSec = files.length / (elapsedMsMedian / 1000)
      const avgOutputKbPerImage = outputBytesMedian / files.length / 1024

      rows.push({
        format,
        width: WIDTH,
        quality,
        total_images: files.length,
        elapsed_ms_median: Number(elapsedMsMedian.toFixed(2)),
        img_per_sec: Number(imgPerSec.toFixed(2)),
        total_output_mb_median: Number((outputBytesMedian / (1024 * 1024)).toFixed(2)),
        avg_output_kb_per_image: Number(avgOutputKbPerImage.toFixed(2)),
        runs_ms: runTimes.map((value) => Number(value.toFixed(2))),
      })

      console.log(
        `[done] format=${format}, quality=${quality}, median=${elapsedMsMedian.toFixed(2)}ms, avgKB=${avgOutputKbPerImage.toFixed(2)}`,
      )
    }
  }

  return rows
}

function buildComparisonRows(rows) {
  const byKey = new Map(rows.map((row) => [`${row.format}:${row.quality}`, row]))
  return QUALITY_LEVELS.map((quality) => {
    const webp = byKey.get(`webp:${quality}`)
    const jpeg = byKey.get(`jpeg:${quality}`)
    if (!webp || !jpeg) {
      return null
    }
    return {
      quality,
      width: WIDTH,
      webp_elapsed_ms: webp.elapsed_ms_median,
      jpeg_elapsed_ms: jpeg.elapsed_ms_median,
      webp_vs_jpeg_speed_ratio: Number((webp.elapsed_ms_median / jpeg.elapsed_ms_median).toFixed(2)),
      webp_avg_kb: webp.avg_output_kb_per_image,
      jpeg_avg_kb: jpeg.avg_output_kb_per_image,
      webp_vs_jpeg_size_ratio: Number((webp.avg_output_kb_per_image / jpeg.avg_output_kb_per_image).toFixed(2)),
    }
  }).filter(Boolean)
}

function toResultTable(rows) {
  const header =
    '| format | width | quality | total_images | elapsed_ms_median | img_per_sec | total_output_mb_median | avg_output_kb_per_image | runs_ms |\n' +
    '|---|---:|---:|---:|---:|---:|---:|---:|---|'
  const body = rows
    .map(
      (r) =>
        `| ${r.format} | ${r.width} | ${r.quality} | ${r.total_images} | ${r.elapsed_ms_median} | ${r.img_per_sec} | ${r.total_output_mb_median} | ${r.avg_output_kb_per_image} | [${r.runs_ms.join(', ')}] |`,
    )
    .join('\n')
  return `${header}\n${body}`
}

function toComparisonTable(rows) {
  const header =
    '| quality | width | webp_elapsed_ms | jpeg_elapsed_ms | webp_vs_jpeg_speed_ratio | webp_avg_kb | jpeg_avg_kb | webp_vs_jpeg_size_ratio |\n' +
    '|---:|---:|---:|---:|---:|---:|---:|---:|'
  const body = rows
    .map(
      (r) =>
        `| ${r.quality} | ${r.width} | ${r.webp_elapsed_ms} | ${r.jpeg_elapsed_ms} | ${r.webp_vs_jpeg_speed_ratio} | ${r.webp_avg_kb} | ${r.jpeg_avg_kb} | ${r.webp_vs_jpeg_size_ratio} |`,
    )
    .join('\n')
  return `${header}\n${body}`
}

async function main() {
  const files = await listInputFiles(INPUT_DIR)
  if (files.length === 0) {
    throw new Error(`No .webp files found in ${INPUT_DIR}`)
  }

  await fs.mkdir(OUTPUT_ROOT, { recursive: true })
  const rows = await benchmarkLossyQuality({ files })
  const comparisonRows = buildComparisonRows(rows)

  const result = {
    meta: {
      input_dir: INPUT_DIR,
      image_count: files.length,
      width: WIDTH,
      formats: FORMATS,
      quality_levels: QUALITY_LEVELS,
      rounds: ROUNDS,
      resize: 'withoutEnlargement=true',
      note: 'same encoder quality parameter for lossy webp/jpeg',
    },
    rows,
    comparison: comparisonRows,
  }

  const jsonPath = path.join(OUTPUT_ROOT, 'results.json')
  const mdPath = path.join(OUTPUT_ROOT, 'results.md')
  await fs.writeFile(jsonPath, JSON.stringify(result, null, 2), 'utf8')

  const markdown = [
    '# Lossy Codec Quality Benchmark (MediaPlayerNext)',
    '',
    `- input_dir: ${INPUT_DIR}`,
    `- image_count: ${files.length}`,
    `- width: ${WIDTH}`,
    `- formats: [${FORMATS.join(', ')}]`,
    `- quality_levels: [${QUALITY_LEVELS.join(', ')}]`,
    `- rounds_per_setting: ${ROUNDS} (median)`,
    '- resize_policy: withoutEnlargement=true',
    '',
    '## Full Results',
    '',
    toResultTable(rows),
    '',
    '## WebP vs JPEG (Same Quality Parameter)',
    '',
    toComparisonTable(comparisonRows),
    '',
  ].join('\n')

  await fs.writeFile(mdPath, markdown, 'utf8')

  console.log('\nBenchmark completed.')
  console.log(`JSON: ${jsonPath}`)
  console.log(`Markdown: ${mdPath}`)
}

main().catch((err) => {
  console.error(err)
  process.exitCode = 1
})
