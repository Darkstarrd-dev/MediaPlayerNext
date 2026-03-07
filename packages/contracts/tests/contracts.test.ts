import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  appErrorSchema,
  mediaProbeSchema,
  playbackSessionSchema,
  subtitleHealthSchema,
  subtitleProgressEventSchema,
  subtitleSessionSchema,
  taskProgressSchema,
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
