import { mkdir, readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

function flatten(obj, prefix = "") {
  const output = {};
  for (const [key, value] of Object.entries(obj ?? {})) {
    const next = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      Object.assign(output, flatten(value, next));
    } else {
      output[next] = Number(value);
    }
  }
  return output;
}

async function readJson(filePath) {
  const raw = await readFile(filePath, "utf8");
  return JSON.parse(raw.replace(/^\uFEFF/, ""));
}

const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const historyPath = path.resolve(projectRoot, "docs/benchmarks/business-benchmark-history.json");
const latestPath = path.resolve(projectRoot, "docs/benchmarks/business-benchmark-latest.json");

if (!existsSync(historyPath) || !existsSync(latestPath)) {
  throw new Error("benchmark history/latest file missing");
}

const history = await readJson(historyPath);
const latest = await readJson(latestPath);
const entries = Array.isArray(history.entries) ? history.entries : [];

const byMachine = new Map();
for (const item of entries) {
  const machine = item.machine || "unknown";
  const list = byMachine.get(machine) ?? [];
  list.push(item);
  byMachine.set(machine, list);
}

for (const [machine, list] of byMachine.entries()) {
  list.sort((a, b) => String(a.recordedAt).localeCompare(String(b.recordedAt)));
  byMachine.set(machine, list);
}

const currentMachine = latest.machine || "unknown";
const currentList = byMachine.get(currentMachine) ?? [];
const previous = currentList.length >= 2 ? currentList[currentList.length - 2] : null;

const latestByMachine = [];
for (const [machine, list] of byMachine.entries()) {
  latestByMachine.push({ machine, latest: list[list.length - 1] });
}
latestByMachine.sort((a, b) => a.machine.localeCompare(b.machine));

const currentFlat = flatten(latest.metrics);
const previousFlat = flatten(previous?.metrics ?? {});

const currentVsPrevious = Object.keys(currentFlat).sort().map((metric) => {
  const currentValue = currentFlat[metric];
  const previousValue = previousFlat[metric];
  const delta = Number.isFinite(previousValue) ? currentValue - previousValue : null;
  const deltaPercent = Number.isFinite(previousValue) && previousValue !== 0
    ? (delta / previousValue) * 100
    : null;
  return {
    metric,
    current: currentValue,
    previous: Number.isFinite(previousValue) ? previousValue : null,
    delta,
    deltaPercent,
  };
});

const crossMachineRows = Object.keys(currentFlat).sort().map((metric) => {
  const row = { metric, machines: [] };
  for (const item of latestByMachine) {
    const metricFlat = flatten(item.latest.metrics);
    row.machines.push({
      machine: item.machine,
      value: Number.isFinite(metricFlat[metric]) ? metricFlat[metric] : null,
      recordedAt: item.latest.recordedAt,
    });
  }
  return row;
});

const now = new Date();
const stamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}-${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}${String(now.getSeconds()).padStart(2, "0")}`;
const outputRoot = path.resolve(projectRoot, "data/quality-gates", stamp, "benchmark-trends");
await mkdir(outputRoot, { recursive: true });

const summary = {
  generatedAt: now.toISOString(),
  historyPath: "docs/benchmarks/business-benchmark-history.json",
  latestPath: "docs/benchmarks/business-benchmark-latest.json",
  currentMachine,
  machineCount: latestByMachine.length,
  entryCount: entries.length,
  currentVsPrevious,
  crossMachineRows,
};

const summaryPath = path.join(outputRoot, "benchmark-trends-summary.json");
await writeFile(summaryPath, `${JSON.stringify(summary, null, 2)}\n`, "utf8");
process.stdout.write(`${path.relative(projectRoot, summaryPath).replaceAll("\\", "/")}\n`);
