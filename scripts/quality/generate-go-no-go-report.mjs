import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

function toPosix(value) {
  return value.split(path.sep).join("/");
}

async function findLatestFile(root, suffix) {
  const entries = await readdir(root, { withFileTypes: true });
  const candidates = entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(root, entry.name, suffix))
    .filter((filePath) => existsSync(filePath))
    .sort((left, right) => right.localeCompare(left));

  return candidates[0] ?? null;
}

async function readJson(filePath) {
  const raw = await readFile(filePath, "utf8");
  return JSON.parse(raw.replace(/^\uFEFF/, ""));
}

const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const qualityRoot = path.join(projectRoot, "data", "quality-gates");
const now = new Date();
const timestamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}-${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}${String(now.getSeconds()).padStart(2, "0")}`;
const outputRoot = path.join(qualityRoot, timestamp, "go-no-go");
const reportPath = path.join(outputRoot, "go-no-go-report.md");

const heavySummaryPath = await findLatestFile(qualityRoot, path.join("rust-gates-heavy", "quality-gates-summary.json"));
const releaseSummaryPath = await findLatestFile(qualityRoot, path.join("release-verify", "release-verify-summary.json"));

if (!heavySummaryPath || !releaseSummaryPath) {
  throw new Error("missing heavy/release summary artifacts for go-no-go report");
}

const heavySummary = await readJson(heavySummaryPath);
const releaseSummary = await readJson(releaseSummaryPath);

const heavyPassed = Number(heavySummary.totals?.failedCount ?? 1) === 0;
const releasePassed = Boolean(releaseSummary.passed);
const decision = heavyPassed && releasePassed ? "Go" : "No-Go";

const lines = [
  "# Release Go/No-Go 报告",
  "",
  `- 生成时间：${now.toISOString()}`,
  `- 结论：**${decision}**`,
  "",
  "## 门禁结果",
  "",
  `- heavy：${heavyPassed ? "passed" : "failed"}`,
  `  - 产物：\`${toPosix(path.relative(projectRoot, heavySummaryPath))}\``,
  `- release verify：${releasePassed ? "passed" : "failed"}`,
  `  - 产物：\`${toPosix(path.relative(projectRoot, releaseSummaryPath))}\``,
  "",
  "## 说明",
  "",
  "- 本报告由脚本自动生成，仅聚合当前可用产物。",
  "- 若结论为 No-Go，请补充阻断项与回滚计划。",
  "",
];

await mkdir(outputRoot, { recursive: true });
await writeFile(reportPath, `${lines.join("\n")}\n`, "utf8");

process.stdout.write(`${toPosix(path.relative(projectRoot, reportPath))}\n`);
