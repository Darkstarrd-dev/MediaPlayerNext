import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  appErrorSchema,
  archiveEntryDetailSchema,
  archiveEntrySummarySchema,
  archiveNormalizeResultSchema,
  itemDetailSchema,
  itemListEntrySchema,
  libraryChangedEventSchema,
  librarySummarySchema,
  externalProcessLogSchema,
  mediaProbeSchema,
  playbackOpenedEventSchema,
  playbackStoppedEventSchema,
  playbackSessionSchema,
  scanFinishedEventSchema,
  scanProgressEventSchema,
  scanRunResultSchema,
  subtitleSessionUpdatedEventSchema,
  subtitleSidecarCrashedEventSchema,
  subtitleHealthSchema,
  subtitleProgressEventSchema,
  subtitleSessionSchema,
  taskProgressSchema,
  thumbnailEnsureResultSchema,
  thumbnailProgressEventSchema,
} from "../src/index.js";

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturesDir = join(__dirname, "..", "fixtures");

test("task progress fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "task-progress.sample.json"), "utf8");
  const parsed = taskProgressSchema.parse(JSON.parse(raw));

  assert.equal(parsed.taskType, "scan");
  assert.equal(parsed.state, "running");
});

test("app error fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "app-error.sample.json"), "utf8");
  const parsed = appErrorSchema.parse(JSON.parse(raw));

  assert.equal(parsed.code, "DB_ERROR");
  assert.equal(parsed.retriable, true);
});

test("not found app error fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "app-error.not-found.sample.json"), "utf8");
  const parsed = appErrorSchema.parse(JSON.parse(raw));

  assert.equal(parsed.code, "NOT_FOUND");
  assert.equal(parsed.retriable, false);
});

test("timeout app error fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "app-error.timeout.sample.json"), "utf8");
  const parsed = appErrorSchema.parse(JSON.parse(raw));

  assert.equal(parsed.code, "TIMEOUT");
  assert.equal(parsed.retriable, true);
});

test("invalid argument app error fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "app-error.invalid-argument.sample.json"), "utf8");
  const parsed = appErrorSchema.parse(JSON.parse(raw));

  assert.equal(parsed.code, "INVALID_ARGUMENT");
  assert.equal(parsed.retriable, false);
});

test("external tool app error fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "app-error.external-tool.sample.json"), "utf8");
  const parsed = appErrorSchema.parse(JSON.parse(raw));

  assert.equal(parsed.code, "EXTERNAL_TOOL_ERROR");
  assert.equal(parsed.retriable, false);
});

test("library summary fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "library-summary.sample.json"), "utf8");
  const parsed = librarySummarySchema.parse(JSON.parse(raw));

  assert.equal(parsed.id, "library_001");
  assert.equal(parsed.libraryType, "filesystem");
});

test("scan run result fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "scan-run-result.sample.json"), "utf8");
  const parsed = scanRunResultSchema.parse(JSON.parse(raw));

  assert.equal(parsed.taskId, "task_scan_fixture_001");
  assert.equal(parsed.discovered, 42);
});

test("scan progress fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "scan-progress.sample.json"), "utf8");
  const parsed = scanProgressEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.taskType, "scan");
  assert.equal(parsed.current, 12);
});

test("library changed event fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "library-changed.event.sample.json"), "utf8");
  const parsed = libraryChangedEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.type, "library.changed");
  assert.equal(parsed.library.id, "library_001");
});

test("scan finished event fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "scan-finished.event.sample.json"), "utf8");
  const parsed = scanFinishedEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.type, "scan.finished");
  assert.equal(parsed.libraryId, "library_001");
});

test("item list entry fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "item-list-entry.sample.json"), "utf8");
  const parsed = itemListEntrySchema.parse(JSON.parse(raw));

  assert.equal(parsed.sourceKind, "archive_entry");
  assert.equal(parsed.thumbnailKey, "thumb_001");
});

test("item detail fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "item-detail.sample.json"), "utf8");
  const parsed = itemDetailSchema.parse(JSON.parse(raw));

  assert.equal(parsed.archiveEntryId, "archive_entry_001");
  assert.equal(parsed.archiveId, "archive_001");
});

