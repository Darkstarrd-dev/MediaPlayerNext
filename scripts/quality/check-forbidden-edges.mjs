import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const args = process.argv.slice(2);

function readOption(name) {
  const index = args.indexOf(name);
  if (index === -1 || index === args.length - 1) {
    return null;
  }

  return args[index + 1];
}

const metadataPath = readOption("--metadata");
const summaryPath = readOption("--summary");

if (!metadataPath || !summaryPath) {
  console.error("usage: node scripts/quality/check-forbidden-edges.mjs --metadata <path> --summary <path>");
  process.exit(1);
}

const allowedEdges = new Map([
  ["shared-model", []],
  ["media-io", ["shared-model"]],
  ["media-playback", ["shared-model"]],
  ["media-thumb", ["media-io", "shared-model"]],
  ["app-core", ["media-io", "media-playback", "media-thumb", "shared-model"]],
  ["media-db", ["app-core", "shared-model"]],
  ["mediaplayernext", ["app-core", "media-db", "shared-model"]],
]);

function parseCargoMetadata(rawText) {
  const trimmed = rawText.trim();
  if (trimmed.length === 0) {
    throw new Error("cargo metadata output is empty");
  }

  try {
    return JSON.parse(trimmed);
  } catch {
    const jsonStart = rawText.indexOf("{");
    const jsonEnd = rawText.lastIndexOf("}");

    if (jsonStart === -1 || jsonEnd === -1 || jsonStart >= jsonEnd) {
      throw new Error("cannot locate JSON payload in cargo metadata output");
    }

    return JSON.parse(rawText.slice(jsonStart, jsonEnd + 1));
  }
}

const metadata = parseCargoMetadata(await readFile(metadataPath, "utf8"));
const workspaceIds = new Set(metadata.workspace_members);
const workspacePackages = metadata.packages.filter((item) => workspaceIds.has(item.id));
const packageById = new Map(workspacePackages.map((item) => [item.id, item]));

const unknownPackages = workspacePackages
  .map((item) => item.name)
  .filter((name) => !allowedEdges.has(name))
  .sort();

const observedEdges = [];
const violations = [];

for (const node of metadata.resolve?.nodes ?? []) {
  if (!workspaceIds.has(node.id)) {
    continue;
  }

  const sourcePackage = packageById.get(node.id);
  if (!sourcePackage) {
    continue;
  }

  const allowed = new Set(allowedEdges.get(sourcePackage.name) ?? []);
  const dependencyNames = [...new Set(
    (node.deps ?? [])
      .map((dependency) => dependency.pkg)
      .filter((dependencyId) => workspaceIds.has(dependencyId))
      .map((dependencyId) => packageById.get(dependencyId)?.name)
      .filter(Boolean)
  )].sort();

  for (const dependencyName of dependencyNames) {
    observedEdges.push({ from: sourcePackage.name, to: dependencyName });
    if (!allowed.has(dependencyName)) {
      violations.push({
        from: sourcePackage.name,
        to: dependencyName,
        reason: "workspace edge is not allowed by the current P6 policy",
      });
    }
  }
}

observedEdges.sort((left, right) => {
  const fromOrder = left.from.localeCompare(right.from);
  if (fromOrder !== 0) {
    return fromOrder;
  }

  return left.to.localeCompare(right.to);
});

violations.sort((left, right) => {
  const fromOrder = left.from.localeCompare(right.from);
  if (fromOrder !== 0) {
    return fromOrder;
  }

  return left.to.localeCompare(right.to);
});

const summary = {
  checkedAt: new Date().toISOString(),
  packageCount: workspacePackages.length,
  packageNames: workspacePackages.map((item) => item.name).sort(),
  unknownPackages,
  observedEdges,
  violations,
  passed: unknownPackages.length === 0 && violations.length === 0,
};

await mkdir(path.dirname(summaryPath), { recursive: true });
await writeFile(summaryPath, JSON.stringify(summary, null, 2));

if (!summary.passed) {
  if (unknownPackages.length > 0) {
    console.error(`unknown workspace packages: ${unknownPackages.join(", ")}`);
  }

  if (violations.length > 0) {
    for (const violation of violations) {
      console.error(`forbidden edge: ${violation.from} -> ${violation.to}`);
    }
  }

  process.exit(1);
}
