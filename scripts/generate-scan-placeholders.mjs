import { spawnSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, utimesSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const workspaceRoot = resolve(__dirname, '..');
const forceRegenerate = process.argv.includes('--force');

const labels = {
  english: 'english',
  chinese: '\u4e2d\u6587\u8d44\u6599',
  japanese: '\u65e5\u672c\u8a9e\u8cc7\u6599',
  spaces: 'space folder',
  deepA: 'level-one',
  deepB: 'level two',
  deepC: 'level_three',
};

const rootTargets = {
  small: join(workspaceRoot, 'docs', 'fixtures', 'small-fixture', 'generated-placeholder'),
  medium: join(workspaceRoot, 'docs', 'fixtures', 'medium-fixture', 'generated-placeholder', 'scan-root'),
  localMock: join(workspaceRoot, 'data', 'scan-validation', 'local-real-dir-mock'),
};

const mediumRealDataRoot = join(workspaceRoot, 'docs', 'fixtures', 'medium-fixture', 'generated-placeholder', 'realdata');
const localMockRealDataRoot = join(rootTargets.localMock, 'realdata');
const mediumPlaceholderContainerRoot = dirname(rootTargets.medium);

const datasetConfigs = [
  {
    key: 'small',
    rootDir: rootTargets.small,
    profile: 'small-fixture',
    counts: { image: 12, audio: 4, video: 3, archive: 4, other: 5 },
  },
  {
    key: 'medium',
    rootDir: rootTargets.medium,
    profile: 'medium-fixture',
    counts: { image: 210, audio: 30, video: 24, archive: 42, other: 24 },
  },
  {
    key: 'localMock',
    rootDir: rootTargets.localMock,
    profile: 'local-real-dir-mock',
    counts: { image: 350, audio: 45, video: 40, archive: 55, other: 30 },
  },
];

main();

function main() {
  const seedDir = join(workspaceRoot, 'data', '.generated-media-seeds');
  resetDirectory(seedDir);
  const seedFiles = buildSeedFiles(seedDir);
  const preserveMediumContainerEntries = ['scan-root'];
  if (existsSync(mediumRealDataRoot)) {
    preserveMediumContainerEntries.push('realdata');
  }
  prepareContainerDirectory(mediumPlaceholderContainerRoot, preserveMediumContainerEntries);
  const mediumRealDataSeeds = collectTypedFiles(mediumRealDataRoot);
  const localMockRealDataSeeds = collectTypedFiles(localMockRealDataRoot);

  const manifests = [];
  for (const config of datasetConfigs) {
    manifests.push(
      generateDataset(
        config,
        chooseDatasetSeeds(config, seedFiles, mediumRealDataSeeds, localMockRealDataSeeds),
      ),
    );
  }

  const summary = {
    generatedAt: new Date().toISOString(),
    datasets: manifests,
    notes: [
      'small-fixture and medium-fixture are repo-managed placeholder datasets.',
      'local-real-dir-mock is generated under data/ and stays outside git.',
      'Existing fixture directories are preserved by default; pass --force to rebuild them.',
    ],
  };

  writeJson(join(workspaceRoot, 'docs', 'fixtures', 'generated-scan-placeholders.summary.json'), summary);

  process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
}

function chooseDatasetSeeds(config, seedFiles, mediumRealDataSeeds, localMockRealDataSeeds) {
  if (config.key !== 'medium') {
    if (config.key === 'localMock') {
      const hasRealData = Object.values(localMockRealDataSeeds).some((items) => items.length > 0);
      return {
        files: mergeSeedFiles(seedFiles, localMockRealDataSeeds),
        sourceMode: hasRealData ? 'local-realdata-randomized' : 'generated-seeds',
        preserveEntries: existsSync(localMockRealDataRoot) ? ['realdata'] : [],
        shouldRegenerate: forceRegenerate || hasRealData || !hasDatasetContent(config.rootDir, ['manifest.generated.json']),
      };
    }

    return {
      files: seedFiles,
      sourceMode: 'generated-seeds',
      preserveEntries: [],
      shouldRegenerate: forceRegenerate || !hasDatasetContent(config.rootDir, ['manifest.generated.json']),
    };
  }

  const hasRealData = Object.values(mediumRealDataSeeds).some((items) => items.length > 0);
  return {
    files: mergeSeedFiles(seedFiles, mediumRealDataSeeds),
    sourceMode: hasRealData ? 'medium-realdata-randomized' : 'generated-seeds',
    preserveEntries: existsSync(mediumRealDataRoot) ? ['realdata'] : [],
    shouldRegenerate: forceRegenerate || hasRealData || !hasDatasetContent(config.rootDir, ['manifest.generated.json']),
  };
}

function buildSeedFiles(seedDir) {
  const configPath = join(workspaceRoot, 'config', 'local.paths.json');
  const config = existsSync(configPath) ? JSON.parse(readFileSync(configPath, 'utf8')) : {};
  const ffmpegPath = config.ffmpeg;

  const paths = {
    png: join(seedDir, 'seed-image.png'),
    jpg: join(seedDir, 'seed-image.jpg'),
    wav: join(seedDir, 'seed-audio.wav'),
    mp4: join(seedDir, 'seed-video.mp4'),
    zipSource: join(seedDir, 'zip-source'),
    zip: join(seedDir, 'seed-archive.zip'),
    cbz: join(seedDir, 'seed-archive.cbz'),
    text: join(seedDir, 'seed-note.txt'),
    json: join(seedDir, 'seed-meta.json'),
    bin: join(seedDir, 'seed-data.bin'),
  };

  mkdirSync(paths.zipSource, { recursive: true });

  let usedFfmpeg = false;
  if (ffmpegPath && existsSync(ffmpegPath)) {
    run(ffmpegPath, ['-y', '-f', 'lavfi', '-i', 'color=c=0x336699:s=32x32:d=0.1', '-frames:v', '1', paths.png]);
    run(ffmpegPath, ['-y', '-f', 'lavfi', '-i', 'color=c=0xcc8844:s=40x24:d=0.1', '-frames:v', '1', paths.jpg]);
    run(ffmpegPath, ['-y', '-f', 'lavfi', '-i', 'sine=frequency=880:duration=0.15', '-c:a', 'pcm_s16le', paths.wav]);
    run(ffmpegPath, [
      '-y',
      '-f',
      'lavfi',
      '-i',
      'testsrc=size=64x48:rate=5:duration=0.4',
      '-pix_fmt',
      'yuv420p',
      '-movflags',
      '+faststart',
      paths.mp4,
    ]);
    usedFfmpeg = true;
  } else {
    writeFileSync(paths.png, Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Wn1cOQAAAAASUVORK5CYII=', 'base64'));
    writeFileSync(paths.jpg, Buffer.from([0xff, 0xd8, 0xff, 0xd9]));
    writeFileSync(paths.wav, createMinimalWav());
    writeFileSync(paths.mp4, Buffer.from('000000186674797069736f6d0000020069736f6d69736f32', 'hex'));
  }

  cpSync(paths.png, join(paths.zipSource, '001-cover.png'));
  cpSync(paths.jpg, join(paths.zipSource, '002-page.jpg'));
  cpSync(paths.png, join(paths.zipSource, '003-extra.png'));
  writeFileSync(join(paths.zipSource, 'readme.txt'), 'placeholder archive payload\n', 'utf8');

  createZipFromDirectory(paths.zipSource, paths.zip);
  cpSync(paths.zip, paths.cbz);

  writeFileSync(paths.text, 'placeholder note for scan ignore coverage\n', 'utf8');
  writeFileSync(paths.json, JSON.stringify({ placeholder: true, generatedBy: 'scan-placeholders' }, null, 2));
  writeFileSync(paths.bin, Buffer.from('placeholder-binary-payload', 'utf8'));

  return {
    image: [paths.png, paths.jpg],
    audio: [paths.wav],
    video: [paths.mp4],
    archive: [paths.zip, paths.cbz],
    other: [paths.text, paths.json, paths.bin],
    usedFfmpeg,
  };
}

function generateDataset(config, datasetSeedConfig) {
  if (!datasetSeedConfig.shouldRegenerate) {
    return loadExistingManifest(config.rootDir, config.profile);
  }

  resetDatasetDirectory(config.rootDir, datasetSeedConfig.preserveEntries);

  const distribution = buildDistributionRoots(config.rootDir);
  const generatedFiles = [];
  const typeSummaries = {};
  let mtimeStep = 0;

  for (const [type, total] of Object.entries(config.counts)) {
    typeSummaries[type] = 0;
    for (let index = 0; index < total; index += 1) {
      const sourceSeed = chooseSeed(datasetSeedConfig.files[type], `${config.profile}:${type}:${index}`);
      const extension = extensionOf(sourceSeed);
      const targetDir = distribution[index % distribution.length];
      const fileName = buildFileName(type, index, extension);
      const targetPath = join(targetDir, fileName);
      mkdirSync(dirname(targetPath), { recursive: true });
      cpSync(sourceSeed, targetPath);
      const timestamp = new Date(Date.UTC(2024, 0, 1, 0, 0, mtimeStep));
      utimesSync(targetPath, timestamp, timestamp);
      mtimeStep += 1;
      typeSummaries[type] += 1;
      generatedFiles.push(targetPath);
    }
  }

  const manifest = {
    datasetName: config.profile,
    generatedAt: new Date().toISOString(),
    rootDir: config.rootDir,
    totalFiles: generatedFiles.length,
    typeDistribution: typeSummaries,
    pathCoverage: {
      english: true,
      chinese: true,
      japanese: true,
      spaces: true,
      deepNesting: true,
    },
    mediaCompliance: {
      images: mediaComplianceMessage('image', datasetSeedConfig),
      audio: mediaComplianceMessage('audio', datasetSeedConfig),
      video: mediaComplianceMessage('video', datasetSeedConfig),
      archives: mediaComplianceMessage('archive', datasetSeedConfig),
    },
    sourceMode: datasetSeedConfig.sourceMode,
    replacementGuidance: [
      'Keep directory layout and file type mix stable when replacing placeholder files.',
      'Prefer replacing in-place with same-kind real media to preserve scan validation scenarios.',
      'For local-real-dir-mock, copy real files into data/scan-validation/local-real-dir-mock and rerun scan validation.',
    ],
  };

  writeJson(join(config.rootDir, 'manifest.generated.json'), manifest);
  return manifest;
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
  const suffixes = ['alpha', '\u4e2d\u6587', '\u65e5\u672c\u8a9e', 'with space'];
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

function mediaComplianceMessage(type, datasetSeedConfig) {
  if (
    datasetSeedConfig.sourceMode === 'medium-realdata-randomized'
    || datasetSeedConfig.sourceMode === 'local-realdata-randomized'
  ) {
    return 'randomized realdata samples when available, with generated fallback';
  }

  if (type === 'video') {
    return datasetSeedConfig.files.usedFfmpeg
      ? 'real files generated by ffmpeg'
      : 'fallback placeholder bytes';
  }

  if (type === 'archive') {
    return 'real zip/cbz built from generated image payload';
  }

  return 'real files';
}

function extensionOf(filePath) {
  const name = filePath.split(/[\\/]/).at(-1);
  return name.includes('.') ? name.split('.').at(-1) : '';
}

function resetDirectory(directoryPath) {
  rmSync(directoryPath, { recursive: true, force: true });
  mkdirSync(directoryPath, { recursive: true });
}

function resetDatasetDirectory(directoryPath, preserveEntries) {
  mkdirSync(directoryPath, { recursive: true });
  const preserve = new Set(preserveEntries);

  for (const entry of readdirSync(directoryPath, { withFileTypes: true })) {
    if (preserve.has(entry.name)) {
      continue;
    }
    rmSync(join(directoryPath, entry.name), { recursive: true, force: true });
  }
}

function prepareContainerDirectory(directoryPath, preserveEntries) {
  resetDatasetDirectory(directoryPath, preserveEntries);
}

function hasDatasetContent(directoryPath, ignoredNames) {
  if (!existsSync(directoryPath)) {
    return false;
  }

  const ignored = new Set(ignoredNames);
  return readdirSync(directoryPath).some((name) => !ignored.has(name));
}

function loadExistingManifest(directoryPath, profile) {
  const manifestPath = join(directoryPath, 'manifest.generated.json');
  if (existsSync(manifestPath)) {
    return JSON.parse(readFileSync(manifestPath, 'utf8'));
  }

  return {
    datasetName: profile,
    generatedAt: new Date().toISOString(),
    rootDir: directoryPath,
    retainedExisting: true,
  };
}

function collectTypedFiles(rootDir) {
  const typedFiles = {
    image: [],
    audio: [],
    video: [],
    archive: [],
    other: [],
  };

  if (!existsSync(rootDir)) {
    return typedFiles;
  }

  visitFiles(rootDir, (filePath) => {
    const type = classifyMediaType(filePath);
    typedFiles[type].push(filePath);
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

function mergeSeedFiles(seedFiles, realDataFiles) {
  return {
    image: realDataFiles.image.length > 0 ? realDataFiles.image : seedFiles.image,
    audio: realDataFiles.audio.length > 0 ? realDataFiles.audio : seedFiles.audio,
    video: realDataFiles.video.length > 0 ? realDataFiles.video : seedFiles.video,
    archive: realDataFiles.archive.length > 0 ? realDataFiles.archive : seedFiles.archive,
    other: realDataFiles.other.length > 0 ? realDataFiles.other : seedFiles.other,
    usedFfmpeg: seedFiles.usedFfmpeg,
  };
}

function writeJson(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function run(command, args) {
  const result = spawnSync(command, args, { stdio: 'pipe', encoding: 'utf8' });
  if (result.status !== 0) {
    const stderr = result.stderr?.trim();
    const stdout = result.stdout?.trim();
    throw new Error(`command failed: ${command} ${args.join(' ')}${stderr ? `\n${stderr}` : stdout ? `\n${stdout}` : ''}`);
  }
}

function createZipFromDirectory(sourceDir, outputPath) {
  if (existsSync(outputPath)) {
    rmSync(outputPath, { force: true });
  }

  const command = [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
    `Compress-Archive -Path '${sourceDir.replace(/'/g, "''")}\\*' -DestinationPath '${outputPath.replace(/'/g, "''")}' -Force`,
  ];
  run('powershell.exe', command);
}

function createMinimalWav() {
  const sampleRate = 8000;
  const durationSeconds = 0.1;
  const totalSamples = Math.floor(sampleRate * durationSeconds);
  const dataSize = totalSamples * 2;
  const buffer = Buffer.alloc(44 + dataSize);
  buffer.write('RIFF', 0, 4, 'ascii');
  buffer.writeUInt32LE(36 + dataSize, 4);
  buffer.write('WAVE', 8, 4, 'ascii');
  buffer.write('fmt ', 12, 4, 'ascii');
  buffer.writeUInt32LE(16, 16);
  buffer.writeUInt16LE(1, 20);
  buffer.writeUInt16LE(1, 22);
  buffer.writeUInt32LE(sampleRate, 24);
  buffer.writeUInt32LE(sampleRate * 2, 28);
  buffer.writeUInt16LE(2, 32);
  buffer.writeUInt16LE(16, 34);
  buffer.write('data', 36, 4, 'ascii');
  buffer.writeUInt32LE(dataSize, 40);
  for (let i = 0; i < totalSamples; i += 1) {
    const sample = Math.round(Math.sin((i / 8) * Math.PI * 2) * 32767 * 0.2);
    buffer.writeInt16LE(sample, 44 + i * 2);
  }
  return buffer;
}
