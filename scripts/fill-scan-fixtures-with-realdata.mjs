import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, utimesSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const workspaceRoot = resolve(__dirname, '..');

const labels = {
  english: 'english',
  chinese: '中文资料',
  japanese: '日本語資料',
  spaces: 'space folder',
  deepA: 'level-one',
  deepB: 'level two',
  deepC: 'level_three',
};

const fixtureTargets = {
  small: join(workspaceRoot, 'docs', 'fixtures', 'small-fixture', 'generated-placeholder'),
  medium: join(workspaceRoot, 'docs', 'fixtures', 'medium-fixture', 'generated-placeholder', 'scan-root'),
  local: join(workspaceRoot, 'data', 'scan-validation', 'local-real-dir-mock'),
};

const sourceRoots = {
  small: join(workspaceRoot, 'docs', 'fixtures', 'realdatasmall'),
  medium: join(workspaceRoot, 'docs', 'fixtures', 'realdata'),
};

const datasetConfigs = [
  {
    key: 'small',
    rootDir: fixtureTargets.small,
    sourceRoot: sourceRoots.small,
    profile: 'small-fixture-real',
    counts: { image: 12, audio: 4, video: 3, archive: 4, other: 5 },
  },
  {
    key: 'medium',
    rootDir: fixtureTargets.medium,
    sourceRoot: sourceRoots.medium,
    profile: 'medium-fixture-real',
    counts: { image: 210, audio: 30, video: 24, archive: 42, other: 24 },
  },
  {
    key: 'local',
    rootDir: fixtureTargets.local,
    sourceRoot: sourceRoots.medium,
    profile: 'local-real-dir-mock-real',
    counts: { image: 350, audio: 45, video: 40, archive: 55, other: 30 },
  },
];

main();

function main() {
  const manifests = datasetConfigs.map(fillDataset);
  const summary = {
    generatedAt: new Date().toISOString(),
    datasets: manifests,
    notes: [
      'All fixture baselines are now filled from real sample pools.',
      'Use these baseline directories as read-mostly sample sources.',
      'Daily tests should copy from baselines into data/scan-validation/runs/.',
    ],
  };

  writeJson(join(workspaceRoot, 'docs', 'fixtures', 'real-fixture-fill.summary.json'), summary);
  process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
}

function fillDataset(config) {
  if (!existsSync(config.sourceRoot)) {
    throw new Error(`source sample directory not found: ${config.sourceRoot}`);
  }

  const sourceFiles = collectTypedFiles(config.sourceRoot);
  assertSufficientSamples(config.sourceRoot, sourceFiles);
  resetDirectory(config.rootDir);

  const distributionRoots = buildDistributionRoots(config.rootDir);
  const typeDistribution = {};
  let mtimeStep = 0;

  for (const [type, total] of Object.entries(config.counts)) {
    typeDistribution[type] = 0;
    for (let index = 0; index < total; index += 1) {
      const sourcePath = chooseSeed(sourceFiles[type], `${config.profile}:${type}:${index}`);
      const extension = extensionOf(sourcePath);
      const targetDir = distributionRoots[index % distributionRoots.length];
      const targetPath = join(targetDir, buildFileName(type, index, extension));
      mkdirSync(dirname(targetPath), { recursive: true });
      cpSync(sourcePath, targetPath);
      const timestamp = new Date(Date.UTC(2024, 0, 1, 0, 0, mtimeStep));
      utimesSync(targetPath, timestamp, timestamp);
      mtimeStep += 1;
      typeDistribution[type] += 1;
    }
  }

  const manifest = {
    datasetName: config.profile,
    generatedAt: new Date().toISOString(),
    rootDir: config.rootDir,
    sourceRoot: config.sourceRoot,
    sourceMode: 'realdata-filled',
    totalFiles: Object.values(typeDistribution).reduce((sum, count) => sum + count, 0),
    typeDistribution,
    pathCoverage: {
      english: true,
      chinese: true,
      japanese: true,
      spaces: true,
      deepNesting: true,
    },
    notes: [
      'This baseline is filled from real sample pools.',
      'Use temporary copies under data/scan-validation/runs/ for destructive tests.',
    ],
  };

  writeJson(join(config.rootDir, 'manifest.real.json'), manifest);
  return manifest;
}

function collectTypedFiles(rootDir) {
  const typedFiles = {
    image: [],
    audio: [],
    video: [],
    archive: [],
    other: [],
  };

  visitFiles(rootDir, (filePath) => {
    typedFiles[classifyMediaType(filePath)].push(filePath);
  });

  return typedFiles;
}

function visitFiles(directoryPath, visitor) {
  for (const entry of readdirSync(directoryPath, { withFileTypes: true })) {
    const entryPath = join(directoryPath, entry.name);
    if (entry.isDirectory()) {
      visitFiles(entryPath, visitor);
      continue;
    }
    if (entry.isFile() || statSync(entryPath).isFile()) {
      visitor(entryPath);
    }
  }
}

function classifyMediaType(filePath) {
  const extension = extensionOf(filePath).toLowerCase();
  if (['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp'].includes(extension)) {
    return 'image';
  }
  if (['mp3', 'flac', 'wav', 'm4a', 'ogg'].includes(extension)) {
    return 'audio';
  }
  if (['mp4', 'mkv', 'webm', 'avi', 'mov'].includes(extension)) {
    return 'video';
  }
  if (['zip', 'cbz'].includes(extension)) {
    return 'archive';
  }
  return 'other';
}

function assertSufficientSamples(sourceRoot, typedFiles) {
  for (const type of ['image', 'audio', 'video', 'archive', 'other']) {
    if (typedFiles[type].length === 0) {
      throw new Error(`source sample directory has no ${type} files: ${sourceRoot}`);
    }
  }
}

function buildDistributionRoots(rootDir) {
  return [
    join(rootDir, labels.english),
    join(rootDir, labels.chinese),
    join(rootDir, labels.japanese),
    join(rootDir, labels.spaces),
    join(rootDir, labels.english, labels.deepA, labels.deepB, labels.deepC),
    join(rootDir, labels.chinese, labels.deepA, labels.deepB),
    join(rootDir, labels.japanese, labels.deepA, labels.deepB, labels.deepC),
    join(rootDir, labels.spaces, labels.deepA, labels.deepB),
  ];
}

function buildFileName(type, index, extension) {
  const sequence = String(index + 1).padStart(4, '0');
  const families = {
    image: ['page', 'cover', 'illustration'],
    audio: ['track', 'sample', 'voice'],
    video: ['clip', 'preview', 'scene'],
    archive: ['chapter', 'bundle', 'pack'],
    other: ['note', 'meta', 'ignore'],
  };
  const family = families[type][index % families[type].length];
  const suffixes = ['alpha', '中文', '日本語', 'with space'];
  const suffix = suffixes[index % suffixes.length];
  return `${sequence}-${family}-${suffix}.${extension}`;
}

function chooseSeed(seeds, salt) {
  let hash = 2166136261;
  for (const char of salt) {
    hash ^= char.charCodeAt(0);
    hash = Math.imul(hash, 16777619);
  }
  const index = Math.abs(hash) % seeds.length;
  return seeds[index];
}

function extensionOf(filePath) {
  const name = filePath.split(/[\\/]/).at(-1);
  return name.includes('.') ? name.split('.').at(-1) : '';
}

function resetDirectory(directoryPath) {
  rmSync(directoryPath, { recursive: true, force: true });
  mkdirSync(directoryPath, { recursive: true });
}

function writeJson(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}
