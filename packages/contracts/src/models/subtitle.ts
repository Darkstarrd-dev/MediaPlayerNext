import { z } from "zod";
import { assetIdSchema, subtitleSessionIdSchema } from "./ids.js";

export const subtitleTransportSchema = z.literal("stdio");

export const subtitlePingSchema = z.object({
  service: z.literal("subtitle-sidecar"),
  protocolVersion: z.string().min(1),
  transport: subtitleTransportSchema,
});

export const subtitleHealthSchema = subtitlePingSchema.extend({
  nodeVersion: z.string().min(1),
  sharpVersion: z.string().min(1).optional(),
  uptimeMs: z.number().int().nonnegative(),
  activeSessions: z.number().int().nonnegative(),
  sessionsRoot: z.string().min(1),
});

export const subtitleHostSchema = z.object({
  executable: z.string().min(1),
  entryPath: z.string().min(1),
  running: z.boolean(),
  restartCount: z.number().int().nonnegative(),
  lastError: z.string().min(1).optional(),
  ping: subtitlePingSchema.optional(),
  health: subtitleHealthSchema.optional(),
});

export const subtitleSessionStateSchema = z.enum([
  "idle",
  "running",
  "stopped",
  "failed",
]);

export const subtitleSessionSchema = z.object({
  sessionId: subtitleSessionIdSchema,
  assetId: assetIdSchema.optional(),
  state: subtitleSessionStateSchema,
  progress: z.number().min(0).max(1),
  createdAt: z.string(),
  updatedAt: z.string(),
  outputPath: z.string().min(1).optional(),
});

export const subtitleProgressSchema = z.object({
  sessionId: subtitleSessionIdSchema,
  state: subtitleSessionStateSchema,
  progress: z.number().min(0).max(1),
  message: z.string().min(1).optional(),
});

export type SubtitleTransport = z.infer<typeof subtitleTransportSchema>;
export type SubtitlePing = z.infer<typeof subtitlePingSchema>;
export type SubtitleHealth = z.infer<typeof subtitleHealthSchema>;
export type SubtitleHost = z.infer<typeof subtitleHostSchema>;
export type SubtitleSessionState = z.infer<typeof subtitleSessionStateSchema>;
export type SubtitleSession = z.infer<typeof subtitleSessionSchema>;
export type SubtitleProgress = z.infer<typeof subtitleProgressSchema>;
