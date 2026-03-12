import { readFile, writeFile, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

function flatten(obj, prefix = "") {
  const out = {};
  for (const [key, value] of Object.entries(obj)) {
    const next = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      Object.assign(out, flatten(value, next));
    } else {
      out[next] = value;
    }
  }
  return out;
}

function parseArgs(argv) {
  const args = {
    outputRoot: null,
    config: null,
    current: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === "--output-root") {
      args.outputRoot = argv[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (token === "--config") {
      args.config = argv[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (token === "--current") {
      args.current = argv[index + 1] ?? null;
      index += 1;
    }
  }

  return args;
}

async function readJson(filePath) {
  const raw = await readFile(filePath, "utf8");
  return JSON.parse(raw.replace(/^\uFEFF/, ""));
}

const args = parseArgs(process.argv.slice(2));
const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const configPath = args.config
  ? path.resolve(projectRoot, args.config)
  : path.resolve(projectRoot, "config/quality/business-benchmark-thresholds.json");
const currentPath = args.current
  ? path.resolve(projectRoot, args.current)
  : path.resolve(projectRoot, "docs/benchmarks/business-benchmark-latest.json");

if (!existsSync(configPath)) {
  throw new Error(`benchmark threshold config missing: ${configPath}`);
}
if (!existsSync(currentPath)) {
  throw new Error(`benchmark current summary missing: ${currentPath}`);
}

const config = await readJson(configPath);
const current = await readJson(currentPath);

const baselineFlat = flatten(config.baseline ?? {});
const thresholdFlat = flatten(config.thresholdsPercent ?? {});
const currentFlat = flatten(current.metrics ?? {});

const checks = [];
for (const [metric, baselineValue] of Object.entries(baselineFlat)) {
  const currentValue = Number(currentFlat[metric]);
  const baselineNumber = Number(baselineValue);
  const regressionPercent = Number(thresholdFlat[metric] ?? 0);

  if (!Number.isFinite(currentValue) || !Number.isFinite(baselineNumber)) {
    checks.push({ metric, passed: false, reason: "invalid numeric values" });
    continue;
  }

  const allowed = baselineNumber * (1 + regressionPercent / 100);
  const passed = currentValue <= allowed;

  checks.push({
    metric,
    baseline: baselineNumber,
    current: currentValue,
    maxRegressionPercent: regressionPercent,
    allowed,
    passed,
  });
}

const failed = checks.filter((item) => !item.passed);
const now = new Date();
const stamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}-${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}${String(now.getSeconds()).padStart(2, "0")}`;
const outputRoot = args.outputRoot
  ? path.resolve(projectRoot, args.outputRoot)
  : path.resolve(projectRoot, "data/quality-gates", stamp, "business-benchmark-thresholds");

await mkdir(outputRoot, { recursive: true });
const summaryPath = path.join(outputRoot, "business-benchmark-thresholds-summary.json");

const summary = {
  checkedAt: now.toISOString(),
  passed: failed.length === 0,
  configPath: path.relative(projectRoot, configPath).replaceAll("\\", "/"),
  currentPath: path.relative(projectRoot, currentPath).replaceAll("\\", "/"),
  checkCount: checks.length,
  failedCount: failed.length,
  checks,
};

await writeFile(summaryPath, `${JSON.stringify(summary, null, 2)}\n`, "utf8");

if (!summary.passed) {
  process.stderr.write("business benchmark threshold check failed\n");
  process.exit(1);
}
