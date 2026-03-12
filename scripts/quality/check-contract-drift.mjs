import { readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

function parseArgs(argv) {
  const args = {
    summary: null,
    baseline: null,
    updateBaseline: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const item = argv[index];
    if (item === "--summary") {
      args.summary = argv[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (item === "--baseline") {
      args.baseline = argv[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (item === "--update-baseline") {
      args.updateBaseline = true;
    }
  }

  return args;
}

function toPosix(value) {
  return value.split(path.sep).join("/");
}

function camelToSnake(value) {
  return value
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/[-\s]+/g, "_")
    .toLowerCase();
}

function uniqueSorted(values) {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

const args = parseArgs(process.argv.slice(2));
const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const baselinePath = args.baseline
  ? path.resolve(projectRoot, args.baseline)
  : path.resolve(projectRoot, "config/quality/contract-drift-baseline.json");
const summaryPath = args.summary
  ? path.resolve(projectRoot, args.summary)
  : path.resolve(projectRoot, "data/quality-gates/contract-drift-summary.json");

const commandDir = path.resolve(projectRoot, "packages/contracts/src/commands");
const commandFiles = [
  "archive.ts",
  "items.ts",
  "library.ts",
  "playback.ts",
  "scan.ts",
  "subtitle.ts",
  "thumbnail.ts",
  "workspace.ts",
];

const contractCommandNames = [];
for (const fileName of commandFiles) {
  const filePath = path.join(commandDir, fileName);
  const source = await readFile(filePath, "utf8");
  const regex = /export\s+const\s+([A-Za-z0-9_]+)RequestSchema\s*=/g;
  for (const match of source.matchAll(regex)) {
    const normalized = `${camelToSnake(match[1])}_command`;
    contractCommandNames.push(normalized);
  }
}

const libRsPath = path.resolve(projectRoot, "src-tauri/src/lib.rs");
const libSource = await readFile(libRsPath, "utf8");
const handlerMatch = libSource.match(/tauri::generate_handler!\[([\s\S]*?)\]\)/m);
const rustCommandNames = [];
if (handlerMatch) {
  const body = handlerMatch[1];
  const regex = /::([a-z0-9_]+)\s*,?/g;
  for (const match of body.matchAll(regex)) {
    rustCommandNames.push(match[1]);
  }
}

const contractCommands = uniqueSorted(contractCommandNames);
const rustCommands = uniqueSorted(rustCommandNames);

let baselineExists = existsSync(baselinePath);
let baseline = {
  ignoredContractCommandsMissingInRust: [],
  ignoredRustCommandsMissingInContracts: [],
};
if (baselineExists) {
  baseline = JSON.parse(await readFile(baselinePath, "utf8"));
}

const ignoredContract = new Set(
  baseline.ignoredContractCommandsMissingInRust ?? []
);
const ignoredRust = new Set(
  baseline.ignoredRustCommandsMissingInContracts ?? []
);

const missingInRust = contractCommands
  .filter((name) => !rustCommands.includes(name))
  .filter((name) => !ignoredContract.has(name));
const missingInContracts = rustCommands
  .filter((name) => !contractCommands.includes(name))
  .filter((name) => !ignoredRust.has(name));

const summary = {
  checkedAt: new Date().toISOString(),
  passed: baselineExists && missingInRust.length === 0 && missingInContracts.length === 0,
  baselinePath: toPosix(path.relative(projectRoot, baselinePath)),
  baselineExists,
  scannedContractCommandCount: contractCommands.length,
  scannedRustCommandCount: rustCommands.length,
  ignoredContractCommandsMissingInRust: uniqueSorted([...ignoredContract]),
  ignoredRustCommandsMissingInContracts: uniqueSorted([...ignoredRust]),
  missingInRust,
  missingInContracts,
  contractCommands,
  rustCommands,
};

if (args.updateBaseline) {
  const nextBaseline = {
    updatedAt: new Date().toISOString(),
    ignoredContractCommandsMissingInRust: uniqueSorted([...ignoredContract]),
    ignoredRustCommandsMissingInContracts: uniqueSorted([...ignoredRust]),
  };
  await writeFile(baselinePath, `${JSON.stringify(nextBaseline, null, 2)}\n`, "utf8");
  baselineExists = true;
}

await writeFile(summaryPath, `${JSON.stringify(summary, null, 2)}\n`, "utf8");

if (!summary.passed) {
  process.stderr.write("contract drift check failed\n");
  process.exit(1);
}