test("external process log fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "external-process-log.sample.json"), "utf8");
  const parsed = externalProcessLogSchema.parse(JSON.parse(raw));

  assert.equal(parsed.event, "external-process");
  assert.equal(parsed.context.sessionId, "subtitle_001");
  assert.equal(parsed.tool, "subtitle-sidecar");
});

test("archive entry summary fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "archive-entry-summary.sample.json"), "utf8");
  const parsed = archiveEntrySummarySchema.parse(JSON.parse(raw));

  assert.equal(parsed.pageIndex, 0);
  assert.equal(parsed.mediaKind, "image");
});

test("archive entry detail fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "archive-entry-detail.sample.json"), "utf8");
  const parsed = archiveEntryDetailSchema.parse(JSON.parse(raw));

  assert.equal(parsed.archiveEntryId, "archive_entry_001");
  assert.equal(parsed.sourceId, "source_001");
});

test("archive normalize result fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "archive-normalize-result.sample.json"), "utf8");
  const parsed = archiveNormalizeResultSchema.parse(JSON.parse(raw));

  assert.equal(parsed.taskId, "task_normalize_001");
  assert.equal(parsed.indexedEntries, 24);
});

test("thumbnail ensure result fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "thumbnail-ensure-result.sample.json"), "utf8");
  const parsed = thumbnailEnsureResultSchema.parse(JSON.parse(raw));

  assert.equal(parsed.profile, "grid-sm");
  assert.equal(parsed.cacheHit, true);
});

test("thumbnail progress fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "thumbnail-progress.sample.json"), "utf8");
  const parsed = thumbnailProgressEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.state, "running");
  assert.equal(parsed.assetId, "asset_001");
});

test("media probe fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "media-probe.sample.json"), "utf8");
  const parsed = mediaProbeSchema.parse(JSON.parse(raw));

  assert.equal(parsed.mime, "video/mp4");
  assert.equal(parsed.videoCodec, "h264");
});

test("playback session fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "playback-session.sample.json"), "utf8");
  const parsed = playbackSessionSchema.parse(JSON.parse(raw));

  assert.equal(parsed.sessionId, "playback_001");
  assert.equal(parsed.state, "paused");
});

test("playback opened event fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "playback-opened.event.sample.json"), "utf8");
  const parsed = playbackOpenedEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.type, "playback.opened");
  assert.equal(parsed.session.sessionId, "playback_001");
});

test("playback stopped event fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "playback-stopped.event.sample.json"), "utf8");
  const parsed = playbackStoppedEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.type, "playback.stopped");
  assert.equal(parsed.session.state, "stopped");
});

test("subtitle health fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "subtitle-health.sample.json"), "utf8");
  const parsed = subtitleHealthSchema.parse(JSON.parse(raw));

  assert.equal(parsed.transport, "stdio");
  assert.equal(parsed.service, "subtitle-sidecar");
});

test("subtitle session fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "subtitle-session.sample.json"), "utf8");
  const parsed = subtitleSessionSchema.parse(JSON.parse(raw));

  assert.equal(parsed.sessionId, "subtitle_001");
  assert.equal(parsed.state, "idle");
});

test("subtitle progress fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "subtitle-progress.sample.json"), "utf8");
  const parsed = subtitleProgressEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.sessionId, "subtitle_001");
  assert.equal(parsed.message, "waiting");
});

test("subtitle sidecar crashed event fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "subtitle-sidecar-crashed.event.sample.json"), "utf8");
  const parsed = subtitleSidecarCrashedEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.type, "subtitle.sidecar.crashed");
  assert.equal(parsed.host.restartCount, 1);
});

test("subtitle session updated event fixture passes zod parse", async () => {
  const raw = await readFile(join(fixturesDir, "subtitle-session-updated.event.sample.json"), "utf8");
  const parsed = subtitleSessionUpdatedEventSchema.parse(JSON.parse(raw));

  assert.equal(parsed.type, "subtitle.session.updated");
  assert.equal(parsed.session.progress, 0.42);
});
