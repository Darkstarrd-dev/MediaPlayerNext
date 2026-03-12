import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

function parseArgs(argv) {
  const args = {
    outputRoot: null,
    summary: null,
    baseline: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const item = argv[index];
    if (item === "--output-root") {
      args.outputRoot = argv[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (item === "--summary") {
      args.summary = argv[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (item === "--baseline") {
      args.baseline = argv[index + 1] ?? null;
      index += 1;
    }
  }

  return args;
}

function toPosix(value) {
  return value.split(path.sep).join("/");
}

async function readJson(filePath) {
  const raw = await readFile(filePath, "utf8");
  return JSON.parse(raw.replace(/^\uFEFF/, ""));
}

async function findLatestFastSummary(projectRoot) {
  const root = path.join(projectRoot, "data", "quality-gates");
  const entries = await readdir(root, { withFileTypes: true });
  const candidates = entries
    .filter((entry) => entry.isDirectory())
    .map((entry) =>
      path.join(root, entry.name, "rust-gates-fast", "quality-gates-summary.json")
    )
    .filter((filePath) => existsSync(filePath))
    .sort((left, right) => right.localeCompare(left));

  if (candidates.length === 0) {
    throw new Error("no rust-gates-fast summary found");
  }

  return candidates[0];
}

const args = parseArgs(process.argv.slice(2));
const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const baselinePath = args.baseline
  ? path.resolve(projectRoot, args.baseline)
  : path.resolve(projectRoot, "config/quality/gate-duration-baseline.json");

const resolvedOutputRoot = args.outputRoot
  ? path.resolve(projectRoot, args.outputRoot)
  : path.resolve(projectRoot, "data/quality-gates", `${new Date().toISOString().replace(/[-:TZ.]/g, "").slice(0, 14)}`, "gate-duration-thresholds");

const summaryPath = path.join(resolvedOutputRoot, "gate-duration-thresholds-summary.json");

const targetSummaryPath = args.summary
  ? path.resolve(projectRoot, args.summary)
  : await findLatestFastSummary(projectRoot);

const baseline = await readJson(baselinePath);
const qualitySummary = await readJson(targetSummaryPath);

const layer = qualitySummary.layer;
const gateRules = baseline.layers?.[layer]?.gates ?? {};
const gateLookup = new Map((qualitySummary.gates ?? []).map((gate) => [gate.name, gate]));

const checks = [];
for (const [gateName, rule] of Object.entries(gateRules)) {
  const gate = gateLookup.get(gateName);
  if (!gate) {
    checks.push({
      gate: gateName,
      found: false,
      passed: false,
      reason: "missing gate in quality summary",
    });
    continue;
  }

  const baselineMs = Number(rule.baselineMs);
  const maxRegressionPercent = Number(rule.maxRegressionPercent);
  const allowedMs = Math.round(baselineMs * (1 + maxRegressionPercent / 100));
  const currentMs = Number(gate.durationMs ?? 0);
  const passed = currentMs <= allowedMs;

  checks.push({
    gate: gateName,
    found: true,
    passed,
    baselineMs,
    currentMs,
    maxRegressionPercent,
    allowedMs,
  });
}

const failedChecks = checks.filter((item) => !item.passed);

const summary = {
  checkedAt: new Date().toISOString(),
  passed: failedChecks.length === 0,
  baselinePath: toPosix(path.relative(projectRoot, baselinePath)),
  qualitySummaryPath: toPosix(path.relative(projectRoot, targetSummaryPath)),
  layer,
  totalChecks: checks.length,
  failedCount: failedChecks.length,
  checks,
};

await mkdir(resolvedOutputRoot, { recursive: true });
await writeFile(summaryPath, `${JSON.stringify(summary, null, 2)}\n`, "utf8");

if (!summary.passed) {
  process.stderr.write("gate duration threshold check failed\n");
  process.exit(1);
}
