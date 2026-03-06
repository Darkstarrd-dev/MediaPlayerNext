import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { appErrorSchema, taskProgressSchema } from "../src/index.js";

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
