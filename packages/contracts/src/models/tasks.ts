import { z } from "zod";
import { taskIdSchema } from "./ids.js";

export const taskStateSchema = z.enum([
  "queued",
  "running",
  "completed",
  "failed",
  "cancelled",
]);

export const taskKindSchema = z.enum([
  "scan",
  "ingest",
  "normalize",
  "thumbnail",
  "ffmpeg",
  "subtitle",
]);

export const taskProgressSchema = z.object({
  taskId: taskIdSchema,
  taskType: taskKindSchema,
  state: taskStateSchema,
  current: z.number().int().nonnegative(),
  total: z.number().int().nonnegative().optional(),
  message: z.string().optional(),
  errorCode: z.string().optional(),
});

export type TaskState = z.infer<typeof taskStateSchema>;
export type TaskKind = z.infer<typeof taskKindSchema>;
export type TaskProgress = z.infer<typeof taskProgressSchema>;
