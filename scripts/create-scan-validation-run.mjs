import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve, basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const workspaceRoot = resolve(__dirname, '..');

const baselines = {
  small: join(workspaceRoot, 'docs', 'fixtures', 'small-fixture', 'generated-placeholder'),
  medium: join(workspaceRoot, 'docs', 'fixtures', 'medium-fixture', 'generated-placeholder', 'scan-root'),
  local: join(workspaceRoot, 'data', 'scan-validation', 'local-real-dir-mock'),
};

const runName = process.argv[2] || timestampName();
const runsRoot = join(workspaceRoot, 'data', 'scan-validation', 'runs');
const runRoot = join(runsRoot, runName);

main();

function main() {
  if (existsSync(runRoot)) {
    throw new Error(`run directory already exists: ${runRoot}`);
  }

  mkdirSync(runRoot, { recursive: true });
  const copied = {};
  for (const [key, sourceDir] of Object.entries(baselines)) {
    if (!existsSync(sourceDir)) {
      throw new Error(`baseline directory not found: ${sourceDir}`);
    }
    const targetDir = join(runRoot, key);
    cpSync(sourceDir, targetDir, { recursive: true });
    copied[key] = targetDir;
  }

  const manifest = {
    runName,
    createdAt: new Date().toISOString(),
    baselines,
    copied,
  };
  writeFileSync(join(runRoot, 'run.manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  process.stdout.write(`${JSON.stringify(manifest, null, 2)}\n`);
}

function timestampName() {
  return new Date().toISOString().replace(/[:.]/g, '-');
}
