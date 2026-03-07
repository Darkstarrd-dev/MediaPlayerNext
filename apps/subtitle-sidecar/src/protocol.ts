import { z } from "zod";

export const protocolVersion = "b8-v1";
export const subtitleServiceName = "subtitle-sidecar";

export const requestTypeSchema = z.enum([
  "ping",
  "health",
  "start_session",
  "stop_session",
  "get_progress",
  "export_srt",
  "shutdown",
]);

export const appErrorSchema = z.object({
  code: z.string().min(1),
  message: z.string().min(1),
  retriable: z.boolean(),
});

export const sidecarRequestSchema = z.object({
  id: z.string().min(1),
  type: requestTypeSchema,
  payload: z.record(z.string(), z.unknown()).optional(),
});

export const sidecarResponseSchema = z.object({
  id: z.string().min(1),
  type: z.literal("response"),
  ok: z.boolean(),
  payload: z.unknown().optional(),
  error: appErrorSchema.optional(),
});

export const subtitleSessionStateSchema = z.enum([
  "idle",
  "running",
  "stopped",
  "failed",
]);

export const subtitleSessionSchema = z.object({
  sessionId: z.string().min(1),
  assetId: z.string().min(1).optional(),
  state: subtitleSessionStateSchema,
  progress: z.number().min(0).max(1),
  createdAt: z.string(),
  updatedAt: z.string(),
  outputPath: z.string().min(1).optional(),
});

export const subtitleProgressSchema = z.object({
  sessionId: z.string().min(1),
  state: subtitleSessionStateSchema,
  progress: z.number().min(0).max(1),
  message: z.string().min(1).optional(),
});

export const startSessionPayloadSchema = z
  .object({
    assetId: z.string().min(1).optional(),
  })
  .optional();

export const stopSessionPayloadSchema = z.object({
  sessionId: z.string().min(1),
});

export const getProgressPayloadSchema = stopSessionPayloadSchema;

export type AppError = z.infer<typeof appErrorSchema>;
export type SidecarRequest = z.infer<typeof sidecarRequestSchema>;
export type SidecarResponse = z.infer<typeof sidecarResponseSchema>;
export type SubtitleSession = z.infer<typeof subtitleSessionSchema>;
export type SubtitleProgress = z.infer<typeof subtitleProgressSchema>;
