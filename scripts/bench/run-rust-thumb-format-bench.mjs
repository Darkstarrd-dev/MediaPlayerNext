import { spawnSync } from 'node:child_process'
import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const PROJECT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const INPUT_DIR = 'Z:/PureBenchFolder/zip/test'
const OUTPUT_ROOT = path.resolve(PROJECT_ROOT, 'bench-user-data/rust-thumb-format-bench')
const WIDTH_VALUES = [256, 307, 358, 410, 461, 512, 563, 614, 666, 717, 768]
const ROUNDS = 3

const CANDIDATES = [
  { format: 'webp-lossless', quality: 50, label: 'webp-lossless-q50' },
  { format: 'jpeg', quality: 60, label: 'jpeg-q60' },
  { format: 'jpeg', quality: 70, label: 'jpeg-q70' },
  { format: 'jpeg', quality: 80, label: 'jpeg-q80' },
]

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

function resolveBenchBinaryPath() {
  const exeName = process.platform === 'win32' ? 'thumb_batch_bench.exe' : 'thumb_batch_bench'
  return path.resolve(PROJECT_ROOT, 'target/release', exeName)
}

function buildBenchBinary() {
  runCommand('cargo', ['build', '--release', '-p', 'media-thumb', '--bin', 'thumb_batch_bench'], PROJECT_ROOT)
}

function runRustBatch({ binaryPath, width, format, quality, outputDir }) {
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
      format,
      '--quality',
      String(quality),
    ],
    PROJECT_ROOT,
  )
  return JSON.parse(stdout)
}

function toMarkdownTable(rows) {
  const header =
    '| format | width | quality | elapsed_ms_median | img_per_sec | avg_output_kb_per_image | runs_ms |\n|---|---:|---:|---:|---:|---:|---|'
  const body = rows
    .map((row) => {
      return `| ${row.format} | ${row.width} | ${row.quality} | ${row.elapsed_ms_median} | ${row.img_per_sec} | ${row.avg_output_kb_per_image} | [${row.runs_ms.join(', ')}] |`
    })
    .join('\n')
  return `${header}\n${body}`
}

function toWinnerTable(rows) {
  const grouped = new Map()
  for (const row of rows) {
    const key = String(row.width)
    if (!grouped.has(key)) {
      grouped.set(key, [])
    }
    grouped.get(key).push(row)
  }

  const winners = [...grouped.entries()]
    .sort((a, b) => Number(a[0]) - Number(b[0]))
    .map(([width, items]) => {
      const sorted = [...items].sort((a, b) => a.elapsed_ms_median - b.elapsed_ms_median)
      const winner = sorted[0]
      return {
        width: Number(width),
        winner: `${winner.format}@q${winner.quality}`,
        elapsed_ms_median: winner.elapsed_ms_median,
        img_per_sec: winner.img_per_sec,
        avg_output_kb_per_image: winner.avg_output_kb_per_image,
      }
    })

  const header =
    '| width | winner | elapsed_ms_median | img_per_sec | avg_output_kb_per_image |\n|---:|---|---:|---:|---:|'
  const body = winners
    .map(
      (row) =>
        `| ${row.width} | ${row.winner} | ${row.elapsed_ms_median} | ${row.img_per_sec} | ${row.avg_output_kb_per_image} |`,
    )
    .join('\n')
  return `${header}\n${body}`
}

async function main() {
  const files = await fs.readdir(INPUT_DIR)
  const imageCount = files.filter((name) => name.toLowerCase().endsWith('.webp')).length
  if (imageCount === 0) {
    throw new Error(`No .webp files found in ${INPUT_DIR}`)
  }

  await fs.mkdir(OUTPUT_ROOT, { recursive: true })
  buildBenchBinary()
  const benchBinaryPath = resolveBenchBinaryPath()

  const rows = []
  for (const candidate of CANDIDATES) {
    for (const width of WIDTH_VALUES) {
      const runTimes = []
      const outputBytes = []

      const warmupDir = path.join(OUTPUT_ROOT, `warmup-${candidate.label}-w${width}`)
      runRustBatch({
        binaryPath: benchBinaryPath,
        width,
        format: candidate.format,
        quality: candidate.quality,
        outputDir: warmupDir,
      })

      for (let round = 1; round <= ROUNDS; round += 1) {
        const outDir = path.join(OUTPUT_ROOT, `${candidate.label}-w${width}-r${round}`)
        const result = runRustBatch({
          binaryPath: benchBinaryPath,
          width,
          format: candidate.format,
          quality: candidate.quality,
          outputDir: outDir,
        })
        runTimes.push(result.elapsedMs)
        outputBytes.push(result.totalOutputBytes)
      }

      const elapsedMsMedian = median(runTimes)
      const outputBytesMedian = median(outputBytes)
      const imgPerSec = imageCount / (elapsedMsMedian / 1000)
      const avgOutputKbPerImage = outputBytesMedian / imageCount / 1024

      rows.push({
        format: candidate.format,
        quality: candidate.quality,
        width,
        elapsed_ms_median: Number(elapsedMsMedian.toFixed(2)),
        img_per_sec: Number(imgPerSec.toFixed(2)),
        avg_output_kb_per_image: Number(avgOutputKbPerImage.toFixed(2)),
        runs_ms: runTimes.map((value) => Number(value.toFixed(2))),
      })

      console.log(
        `[done] ${candidate.label} width=${width} median=${elapsedMsMedian.toFixed(2)}ms avg_kb=${avgOutputKbPerImage.toFixed(2)}`,
      )
    }
  }

  const jsonPath = path.join(OUTPUT_ROOT, 'results.json')
  const mdPath = path.join(OUTPUT_ROOT, 'results.md')
  await fs.writeFile(
    jsonPath,
    JSON.stringify(
      {
        meta: {
          input_dir: INPUT_DIR,
          image_count: imageCount,
          width_values: WIDTH_VALUES,
          rounds: ROUNDS,
          candidates: CANDIDATES,
        },
        rows,
      },
      null,
      2,
    ),
    'utf8',
  )

  const markdown = [
    '# Rust Thumbnail Format Benchmark (MediaPlayerNext)',
    '',
    `- input_dir: ${INPUT_DIR}`,
    `- image_count: ${imageCount}`,
    `- width_values: [${WIDTH_VALUES.join(', ')}]`,
    `- rounds_per_setting: ${ROUNDS} (median)`,
    `- candidates: ${CANDIDATES.map((item) => `${item.format}@q${item.quality}`).join(', ')}`,
    '',
    '## Full Results',
    '',
    toMarkdownTable(rows),
    '',
    '## Winner Per Width (Fastest Encode)',
    '',
    toWinnerTable(rows),
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
