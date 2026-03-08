import { z } from "zod";
import { assetIdSchema, sourceIdSchema, taskIdSchema } from "./ids.js";

export const logContextSchema = z.object({
  taskId: taskIdSchema.optional(),
  assetId: assetIdSchema.optional(),
  sourceId: sourceIdSchema.optional(),
  sessionId: z.string().min(1).optional(),
});

export const externalProcessLogSchema = z.object({
  event: z.literal("external-process"),
  phase: z.enum(["spawned", "completed", "spawn_failed", "timeout"]),
  tool: z.string().min(1),
  executable: z.string().min(1),
  arguments: z.array(z.string()),
  commandLine: z.string().min(1),
  exitCode: z.number().int().optional(),
  durationMs: z.number().int().nonnegative().optional(),
  ok: z.boolean(),
  context: logContextSchema,
  stderrExcerpt: z.string().min(1).optional(),
});

export type LogContext = z.infer<typeof logContextSchema>;
export type ExternalProcessLog = z.infer<typeof externalProcessLogSchema>;
